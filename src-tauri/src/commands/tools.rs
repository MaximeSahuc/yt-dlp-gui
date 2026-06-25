//! Toolbox commands: thumbnail download, subtitle download, live chat/comments fetching

use crate::utils;
use serde_json::Value;
use tauri::AppHandle;

use super::common::{self, append_cookie_proxy_args, build_http_client, extract_ytdlp_error};

#[cfg(target_os = "windows")]
use super::CREATE_NO_WINDOW;

/// Generic tool command runner (--skip-download mode, does not download the video itself)
async fn run_ytdlp_tool(
    app: &AppHandle,
    url: &str,
    download_dir: &str,
    extra_args: Vec<String>,
    cookie_file: Option<&str>,
    cookie_browser: Option<&str>,
    proxy: Option<&str>,
) -> Result<String, String> {
    let ytdlp_path = utils::get_ytdlp_path(app)?;
    if !ytdlp_path.exists() {
        return Err("err_ytdlp_not_installed".to_string());
    }

    let mut args = vec![
        "--skip-download".to_string(),
        "--ignore-config".to_string(),
        "--color".to_string(),
        "never".to_string(),
        "--windows-filenames".to_string(),
        "--no-warnings".to_string(),
        "--socket-timeout".to_string(),
        "15".to_string(),
        "--retries".to_string(),
        "3".to_string(),
    ];
    args.extend(utils::build_js_runtime_args(app));
    args.extend(utils::build_plugin_args(app));
    args.extend(utils::build_youtube_extractor_args());

    let output_template = std::path::PathBuf::from(download_dir)
        .join("%(title).200s.%(ext)s")
        .to_string_lossy()
        .to_string();
    args.push("-o".to_string());
    args.push(output_template);

    args.extend(extra_args);
    append_cookie_proxy_args(&mut args, cookie_file, cookie_browser, proxy);
    args.push(url.to_string());

    let mut cmd = tokio::process::Command::new(&ytdlp_path);
    cmd.args(&args)
        .env("PYTHONUTF8", "1")
        .env("PYTHONIOENCODING", "utf-8");
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("err_run_ytdlp:{}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        Ok(stdout.to_string())
    } else {
        Err(extract_ytdlp_error(&stderr))
    }
}

/// Lightweight fetch of the video thumbnail list (skips format checks, faster)
#[tauri::command]
pub async fn tool_fetch_thumbnails(
    app: AppHandle,
    url: String,
    cookie_file: Option<String>,
    cookie_browser: Option<String>,
    proxy: Option<String>,
) -> Result<Value, String> {
    common::run_ytdlp_json(
        &app,
        &url,
        &["--no-check-formats", "--no-playlist"],
        cookie_file.as_deref(),
        cookie_browser.as_deref(),
        proxy.as_deref(),
    )
    .await
}

/// Download an image from the specified URL and save it to the specified file path
#[tauri::command]
pub async fn tool_save_thumbnail(
    url: String,
    file_path: String,
    proxy: Option<String>,
) -> Result<(), String> {
    let client = build_http_client(proxy.as_deref())?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("err_download_thumbnail:{}", e))?;

    if !response.status().is_success() {
        return Err(format!("err_download_thumbnail:HTTP {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("err_read_thumbnail_data:{}", e))?;

    if let Some(parent) = std::path::Path::new(&file_path).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("err_create_dir:{}", e))?;
    }

    tokio::fs::write(&file_path, &bytes)
        .await
        .map_err(|e| format!("err_save_file:{}", e))?;

    Ok(())
}

/// Download the video thumbnail
#[tauri::command]
pub async fn tool_download_thumbnail(
    app: AppHandle,
    url: String,
    download_dir: String,
    cookie_file: Option<String>,
    cookie_browser: Option<String>,
    proxy: Option<String>,
) -> Result<String, String> {
    run_ytdlp_tool(
        &app,
        &url,
        &download_dir,
        vec![
            "--write-thumbnail".to_string(),
            "--convert-thumbnails".to_string(),
            "jpg".to_string(),
        ],
        cookie_file.as_deref(),
        cookie_browser.as_deref(),
        proxy.as_deref(),
    )
    .await
}

/// Fetch video chapter information (chapters field)
#[tauri::command]
pub async fn tool_fetch_chapters(
    app: AppHandle,
    url: String,
    cookie_file: Option<String>,
    cookie_browser: Option<String>,
    proxy: Option<String>,
) -> Result<Value, String> {
    let info = common::run_ytdlp_json(
        &app,
        &url,
        &["--no-check-formats", "--no-playlist"],
        cookie_file.as_deref(),
        cookie_browser.as_deref(),
        proxy.as_deref(),
    )
    .await?;

    Ok(serde_json::json!({
        "title": info.get("title").cloned().unwrap_or(Value::Null),
        "duration": info.get("duration").cloned().unwrap_or(Value::Null),
        "chapters": info.get("chapters").cloned().unwrap_or(Value::Array(vec![])),
    }))
}

/// Fetch the list of available subtitles for a video (returns subtitles + automatic_captions)
/// Supports single videos and playlists: for playlist URLs, aggregates subtitles from all entries (for each language, uses the first entry that has it).
#[tauri::command]
pub async fn tool_fetch_subtitles(
    app: AppHandle,
    url: String,
    cookie_file: Option<String>,
    cookie_browser: Option<String>,
    proxy: Option<String>,
) -> Result<Value, String> {
    let info = common::run_ytdlp_json(
        &app,
        &url,
        &["--no-check-formats"],
        cookie_file.as_deref(),
        cookie_browser.as_deref(),
        proxy.as_deref(),
    )
    .await?;

    let is_playlist = info.get("_type").and_then(Value::as_str) == Some("playlist");
    if is_playlist {
        if let Some(entries) = info.get("entries").and_then(Value::as_array) {
            return Ok(serde_json::json!({
                "title": info.get("title").cloned().unwrap_or(Value::Null),
                "subtitles": aggregate_subtitle_map(entries, "subtitles"),
                "automatic_captions": aggregate_subtitle_map(entries, "automatic_captions"),
            }));
        }
    }

    // Single video: read directly from the root fields
    Ok(serde_json::json!({
        "title": info.get("title").cloned().unwrap_or(Value::Null),
        "subtitles": info.get("subtitles").cloned().unwrap_or(Value::Object(Default::default())),
        "automatic_captions": info.get("automatic_captions").cloned().unwrap_or(Value::Object(Default::default())),
    }))
}

/// Aggregate subtitles from all playlist entries into a union; for each language, use the tracks from the first entry that has them.
fn aggregate_subtitle_map(entries: &[Value], field: &str) -> Value {
    let mut merged = serde_json::Map::new();
    for entry in entries {
        let Some(map) = entry.get(field).and_then(Value::as_object) else {
            continue;
        };
        for (lang, tracks) in map {
            if !merged.contains_key(lang) {
                if let Some(arr) = tracks.as_array() {
                    if !arr.is_empty() {
                        merged.insert(lang.clone(), tracks.clone());
                    }
                }
            }
        }
    }
    Value::Object(merged)
}

/// Download a single subtitle file and save it as a new file
#[tauri::command]
pub async fn tool_save_subtitle(
    url: String,
    file_path: String,
    proxy: Option<String>,
) -> Result<(), String> {
    let client = build_http_client(proxy.as_deref())?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("err_download_subtitle:{}", e))?;

    if !response.status().is_success() {
        return Err(format!("err_download_subtitle:HTTP {}", response.status()));
    }

    let text = response
        .text()
        .await
        .map_err(|e| format!("err_read_subtitle_data:{}", e))?;

    if let Some(parent) = std::path::Path::new(&file_path).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("err_create_dir:{}", e))?;
    }

    tokio::fs::write(&file_path, &text)
        .await
        .map_err(|e| format!("err_save_file:{}", e))?;

    Ok(())
}

/// Download the text content from a URL and return it (used by the frontend to fetch subtitle text for merging)
#[tauri::command]
pub async fn tool_download_text(url: String, proxy: Option<String>) -> Result<String, String> {
    let client = build_http_client(proxy.as_deref())?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("err_download_failed:{}", e))?;

    if !response.status().is_success() {
        return Err(format!("err_download_failed:HTTP {}", response.status()));
    }

    response
        .text()
        .await
        .map_err(|e| format!("err_read_text:{}", e))
}

/// Save text content to the specified file path
#[tauri::command]
pub async fn tool_save_text_to_file(content: String, file_path: String) -> Result<(), String> {
    // Path safety check: prevent writing to critical system paths
    let path = std::path::Path::new(&file_path);
    if file_path.contains("..") {
        return Err("err_path_traversal".to_string());
    }

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("err_create_dir:{}", e))?;
    }

    tokio::fs::write(&file_path, &content)
        .await
        .map_err(|e| format!("err_save_file:{}", e))?;

    Ok(())
}

/// Download video subtitle files
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn tool_download_subtitles(
    app: AppHandle,
    url: String,
    download_dir: String,
    sub_langs: String,
    write_auto_subs: bool,
    cookie_file: Option<String>,
    cookie_browser: Option<String>,
    proxy: Option<String>,
) -> Result<String, String> {
    let mut extra = vec![
        "--write-subs".to_string(),
        "--sub-langs".to_string(),
        sub_langs,
    ];
    if write_auto_subs {
        extra.push("--write-auto-subs".to_string());
    }
    run_ytdlp_tool(
        &app,
        &url,
        &download_dir,
        extra,
        cookie_file.as_deref(),
        cookie_browser.as_deref(),
        proxy.as_deref(),
    )
    .await
}

/// Video comment
#[derive(serde::Serialize, Clone)]
pub struct VideoComment {
    pub id: String,
    /// Parent comment ID ("root" for top-level comments)
    pub parent: String,
    pub author: String,
    pub author_id: String,
    pub text: String,
    /// Unix timestamp (seconds)
    pub timestamp: i64,
    pub like_count: i64,
    pub is_favorited: bool,
    pub author_is_uploader: bool,
}

/// Comment sort order
fn comment_sort_value(sort: &str) -> &'static str {
    match sort {
        "top" => "top",
        _ => "new",
    }
}

/// Fetch video comments (YouTube only; other sites may not have a comments field)
#[tauri::command]
pub async fn tool_fetch_comments(
    app: AppHandle,
    url: String,
    max_comments: u32,
    sort: String,
    cookie_file: Option<String>,
    cookie_browser: Option<String>,
    proxy: Option<String>,
) -> Result<Value, String> {
    let max_str = max_comments.to_string();
    let extractor_arg = format!(
        "youtube:max_comments={};comment_sort={}",
        max_str,
        comment_sort_value(&sort)
    );

    let info = common::run_ytdlp_json(
        &app,
        &url,
        &[
            "--no-check-formats",
            "--no-playlist",
            "--write-comments",
            "--extractor-args",
            &extractor_arg,
        ],
        cookie_file.as_deref(),
        cookie_browser.as_deref(),
        proxy.as_deref(),
    )
    .await?;

    let comments_raw = info
        .get("comments")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let comments: Vec<VideoComment> = comments_raw
        .into_iter()
        .map(|c| VideoComment {
            id: c
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            parent: c
                .get("parent")
                .and_then(Value::as_str)
                .unwrap_or("root")
                .to_string(),
            author: c
                .get("author")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            author_id: c
                .get("author_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            text: c
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            timestamp: c.get("timestamp").and_then(Value::as_i64).unwrap_or(0),
            like_count: c.get("like_count").and_then(Value::as_i64).unwrap_or(0),
            is_favorited: c
                .get("is_favorited")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            author_is_uploader: c
                .get("author_is_uploader")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        })
        .collect();

    Ok(serde_json::json!({
        "title": info.get("title").cloned().unwrap_or(Value::Null),
        "comment_count": info.get("comment_count").cloned().unwrap_or(Value::Null),
        "comments": comments,
    }))
}
