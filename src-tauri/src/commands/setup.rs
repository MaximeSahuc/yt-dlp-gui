//! Platform info, yt-dlp and Deno installation management

use crate::utils;
use futures_util::StreamExt;
use std::process::Stdio;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncBufReadExt;

use super::common;
use super::{DenoStatus, FfmpegStatus, YtdlpStatus};

/// HTTP download timeout (30 minutes, for large file downloads)
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(1800);

#[cfg(target_os = "windows")]
use super::CREATE_NO_WINDOW;

// ========== Platform info ==========

/// Get the current runtime platform
#[tauri::command]
pub fn get_platform() -> String {
    if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "macos") {
        "macos".to_string()
    } else {
        "linux".to_string()
    }
}

/// Set YouTube extractor arguments (PO Token / visitor_data) to bypass YouTube 403 / rate limiting
#[tauri::command]
pub fn set_youtube_extractor_args(po_token: String, visitor_data: String) -> Result<(), String> {
    utils::set_youtube_extractor_args(&po_token, &visitor_data)
}

// ========== yt-dlp management ==========

/// Get yt-dlp installation status and version
#[tauri::command]
pub async fn get_ytdlp_status(app: AppHandle) -> Result<YtdlpStatus, String> {
    let ytdlp_path = utils::get_ytdlp_path(&app)?;
    let managed_path = utils::get_managed_ytdlp_path(&app)?;
    let is_managed = ytdlp_path == managed_path;

    if !ytdlp_path.exists() {
        return Ok(YtdlpStatus {
            installed: false,
            version: String::new(),
            path: ytdlp_path.to_string_lossy().to_string(),
            is_managed,
        });
    }

    let mut cmd = tokio::process::Command::new(&ytdlp_path);
    cmd.arg("--version")
        .env("PYTHONUTF8", "1")
        .env("PYTHONIOENCODING", "utf-8");
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("err_run_ytdlp:{}", e))?;

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(YtdlpStatus {
        installed: true,
        version,
        path: ytdlp_path.to_string_lossy().to_string(),
        is_managed,
    })
}

/// Download the yt-dlp executable
#[tauri::command]
pub async fn download_ytdlp(app: AppHandle) -> Result<(), String> {
    let ytdlp_path = utils::get_managed_ytdlp_path(&app)?;
    let url = utils::get_ytdlp_download_url();

    let client = reqwest::Client::builder()
        .timeout(DOWNLOAD_TIMEOUT)
        .build()
        .map_err(|e| format!("err_create_http_client:{}", e))?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("err_download_failed:{}", e))?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = tokio::fs::File::create(&ytdlp_path)
        .await
        .map_err(|e| format!("err_create_file:{}", e))?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("err_download_error:{}", e))?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
            .await
            .map_err(|e| format!("err_write_error:{}", e))?;

        downloaded += chunk.len() as u64;
        let percent = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        let _ = app.emit(
            "ytdlp-download-progress",
            serde_json::json!({
                "percent": percent,
                "downloaded": downloaded,
                "total": total_size,
            }),
        );
    }

    // Unix: set executable permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&ytdlp_path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("err_set_permissions:{}", e))?;
    }

    Ok(())
}

/// Update yt-dlp to the latest version (always updates the app-managed copy, not the system-installed one)
#[tauri::command]
pub async fn update_ytdlp(app: AppHandle) -> Result<String, String> {
    // System-installed yt-dlp is typically in a protected directory, so `-U` self-update fails due to permissions;
    // only the copy in the app data directory is updated here, consistent with download_ytdlp.
    let ytdlp_path = utils::get_managed_ytdlp_path(&app)?;
    if !ytdlp_path.exists() {
        return Err("err_ytdlp_not_installed".to_string());
    }

    let mut cmd = tokio::process::Command::new(&ytdlp_path);
    cmd.arg("-U")
        .env("PYTHONUTF8", "1")
        .env("PYTHONIOENCODING", "utf-8");
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("err_start_update:{}", e))?;

    let stdout = child.stdout.take().ok_or("err_capture_stdout")?;
    let stderr = child.stderr.take().ok_or("err_capture_stderr")?;

    let app_clone = app.clone();
    let stdout_handle = tokio::spawn(async move {
        let reader = tokio::io::BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut output = String::new();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_clone.emit("ytdlp-update-log", &line);
            output.push_str(&line);
            output.push('\n');
        }
        output
    });

    let app_clone2 = app.clone();
    let stderr_handle = tokio::spawn(async move {
        let reader = tokio::io::BufReader::new(stderr);
        let mut lines = reader.lines();
        let mut output = String::new();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_clone2.emit("ytdlp-update-log", &line);
            output.push_str(&line);
            output.push('\n');
        }
        output
    });

    let stdout_out = stdout_handle.await.unwrap_or_default();
    let stderr_out = stderr_handle.await.unwrap_or_default();

    let status = child
        .wait()
        .await
        .map_err(|e| format!("err_process:{}", e))?;

    if status.success() {
        Ok(format!("{}\n{}", stdout_out, stderr_out).trim().to_string())
    } else {
        Err(format!("err_update_failed:{}", stderr_out.trim()))
    }
}

// ========== yt-dlp plugin management ==========

/// Check whether a plugin is installed (by checking if the file exists at the relative path)
#[tauri::command]
pub async fn check_plugin_installed(app: AppHandle, file_path: String) -> Result<bool, String> {
    let plugin_dir = utils::get_plugin_dir(&app)?;
    Ok(plugin_dir.join(&file_path).exists())
}

/// Uninstall a yt-dlp plugin (delete the specified file)
#[tauri::command]
pub async fn uninstall_plugin(app: AppHandle, file_path: String) -> Result<(), String> {
    let plugin_dir = utils::get_plugin_dir(&app)?;
    // Path safety check: ensure the target file is within the plugin directory to prevent path traversal attacks
    let target = common::validate_path_within(&plugin_dir, &file_path)?;
    if target.exists() {
        tokio::fs::remove_file(&target)
            .await
            .map_err(|e| format!("err_delete_file:{}", e))?;
    }
    Ok(())
}

/// Download and install a yt-dlp plugin (zip format, automatically extracted to the plugin directory)
#[tauri::command]
pub async fn install_plugin(app: AppHandle, url: String) -> Result<(), String> {
    let plugin_dir = utils::get_plugin_dir(&app)?;

    let client = reqwest::Client::builder()
        .timeout(DOWNLOAD_TIMEOUT)
        .build()
        .map_err(|e| format!("err_create_http_client:{}", e))?;
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("err_download_failed:{}", e))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("err_download_error:{}", e))?;

    // Extract the zip, preserving the directory structure under yt_dlp_plugins/
    let plugin_dir_clone = plugin_dir.clone();
    tokio::task::spawn_blocking(move || {
        let cursor = std::io::Cursor::new(bytes);
        let mut archive =
            zip::ZipArchive::new(cursor).map_err(|e| format!("err_read_zip:{}", e))?;

        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| format!("err_read_zip_entry:{}", e))?;
            let name = entry.name().to_string();

            // Only extract .py files under yt_dlp_plugins/, preserving the subdirectory structure
            if let Some(rel) = name.strip_prefix("yt_dlp_plugins/") {
                if !rel.is_empty() && !entry.is_dir() {
                    let out_path = plugin_dir_clone.join("yt_dlp_plugins").join(rel);
                    if let Some(parent) = out_path.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| format!("err_create_dir:{}", e))?;
                    }
                    let mut outfile = std::fs::File::create(&out_path)
                        .map_err(|e| format!("err_create_file:{}", e))?;
                    std::io::copy(&mut entry, &mut outfile)
                        .map_err(|e| format!("err_write_error:{}", e))?;
                }
            }
        }
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("err_task:{}", e))??;

    Ok(())
}

// ========== Deno management ==========

/// Get Deno installation status and version
#[tauri::command]
pub async fn get_deno_status(app: AppHandle) -> Result<DenoStatus, String> {
    let deno_path = utils::get_deno_path(&app)?;
    let managed_path = utils::get_managed_deno_path(&app)?;
    let is_managed = deno_path == managed_path;

    if !deno_path.exists() {
        return Ok(DenoStatus {
            installed: false,
            version: String::new(),
            path: deno_path.to_string_lossy().to_string(),
            is_managed,
        });
    }

    let mut cmd = tokio::process::Command::new(&deno_path);
    cmd.arg("--version");
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd.output().await;

    match output {
        Ok(out) if out.status.success() => {
            let version_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let version = version_str
                .lines()
                .next()
                .unwrap_or("")
                .replace("deno ", "")
                .trim()
                .to_string();
            Ok(DenoStatus {
                installed: true,
                version,
                path: deno_path.to_string_lossy().to_string(),
                is_managed,
            })
        }
        _ => Ok(DenoStatus {
            installed: true,
            version: String::new(),
            path: deno_path.to_string_lossy().to_string(),
            is_managed,
        }),
    }
}

/// Download the Deno executable (extracted from a zip archive)
#[tauri::command]
pub async fn download_deno(app: AppHandle) -> Result<(), String> {
    let deno_path = utils::get_managed_deno_path(&app)?;
    let url = utils::get_deno_download_url();

    let client = reqwest::Client::builder()
        .timeout(DOWNLOAD_TIMEOUT)
        .build()
        .map_err(|e| format!("err_create_http_client:{}", e))?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("err_download_failed:{}", e))?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    // Download the zip to a temporary file
    let zip_path = deno_path.with_extension("zip");
    let mut file = tokio::fs::File::create(&zip_path)
        .await
        .map_err(|e| format!("err_create_file:{}", e))?;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("err_download_error:{}", e))?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
            .await
            .map_err(|e| format!("err_write_error:{}", e))?;

        downloaded += chunk.len() as u64;
        let percent = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };
        let _ = app.emit(
            "deno-download-progress",
            serde_json::json!({
                "percent": percent,
                "downloaded": downloaded,
                "total": total_size,
            }),
        );
    }

    // Ensure the file is fully flushed to disk
    tokio::io::AsyncWriteExt::shutdown(&mut file)
        .await
        .map_err(|e| format!("err_flush_file:{}", e))?;
    drop(file);

    // Extract the deno executable
    let zip_path_clone = zip_path.clone();
    let deno_path_clone = deno_path.clone();
    let deno_bin_name = if cfg!(target_os = "windows") {
        "deno.exe"
    } else {
        "deno"
    };

    tokio::task::spawn_blocking(move || {
        let file =
            std::fs::File::open(&zip_path_clone).map_err(|e| format!("err_open_zip:{}", e))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("err_read_zip:{}", e))?;

        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| format!("err_read_zip_entry:{}", e))?;
            let name = entry.name().to_lowercase();
            if name == deno_bin_name || name.ends_with(&format!("/{}", deno_bin_name)) {
                let mut outfile = std::fs::File::create(&deno_path_clone)
                    .map_err(|e| format!("err_create_file:{}", e))?;
                std::io::copy(&mut entry, &mut outfile)
                    .map_err(|e| format!("err_extract_deno:{}", e))?;
                return Ok(());
            }
        }
        Err(format!("err_not_found_in_zip:{}", deno_bin_name))
    })
    .await
    .map_err(|e| format!("err_task:{}", e))??;

    // Unix: set executable permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&deno_path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("err_set_permissions:{}", e))?;
    }

    // Clean up the zip file
    let _ = tokio::fs::remove_file(&zip_path).await;

    Ok(())
}

// ========== ffmpeg management ==========

/// Get ffmpeg installation status and version.
/// Prefers the app-managed copy, falling back to ffmpeg found in the system PATH.
#[tauri::command]
pub async fn get_ffmpeg_status(app: AppHandle) -> Result<FfmpegStatus, String> {
    let managed_path = utils::get_managed_ffmpeg_path(&app)?;

    let (ffmpeg_path, is_managed) = if managed_path.exists() {
        (managed_path, true)
    } else if let Ok(system_path) = which::which("ffmpeg") {
        (system_path, false)
    } else {
        return Ok(FfmpegStatus {
            installed: false,
            version: String::new(),
            path: managed_path.to_string_lossy().to_string(),
            is_managed: true,
        });
    };

    let mut cmd = tokio::process::Command::new(&ffmpeg_path);
    cmd.arg("-version");
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd.output().await;
    let version = match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .trim_start_matches("ffmpeg version ")
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string(),
        _ => String::new(),
    };

    Ok(FfmpegStatus {
        installed: true,
        version,
        path: ffmpeg_path.to_string_lossy().to_string(),
        is_managed,
    })
}

/// Download ffmpeg (extract ffmpeg and ffprobe from a zip archive to the app data directory).
/// In-app download is only available on Windows; for other platforms, install via the system package manager.
#[tauri::command]
pub async fn download_ffmpeg(app: AppHandle) -> Result<(), String> {
    let url = utils::get_ffmpeg_download_url().ok_or("err_ffmpeg_no_managed_build")?;
    let ffmpeg_path = utils::get_managed_ffmpeg_path(&app)?;
    let dest_dir = ffmpeg_path
        .parent()
        .ok_or("err_app_data_dir")?
        .to_path_buf();

    let client = reqwest::Client::builder()
        .timeout(DOWNLOAD_TIMEOUT)
        .build()
        .map_err(|e| format!("err_create_http_client:{}", e))?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("err_download_failed:{}", e))?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    // Download the zip to a temporary file
    let zip_path = ffmpeg_path.with_extension("zip");
    let mut file = tokio::fs::File::create(&zip_path)
        .await
        .map_err(|e| format!("err_create_file:{}", e))?;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("err_download_error:{}", e))?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
            .await
            .map_err(|e| format!("err_write_error:{}", e))?;

        downloaded += chunk.len() as u64;
        let percent = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };
        let _ = app.emit(
            "ffmpeg-download-progress",
            serde_json::json!({
                "percent": percent,
                "downloaded": downloaded,
                "total": total_size,
            }),
        );
    }

    tokio::io::AsyncWriteExt::shutdown(&mut file)
        .await
        .map_err(|e| format!("err_flush_file:{}", e))?;
    drop(file);

    // Extract ffmpeg / ffprobe executables (located in the .../bin/ subdirectory within the zip)
    let zip_path_clone = zip_path.clone();
    tokio::task::spawn_blocking(move || {
        let file =
            std::fs::File::open(&zip_path_clone).map_err(|e| format!("err_open_zip:{}", e))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("err_read_zip:{}", e))?;

        let wanted = ["ffmpeg.exe", "ffprobe.exe"];
        let mut found_ffmpeg = false;

        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| format!("err_read_zip_entry:{}", e))?;
            if entry.is_dir() {
                continue;
            }
            let name = entry.name().replace('\\', "/");
            let base = name.rsplit('/').next().unwrap_or("").to_lowercase();
            if wanted.contains(&base.as_str()) {
                let out_path = dest_dir.join(&base);
                let mut outfile = std::fs::File::create(&out_path)
                    .map_err(|e| format!("err_create_file:{}", e))?;
                std::io::copy(&mut entry, &mut outfile)
                    .map_err(|e| format!("err_write_error:{}", e))?;
                if base == "ffmpeg.exe" {
                    found_ffmpeg = true;
                }
            }
        }

        if !found_ffmpeg {
            return Err("err_not_found_in_zip:ffmpeg".to_string());
        }
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("err_task:{}", e))??;

    let _ = tokio::fs::remove_file(&zip_path).await;

    Ok(())
}
