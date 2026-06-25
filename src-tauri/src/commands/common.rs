//! Shared helper functions for the commands module

use crate::utils;
use serde_json::Value;
use std::time::Duration;
use tauri::AppHandle;

#[cfg(target_os = "windows")]
use super::CREATE_NO_WINDOW;

/// Default HTTP request timeout (5 minutes)
const HTTP_TIMEOUT: Duration = Duration::from_secs(300);

/// Append Cookie and proxy-related arguments to the argument list
pub fn append_cookie_proxy_args(
    args: &mut Vec<String>,
    cookie_file: Option<&str>,
    cookie_browser: Option<&str>,
    proxy: Option<&str>,
) {
    if let Some(cf) = cookie_file {
        if !cf.is_empty() {
            args.push("--cookies".to_string());
            args.push(cf.to_string());
        }
    }
    if let Some(browser) = cookie_browser {
        if !browser.is_empty() {
            args.push("--cookies-from-browser".to_string());
            args.push(browser.to_string());
        }
    }
    if let Some(p) = proxy {
        if !p.is_empty() {
            args.push("--proxy".to_string());
            args.push(p.to_string());
        }
    }
}

/// Build an HTTP client with optional proxy and timeout
pub fn build_http_client(proxy: Option<&str>) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder().timeout(HTTP_TIMEOUT);
    if let Some(p) = proxy {
        if !p.is_empty() {
            let reqwest_proxy =
                reqwest::Proxy::all(p).map_err(|e| format!("err_proxy_config:{}", e))?;
            builder = builder.proxy(reqwest_proxy);
        }
    }
    builder
        .build()
        .map_err(|e| format!("err_create_http_client:{}", e))
}

/// Run yt-dlp -J and parse the JSON output (used to fetch video info, thumbnail list, subtitle list, etc.)
pub async fn run_ytdlp_json(
    app: &AppHandle,
    url: &str,
    extra_args: &[&str],
    cookie_file: Option<&str>,
    cookie_browser: Option<&str>,
    proxy: Option<&str>,
) -> Result<Value, String> {
    let ytdlp_path = utils::get_ytdlp_path(app)?;
    if !ytdlp_path.exists() {
        return Err("err_ytdlp_not_installed".to_string());
    }

    let mut args = vec![
        "-J".to_string(),
        "--ignore-config".to_string(),
        "--color".to_string(),
        "never".to_string(),
        "--no-warnings".to_string(),
        // Fail fast on network errors: the default retries=10 with no socket timeout can stall for several minutes
        "--socket-timeout".to_string(),
        "15".to_string(),
        "--retries".to_string(),
        "3".to_string(),
        "--extractor-retries".to_string(),
        "2".to_string(),
    ];
    for a in extra_args {
        args.push(a.to_string());
    }
    args.extend(utils::build_js_runtime_args(app));
    args.extend(utils::build_plugin_args(app));
    args.extend(utils::build_youtube_extractor_args());
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

    // Prefer parsing JSON from stdout (yt-dlp may write warnings to stderr but still succeed)
    if let Some(json_str) = stdout
        .lines()
        .find(|line| line.trim_start().starts_with('{'))
    {
        return serde_json::from_str(json_str).map_err(|e| format!("err_parse_video_info:{}", e));
    }

    // No JSON found; extract ERROR lines from stderr as the error message
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(extract_ytdlp_error(&stderr))
}

/// Extract error information from yt-dlp's stderr output
pub fn extract_ytdlp_error(stderr: &str) -> String {
    let error_lines: Vec<&str> = stderr.lines().filter(|l| l.contains("ERROR:")).collect();
    if error_lines.is_empty() {
        stderr.trim().to_string()
    } else {
        error_lines.join("\n")
    }
}

/// Validate file path safety (prevent path traversal attacks)
/// Ensure the resolved path is within base_dir
pub fn validate_path_within(
    base_dir: &std::path::Path,
    relative_path: &str,
) -> Result<std::path::PathBuf, String> {
    let target = base_dir.join(relative_path);
    // Normalize the path, eliminating .. and other relative path components
    let canonical_base = base_dir
        .canonicalize()
        .map_err(|e| format!("err_resolve_path:{}", e))?;
    // For paths that may not exist, check their parent directory
    let target_for_check = if target.exists() {
        target
            .canonicalize()
            .map_err(|e| format!("err_resolve_path:{}", e))?
    } else {
        // If the target does not exist, check whether its parent directory is within the base directory
        let parent = target.parent().ok_or("err_invalid_path")?;
        if !parent.exists() {
            // If the parent directory also does not exist, at least verify there are no .. components in the path
            if relative_path.contains("..") {
                return Err("err_path_traversal".to_string());
            }
            return Ok(base_dir.join(relative_path));
        }
        let canonical_parent = parent
            .canonicalize()
            .map_err(|e| format!("err_resolve_path:{}", e))?;
        if !canonical_parent.starts_with(&canonical_base) {
            return Err("err_path_traversal".to_string());
        }
        return Ok(base_dir.join(relative_path));
    };
    if !target_for_check.starts_with(&canonical_base) {
        return Err("err_path_traversal".to_string());
    }
    Ok(target_for_check)
}
