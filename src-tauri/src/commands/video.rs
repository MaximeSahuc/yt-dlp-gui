//! Video info fetching and Cookie management

use crate::utils;
use serde_json::Value;
use tauri::AppHandle;

use super::common;

// ========== Cookie management ==========

/// Save Cookie text (Netscape format) to the app data directory
#[tauri::command]
pub async fn save_cookie_text(app: AppHandle, text: String) -> Result<String, String> {
    let cookie_path = utils::get_cookie_path(&app)?;
    tokio::fs::write(&cookie_path, text.as_bytes())
        .await
        .map_err(|e| format!("err_save_cookie:{}", e))?;
    Ok(cookie_path.to_string_lossy().to_string())
}

// ========== Video info ==========

/// Fetch video metadata using yt-dlp -J (title, format list, subtitles, etc.)
#[tauri::command]
pub async fn fetch_video_info(
    app: AppHandle,
    url: String,
    cookie_file: Option<String>,
    cookie_browser: Option<String>,
    proxy: Option<String>,
) -> Result<Value, String> {
    common::run_ytdlp_json(
        &app,
        &url,
        &[],
        cookie_file.as_deref(),
        cookie_browser.as_deref(),
        proxy.as_deref(),
    )
    .await
}
