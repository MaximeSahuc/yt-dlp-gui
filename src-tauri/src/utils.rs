use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};
use tauri::{AppHandle, Manager};

// ========== YouTube extractor args (po_token / visitor_data) ==========

#[derive(Default, Clone)]
struct YoutubeExtractorArgs {
    po_token: String,
    visitor_data: String,
}

static YOUTUBE_EXTRACTOR_ARGS: OnceLock<RwLock<YoutubeExtractorArgs>> = OnceLock::new();

fn youtube_args_lock() -> &'static RwLock<YoutubeExtractorArgs> {
    YOUTUBE_EXTRACTOR_ARGS.get_or_init(|| RwLock::new(YoutubeExtractorArgs::default()))
}

/// Set YouTube PO Token / visitor_data; an empty string clears the value.
/// Used to bypass YouTube 403 / rate limiting (see yt-dlp wiki: Extractors > YouTube).
pub fn set_youtube_extractor_args(po_token: &str, visitor_data: &str) -> Result<(), String> {
    let mut guard = youtube_args_lock()
        .write()
        .map_err(|e| format!("err_set_youtube_args:{}", e))?;
    guard.po_token = po_token.trim().to_string();
    guard.visitor_data = visitor_data.trim().to_string();
    Ok(())
}

/// Build yt-dlp `--extractor-args` from the current PO Token / visitor_data;
/// Returns an empty vec if both values are empty (no arguments appended).
pub fn build_youtube_extractor_args() -> Vec<String> {
    let guard = match youtube_args_lock().read() {
        Ok(g) => g,
        Err(_) => return vec![],
    };
    let mut parts: Vec<String> = Vec::new();
    if !guard.po_token.is_empty() {
        parts.push(format!("po_token={}", guard.po_token));
    }
    if !guard.visitor_data.is_empty() {
        parts.push(format!("visitor_data={}", guard.visitor_data));
    }
    if parts.is_empty() {
        return vec![];
    }
    vec![
        "--extractor-args".to_string(),
        format!("youtube:{}", parts.join(";")),
    ]
}

/// Build the executable path within the app data directory
fn get_managed_executable_path(app: &AppHandle, file_name: &str) -> Result<PathBuf, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("err_app_data_dir:{}", e))?;
    std::fs::create_dir_all(&app_data).map_err(|e| format!("err_create_dir:{}", e))?;
    Ok(app_data.join(file_name))
}

/// Get the app-managed yt-dlp path (app data directory)
pub fn get_managed_ytdlp_path(app: &AppHandle) -> Result<PathBuf, String> {
    if cfg!(target_os = "windows") {
        get_managed_executable_path(app, "yt-dlp.exe")
    } else {
        get_managed_executable_path(app, "yt-dlp")
    }
}

/// Get the yt-dlp executable path (always uses the app-managed copy in the app data directory)
pub fn get_ytdlp_path(app: &AppHandle) -> Result<PathBuf, String> {
    get_managed_ytdlp_path(app)
}

/// Get the app-managed ffmpeg path (app data directory)
pub fn get_managed_ffmpeg_path(app: &AppHandle) -> Result<PathBuf, String> {
    if cfg!(target_os = "windows") {
        get_managed_executable_path(app, "ffmpeg.exe")
    } else {
        get_managed_executable_path(app, "ffmpeg")
    }
}

/// If an app-managed ffmpeg exists, return the yt-dlp `--ffmpeg-location` argument.
/// Points to the ffmpeg executable itself; yt-dlp will look for ffprobe in the same directory.
/// Returns an empty vec when no app-managed copy is installed, letting yt-dlp fall back to the system PATH ffmpeg.
pub fn build_ffmpeg_location_args(app: &AppHandle) -> Vec<String> {
    if let Ok(path) = get_managed_ffmpeg_path(app) {
        if path.exists() {
            return vec![
                "--ffmpeg-location".to_string(),
                path.to_string_lossy().to_string(),
            ];
        }
    }
    vec![]
}

/// Get the ffmpeg download URL. In-app download is only available on Windows (BtbN zip build);
/// Other platforms return None; users should install ffmpeg via the system package manager.
pub fn get_ffmpeg_download_url() -> Option<&'static str> {
    if cfg!(target_os = "windows") {
        Some("https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip")
    } else {
        None
    }
}

/// Get the app-managed Deno path (app data directory)
pub fn get_managed_deno_path(app: &AppHandle) -> Result<PathBuf, String> {
    if cfg!(target_os = "windows") {
        get_managed_executable_path(app, "deno.exe")
    } else {
        get_managed_executable_path(app, "deno")
    }
}

/// Get the Deno executable path (always uses the app-managed copy in the app data directory)
pub fn get_deno_path(app: &AppHandle) -> Result<PathBuf, String> {
    get_managed_deno_path(app)
}

/// Get the Cookie file path (stored in the app data directory)
pub fn get_cookie_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("err_app_data_dir:{}", e))?;
    Ok(app_data.join("cookies.txt"))
}

/// Get the yt-dlp download URL (per platform)
pub fn get_ytdlp_download_url() -> &'static str {
    if cfg!(target_os = "windows") {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
    } else if cfg!(target_os = "macos") {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
    } else {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux"
    }
}

/// Get the yt-dlp plugin directory path
pub fn get_plugin_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("err_app_data_dir:{}", e))?;
    Ok(app_data.join("yt-dlp-plugins"))
}

/// If the plugin directory exists, return the --plugin-dirs argument
pub fn build_plugin_args(app: &AppHandle) -> Vec<String> {
    if let Ok(plugin_dir) = get_plugin_dir(app) {
        if plugin_dir.exists() {
            return vec![
                "--plugin-dirs".to_string(),
                plugin_dir.to_string_lossy().to_string(),
            ];
        }
    }
    vec![]
}

/// If Deno is installed, return JS runtime arguments
pub fn build_js_runtime_args(app: &AppHandle) -> Vec<String> {
    if let Ok(deno_path) = get_deno_path(app) {
        if deno_path.exists() {
            return vec![
                "--js-runtimes".to_string(),
                format!("deno:{}", deno_path.to_string_lossy()),
            ];
        }
    }
    vec![]
}

/// Get the Deno download URL (per platform and architecture)
pub fn get_deno_download_url() -> &'static str {
    if cfg!(target_os = "windows") {
        "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-pc-windows-msvc.zip"
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "https://github.com/denoland/deno/releases/latest/download/deno-aarch64-apple-darwin.zip"
        } else {
            "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-apple-darwin.zip"
        }
    } else {
        "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-unknown-linux-gnu.zip"
    }
}
