//! Tauri command module
//!
//! Split by functional domain:
//! - common: Shared helper functions (Cookie/proxy arguments, HTTP client, yt-dlp JSON execution)
//! - setup: Platform info, yt-dlp/Deno installation management
//! - video: Video info fetching, Cookie management
//! - download: Download task control
//! - tools: Toolbox commands (thumbnails, subtitles, comments)

pub(crate) mod common;
mod download;
mod setup;
mod tools;
mod video;

// Glob re-exports: the Tauri generate_handler! macro needs access to hidden __cmd__ items
pub use download::*;
pub use setup::*;
pub use tools::*;
pub use video::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ========== Shared types ==========

/// yt-dlp installation status
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YtdlpStatus {
    pub installed: bool,
    pub version: String,
    pub path: String,
    /// `true` if the currently active binary is the app-managed copy;
    /// `false` if using the system-installed version (in this case, "Check for updates" only updates the managed copy and has no effect on the active binary)
    pub is_managed: bool,
}

/// Deno installation status
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DenoStatus {
    pub installed: bool,
    pub version: String,
    pub path: String,
    pub is_managed: bool,
}

/// ffmpeg installation status
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegStatus {
    pub installed: bool,
    pub version: String,
    pub path: String,
    /// `true` if using the app-managed copy; `false` if using the ffmpeg found in the system PATH
    pub is_managed: bool,
}

/// Download process information (runtime state)
pub struct DownloadProcessInfo {
    /// Process PID
    pub pid: u32,
    /// Whether the task has been cancelled by the user
    pub cancelled: bool,
    /// Output file path parsed from stdout (used as a fallback)
    pub output_files: Vec<String>,
    /// Download directory
    pub download_dir: String,
    /// Temporary file path used to store the final output path written by --print-to-file
    pub filepath_file: Option<String>,
    /// Duration (in seconds) of the clipped segment, used to calculate ffmpeg processing progress
    pub clip_duration: Option<f64>,
}

/// Download state management (globally shared)
pub struct DownloadState {
    pub processes: Arc<Mutex<HashMap<String, DownloadProcessInfo>>>,
}

impl Default for DownloadState {
    fn default() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Download task parameters (passed from the frontend)
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadParams {
    pub id: String,
    pub url: String,
    pub download_dir: String,
    pub download_mode: String,
    pub video_format: Option<String>,
    pub audio_format: Option<String>,
    pub cookie_file: Option<String>,
    /// Browser name for reading cookies from browser
    pub cookie_browser: Option<String>,
    /// Proxy address
    pub proxy: Option<String>,
    /// Filename template
    pub output_template: Option<String>,
    /// Concurrent fragment count
    pub concurrent_fragments: Option<u32>,
    /// Do not overwrite existing files
    pub no_overwrites: bool,
    pub embed_subs: bool,
    pub embed_thumbnail: bool,
    pub embed_metadata: bool,
    /// Embed chapter markers
    pub embed_chapters: bool,
    /// Remove sponsor segments (SponsorBlock)
    pub sponsorblock_remove: bool,
    /// Extract audio mode (-x)
    pub extract_audio: bool,
    /// Audio conversion format (--audio-format)
    pub audio_convert_format: Option<String>,
    /// Audio quality/bitrate (--audio-quality)
    pub audio_quality: Option<String>,
    pub no_merge: bool,
    pub limit_rate: Option<String>,
    /// Custom FFmpeg post-processing arguments (--postprocessor-args)
    pub ffmpeg_args: Option<String>,
    pub subtitles: Vec<String>,
    pub start_time: Option<f64>,
    pub end_time: Option<f64>,
    pub no_playlist: bool,
    pub playlist_items: Option<String>,
}

// ========== Platform constants ==========

/// Windows: flag to hide the console window
#[cfg(target_os = "windows")]
pub(crate) const CREATE_NO_WINDOW: u32 = 0x08000000;
