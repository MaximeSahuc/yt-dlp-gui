use tauri::path::BaseDirectory;
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_opener::OpenerExt;

mod commands;
mod parser;
mod process;
mod utils;

/// Reveal the bundled browser-extension folder in the OS file manager
/// and return the absolute path.
#[tauri::command]
fn reveal_browser_extension(app: tauri::AppHandle) -> Result<String, String> {
    let path = app
        .path()
        .resolve("browser-extension", BaseDirectory::Resource)
        .map_err(|e| e.to_string())?;
    let path_str = path.to_string_lossy().into_owned();
    app.opener()
        .open_path(path_str.clone(), None::<&str>)
        .map_err(|e| e.to_string())?;
    Ok(path_str)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            // forward deep-link URLs to the frontend
            for arg in &args {
                if arg.starts_with("mp3buddy://") {
                    let _ = app.emit("deep-link-url", arg.clone());
                }
            }
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .setup(|_app| {
            #[cfg(target_os = "linux")]
            if let Some(w) = _app.get_webview_window("main") {
                let _ = w.set_decorations(false);
            }

            Ok(())
        })
        .manage(commands::DownloadState::default())
        .invoke_handler(tauri::generate_handler![
            reveal_browser_extension,
            commands::get_platform,
            commands::set_youtube_extractor_args,
            commands::get_ytdlp_status,
            commands::download_ytdlp,
            commands::update_ytdlp,
            commands::get_deno_status,
            commands::download_deno,
            commands::get_ffmpeg_status,
            commands::download_ffmpeg,
            commands::check_plugin_installed,
            commands::install_plugin,
            commands::uninstall_plugin,
            commands::save_cookie_text,
            commands::fetch_video_info,
            commands::start_download,
            commands::pause_download,
            commands::resume_download,
            commands::cancel_download,
            commands::check_files_exist,
            commands::delete_file,
            commands::tool_download_thumbnail,
            commands::tool_fetch_thumbnails,
            commands::tool_save_thumbnail,
            commands::tool_download_subtitles,
            commands::tool_fetch_subtitles,
            commands::tool_save_subtitle,
            commands::tool_download_text,
            commands::tool_save_text_to_file,
            commands::tool_fetch_chapters,
            commands::tool_fetch_comments,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
