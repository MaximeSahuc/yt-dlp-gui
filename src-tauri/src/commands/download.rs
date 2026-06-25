//! Download task control: start, pause, resume, cancel, file check

use crate::parser;
use crate::process;
use crate::utils;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

use super::common::append_cookie_proxy_args;
use super::{DownloadParams, DownloadProcessInfo, DownloadState};

#[cfg(target_os = "windows")]
use super::CREATE_NO_WINDOW;

// ========== Helper functions ==========

/// Format a duration in seconds as HH:MM:SS
fn format_duration(secs: f64) -> String {
    let total = secs as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}

// ========== Output processing ==========

/// Process a single line of yt-dlp output: parse progress and emit events to the frontend
fn process_output_line(
    app: &AppHandle,
    task_id: &str,
    processes: &Arc<Mutex<HashMap<String, DownloadProcessInfo>>>,
    line: &str,
) {
    // Parse the JSON progress output from --progress-template
    if let Some(info) = parser::parse_progress_json(line) {
        let _ = app.emit(
            "download-progress",
            serde_json::json!({
                "id": task_id,
                "percent": info.percent,
                "speed": info.speed,
                "eta": info.eta,
                "downloaded": info.downloaded,
                "total": info.total,
            }),
        );
        return; // Progress lines do not need to be forwarded to the log
    }

    // Parse the time= field from ffmpeg output (used for progress in time-clip mode)
    if line.contains("time=") && line.contains("frame=") {
        if let Some(current_secs) = parser::parse_ffmpeg_time(line) {
            let clip_dur = processes
                .lock()
                .ok()
                .and_then(|map| map.get(task_id).and_then(|info| info.clip_duration));
            if let Some(duration) = clip_dur {
                let percent = (current_secs / duration * 100.0).min(100.0);
                let _ = app.emit(
                    "download-progress",
                    serde_json::json!({
                        "id": task_id,
                        "percent": percent,
                        "speed": "",
                        "eta": "",
                        "downloaded": format_duration(current_secs),
                        "total": format_duration(duration),
                    }),
                );
            }
        }
        return; // ffmpeg frame progress is not forwarded to the log
    }

    // Track the output file path (parsed from lines like [download] Destination, used as a fallback)
    if let Some(dest) = parse_destination(line) {
        if let Ok(mut map) = processes.lock() {
            if let Some(info) = map.get_mut(task_id) {
                info.output_files.push(dest);
            }
        }
    }

    // Forward log lines to the frontend (excluding progress JSON lines, to keep the log clean)
    let _ = app.emit(
        "download-log",
        serde_json::json!({ "id": task_id, "line": line }),
    );
}

/// Parse the destination file path from a yt-dlp output line (fallback; may have encoding issues)
fn parse_destination(line: &str) -> Option<String> {
    let trimmed = line.trim();
    // [download] Destination: /path/to/file.ext
    if let Some(rest) = trimmed.strip_prefix("[download] Destination: ") {
        return Some(rest.trim().to_string());
    }
    // [download] /path/to/file.ext has already been downloaded
    if trimmed.starts_with("[download] ") && trimmed.ends_with("has already been downloaded") {
        let inner = trimmed
            .strip_prefix("[download] ")?
            .strip_suffix("has already been downloaded")?
            .trim();
        if !inner.is_empty() {
            return Some(inner.to_string());
        }
    }
    // [Merger] Merging formats into "file.ext"
    if trimmed.contains("[Merger] Merging formats into") {
        let start = trimmed.find('"')? + 1;
        let end = trimmed.rfind('"')?;
        if start < end {
            return Some(trimmed[start..end].to_string());
        }
    }
    None
}

/// Read the final output file path written by yt-dlp's --print-to-file from a temporary file
/// Returns the last line (playlists may produce multiple lines)
fn read_filepath_from_file(filepath_file: &str) -> Option<String> {
    let content = std::fs::read_to_string(filepath_file).ok()?;
    let last_line = content.trim().lines().last()?.trim().to_string();
    if last_line.is_empty() {
        None
    } else {
        Some(last_line)
    }
}

// ========== Download commands ==========

/// Start a download task
#[tauri::command]
pub async fn start_download(
    app: AppHandle,
    state: tauri::State<'_, DownloadState>,
    params: DownloadParams,
) -> Result<(), String> {
    let ytdlp_path = utils::get_ytdlp_path(&app)?;
    if !ytdlp_path.exists() {
        return Err("err_ytdlp_not_installed".to_string());
    }

    let args = build_download_args(&app, &params)?;

    // Generate a temporary file path for --print-to-file to write the final output path
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("err_app_data_dir:{}", e))?;
    let filepath_file = app_data
        .join(format!("{}_filepath.txt", params.id))
        .to_string_lossy()
        .to_string();

    // Assemble full arguments: base args + --print-to-file
    let mut full_args = args;
    full_args.push("--print-to-file".to_string());
    full_args.push("after_move:filepath".to_string());
    full_args.push(filepath_file.clone());

    // Launch the yt-dlp subprocess
    let mut cmd = tokio::process::Command::new(&ytdlp_path);
    cmd.args(&full_args)
        .env("PYTHONUTF8", "1")
        .env("PYTHONIOENCODING", "utf-8");
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("err_start_download:{}", e))?;

    let pid = child.id().ok_or("err_get_pid")?;
    let task_id = params.id.clone();

    // Calculate the clip segment duration (used for ffmpeg progress calculation)
    let clip_duration = match (params.start_time, params.end_time) {
        (Some(s), Some(e)) => Some(e - s),
        (None, Some(e)) => Some(e),
        _ => None,
    };

    // Record process info
    let processes = state.processes.clone();
    {
        let mut map = processes.lock().map_err(|e| e.to_string())?;
        map.insert(
            task_id.clone(),
            DownloadProcessInfo {
                pid,
                cancelled: false,
                output_files: Vec::new(),
                download_dir: params.download_dir.clone(),
                filepath_file: Some(filepath_file),
                clip_duration,
            },
        );
    }

    let stdout = child.stdout.take().ok_or("err_capture_stdout")?;
    let stderr = child.stderr.take().ok_or("err_capture_stderr")?;

    // Read stdout (raw bytes, lossy-decoded to handle Windows GBK encoding)
    spawn_output_reader(app.clone(), task_id.clone(), processes.clone(), stdout);
    // Read stderr
    spawn_output_reader(app.clone(), task_id.clone(), processes.clone(), stderr);

    // Wait for the process to finish and handle the result
    spawn_completion_handler(app.clone(), task_id, processes.clone(), child);

    Ok(())
}

/// Build yt-dlp download arguments
fn build_download_args(app: &AppHandle, params: &DownloadParams) -> Result<Vec<String>, String> {
    let mut args: Vec<String> = vec![
        "--newline".to_string(),
        "--ignore-config".to_string(),  // Ignore user system config to prevent interfering with the GUI
        "--color".to_string(), "never".to_string(),  // Disable ANSI color escape sequences
        // Use --progress-template for structured progress output (the recommended approach, avoids parsing regular stdout)
        "--progress-template".to_string(),
        r#"download:PROGRESS_JSON:{"percent":"%(progress._percent_str|0%)s","speed":"%(progress._speed_str|)s","eta":"%(progress._eta_str|)s","downloaded":"%(progress._downloaded_bytes_str|)s","total":"%(progress._total_bytes_str|)s"}"#.to_string(),
    ];

    // JS runtime (Deno)
    args.extend(utils::build_js_runtime_args(app));
    args.extend(utils::build_plugin_args(app));
    // App-managed ffmpeg (if installed), to ensure audio transcoding is available
    args.extend(utils::build_ffmpeg_location_args(app));
    // YouTube PO Token / visitor_data (if configured)
    args.extend(utils::build_youtube_extractor_args());

    // Format selection
    match params.download_mode.as_str() {
        "video" => {
            if let Some(ref vf) = params.video_format {
                if !vf.is_empty() {
                    args.push("-f".to_string());
                    args.push(vf.clone());
                }
            }
        }
        "audio" => {
            if let Some(ref af) = params.audio_format {
                if !af.is_empty() {
                    args.push("-f".to_string());
                    args.push(af.clone());
                }
            }
        }
        _ => {
            let vf = params.video_format.as_deref().filter(|s| !s.is_empty());
            let af = params.audio_format.as_deref().filter(|s| !s.is_empty());
            match (vf, af) {
                (Some(v), Some(a)) => {
                    args.push("-f".to_string());
                    args.push(format!("{}+{}", v, a));
                }
                (Some(v), None) => {
                    args.push("-f".to_string());
                    args.push(format!("{}+bestaudio", v));
                }
                (None, Some(a)) => {
                    args.push("-f".to_string());
                    args.push(format!("bestvideo+{}", a));
                }
                _ => {}
            }
        }
    }

    // Proxy
    if let Some(ref proxy) = params.proxy {
        if !proxy.is_empty() {
            args.push("--proxy".to_string());
            args.push(proxy.clone());
        }
    }

    // Output path template
    let template = params
        .output_template
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("%(title).200s.%(ext)s");
    let output_template = std::path::PathBuf::from(&params.download_dir)
        .join(template)
        .to_string_lossy()
        .to_string();
    args.push("-o".to_string());
    args.push(output_template);
    args.push("--windows-filenames".to_string());

    // Do not overwrite existing files
    if params.no_overwrites {
        args.push("--no-overwrites".to_string());
    }

    // Concurrent fragment download
    if let Some(n) = params.concurrent_fragments {
        if n > 1 {
            args.push("--concurrent-fragments".to_string());
            args.push(n.to_string());
        }
    }

    // Cookie and browser Cookie
    append_cookie_proxy_args(
        &mut args,
        params.cookie_file.as_deref(),
        params.cookie_browser.as_deref(),
        None, // proxy is handled separately above
    );

    // Additional options
    if params.embed_subs {
        args.push("--embed-subs".to_string());
    }
    if params.embed_thumbnail {
        args.push("--embed-thumbnail".to_string());
    }
    if params.embed_metadata {
        args.push("--embed-metadata".to_string());
    }
    // Embed chapter markers
    if params.embed_chapters {
        args.push("--embed-chapters".to_string());
    }
    // SponsorBlock: remove sponsor segments
    if params.sponsorblock_remove {
        args.push("--sponsorblock-remove".to_string());
        args.push("all".to_string());
    }
    // Extract audio mode
    if params.extract_audio {
        args.push("-x".to_string());
        if let Some(ref fmt) = params.audio_convert_format {
            if !fmt.is_empty() {
                args.push("--audio-format".to_string());
                args.push(fmt.clone());
            }
        }
        if let Some(ref q) = params.audio_quality {
            if !q.is_empty() {
                args.push("--audio-quality".to_string());
                args.push(q.clone());
            }
        }
    }
    if params.no_merge {
        args.push("--no-merge-output".to_string());
    }
    if let Some(ref rate) = params.limit_rate {
        if !rate.is_empty() {
            args.push("-r".to_string());
            args.push(rate.clone());
        }
    }
    // Custom FFmpeg post-processing arguments
    if let Some(ref ffmpeg_args) = params.ffmpeg_args {
        if !ffmpeg_args.is_empty() {
            args.push("--postprocessor-args".to_string());
            args.push(ffmpeg_args.clone());
        }
    }

    // Subtitles
    if !params.subtitles.is_empty() {
        args.push("--write-subs".to_string());
        args.push("--sub-langs".to_string());
        args.push(params.subtitles.join(","));
    }

    // Time range clipping (only added when there is an actual clip range, to avoid *0-inf triggering unnecessary ffmpeg processing)
    // The frontend has already converted time picker values to seconds
    let has_start = params.start_time.is_some_and(|t| t > 0.0);
    let has_end = params.end_time.is_some();
    if has_start || has_end {
        let start = params.start_time.unwrap_or(0.0);
        let end_str = params
            .end_time
            .map(|t| format!("{}", t))
            .unwrap_or_else(|| "inf".to_string());
        args.push("--download-sections".to_string());
        args.push(format!("*{}-{}", start, end_str));
    }

    // Playlist
    if params.no_playlist {
        args.push("--no-playlist".to_string());
    } else if let Some(ref items) = params.playlist_items {
        if !items.is_empty() {
            args.push("--playlist-items".to_string());
            args.push(items.clone());
        }
    }

    // URL (must be placed last)
    args.push(params.url.clone());

    Ok(args)
}

/// Spawn an async task to read the subprocess output stream
/// Handles both \n and \r as line separators (ffmpeg progress output uses \r)
fn spawn_output_reader<R: tokio::io::AsyncRead + Unpin + Send + 'static>(
    app: AppHandle,
    task_id: String,
    processes: Arc<Mutex<HashMap<String, DownloadProcessInfo>>>,
    reader: R,
) {
    tokio::spawn(async move {
        use tokio::io::AsyncReadExt;
        let mut buf_reader = tokio::io::BufReader::new(reader);
        const MAX_LINE_LEN: usize = 64 * 1024; // 64KB
        let mut line_buf = Vec::with_capacity(1024);
        let mut byte_buf = [0u8; 1];

        loop {
            match buf_reader.read(&mut byte_buf).await {
                Ok(0) => {
                    // EOF: process any remaining content in the buffer
                    if !line_buf.is_empty() {
                        let line = String::from_utf8_lossy(&line_buf).trim().to_string();
                        if !line.is_empty() {
                            process_output_line(&app, &task_id, &processes, &line);
                        }
                    }
                    break;
                }
                Ok(_) => {
                    if byte_buf[0] == b'\n' || byte_buf[0] == b'\r' {
                        if !line_buf.is_empty() {
                            let line = String::from_utf8_lossy(&line_buf).trim().to_string();
                            if !line.is_empty() {
                                process_output_line(&app, &task_id, &processes, &line);
                            }
                            line_buf.clear();
                        }
                    } else if line_buf.len() < MAX_LINE_LEN {
                        line_buf.push(byte_buf[0]);
                    }
                }
                Err(_) => break,
            }
        }
    });
}

/// Spawn an async task to wait for the subprocess to finish and emit a result event
fn spawn_completion_handler(
    app: AppHandle,
    task_id: String,
    processes: Arc<Mutex<HashMap<String, DownloadProcessInfo>>>,
    mut child: tokio::process::Child,
) {
    tokio::spawn(async move {
        let status = child.wait().await;

        let was_cancelled = processes
            .lock()
            .ok()
            .and_then(|map| map.get(&task_id).map(|info| info.cancelled))
            .unwrap_or(false);

        // Use only the yt-dlp exit code to determine success; do not fall back on seeing a Destination line in the log,
        // because yt-dlp prints the destination path before writing any bytes, so a mid-download timeout also leaves this line.
        let success = matches!(&status, Ok(s) if s.success());

        if success {
            let (output_file, _) = resolve_output_file(&processes, &task_id);
            let _ = app.emit(
                "download-complete",
                serde_json::json!({ "id": task_id, "outputFile": output_file }),
            );
        } else if !was_cancelled {
            // On failure, still clean up the --print-to-file temporary file to avoid leftover files
            let _ = resolve_output_file(&processes, &task_id);
            let error_msg = status
                .as_ref()
                .map(|s| format!("err_exit_code:{}", s.code().unwrap_or(-1)))
                .unwrap_or_else(|e| e.to_string());
            let _ = app.emit(
                "download-error",
                serde_json::json!({
                    "id": task_id,
                    "error": error_msg,
                }),
            );
        }

        // Remove the process record
        if let Ok(mut map) = processes.lock() {
            map.remove(&task_id);
        }
    });
}

/// Resolve the final output file path
/// Prefers reading from the --print-to-file temporary file (reliable UTF-8), falls back to the stdout-parsed path
fn resolve_output_file(
    processes: &Arc<Mutex<HashMap<String, DownloadProcessInfo>>>,
    task_id: &str,
) -> (String, bool) {
    processes
        .lock()
        .ok()
        .map(|map| {
            map.get(task_id)
                .map(|info| {
                    let mut file = String::new();

                    // Prefer reading from the temporary file (avoids Windows stdout GBK encoding garbled text issues)
                    if let Some(ref fp_file) = info.filepath_file {
                        if let Some(path) = read_filepath_from_file(fp_file) {
                            file = path;
                        }
                        // Clean up the temporary file
                        let _ = std::fs::remove_file(fp_file);
                    }

                    // Fallback: path parsed from stdout
                    if file.is_empty() {
                        file = info.output_files.last().cloned().unwrap_or_default();
                        // Resolve relative paths to absolute paths
                        if !file.is_empty() && !std::path::Path::new(&file).is_absolute() {
                            file = std::path::PathBuf::from(&info.download_dir)
                                .join(&file)
                                .to_string_lossy()
                                .to_string();
                        }
                    }

                    let has = !info.output_files.is_empty() || !file.is_empty();
                    (file, has)
                })
                .unwrap_or_default()
        })
        .unwrap_or_default()
}

// ========== Download control commands ==========

/// Pause a download task (suspend the subprocess)
#[tauri::command]
pub async fn pause_download(
    state: tauri::State<'_, DownloadState>,
    id: String,
) -> Result<(), String> {
    let processes = state.processes.lock().map_err(|e| e.to_string())?;
    let info = processes.get(&id).ok_or("err_task_not_found")?;
    process::suspend_process(info.pid)
}

/// Resume a download task (resume the subprocess)
#[tauri::command]
pub async fn resume_download(
    state: tauri::State<'_, DownloadState>,
    id: String,
) -> Result<(), String> {
    let processes = state.processes.lock().map_err(|e| e.to_string())?;
    let info = processes.get(&id).ok_or("err_task_not_found")?;
    process::resume_process(info.pid)
}

/// Cancel a download task and optionally delete already-downloaded files
#[tauri::command]
pub async fn cancel_download(
    state: tauri::State<'_, DownloadState>,
    id: String,
    delete_files: bool,
) -> Result<(), String> {
    let (pid, files) = {
        let mut processes = state.processes.lock().map_err(|e| e.to_string())?;
        let info = processes.get_mut(&id).ok_or("err_task_not_found")?;
        info.cancelled = true;
        (info.pid, info.output_files.clone())
    };

    process::kill_process(pid)?;

    if delete_files {
        for file in &files {
            let _ = std::fs::remove_file(file);
            let _ = std::fs::remove_file(format!("{}.part", file));
        }
    }

    Ok(())
}

// ========== File checking ==========

/// Check whether multiple files exist in batch
#[tauri::command]
pub fn check_files_exist(paths: Vec<String>) -> Vec<bool> {
    paths
        .iter()
        .map(|p| std::path::Path::new(p).exists())
        .collect()
}

/// Delete the specified file
#[tauri::command]
pub fn delete_file(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if p.exists() {
        std::fs::remove_file(p).map_err(|e| format!("err_delete_file:{}", e))?;
    }
    Ok(())
}
