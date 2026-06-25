# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

yt-dlp-gui is a desktop application for downloading videos via yt-dlp. Built with **Tauri 2** (Rust backend) + **Vue 3** (TypeScript frontend). The UI is in Chinese.

## Development Commands

```bash
pnpm install          # Install frontend dependencies
pnpm tauri dev        # Run the full app in development (starts Vite + Rust backend)
pnpm dev              # Run frontend only (Vite dev server on port 5688)
pnpm build            # Type-check and build frontend (vue-tsc + vite build)
pnpm tauri build      # Build production app bundle
```

Rust backend builds are handled by Tauri automatically during `pnpm tauri dev` / `pnpm tauri build`. To check Rust code independently:
```bash
cd src-tauri && cargo check
```

## Architecture

### Frontend (`src/`)
- **Vue 3 + TypeScript** with `<script setup>` SFCs
- **Naive UI** component library, auto-imported via `unplugin-vue-components` (NaiveUiResolver)
- **Auto-imports** configured in `vite.config.ts`: Vue, Vue Router, VueUse APIs, and Naive UI composables are available without explicit imports
- **Pinia** for state with `pinia-plugin-persistedstate` for localStorage persistence
- **Path alias**: `@` maps to `src/`
- **Pages**: Home (video search/download UI), Downloads, Settings
- **Tauri IPC**: Frontend calls Rust commands via `invoke()` from `@tauri-apps/api/core`

### Backend (`src-tauri/src/`)
- `lib.rs` — Tauri app builder, registers all commands and plugins
- `commands/` — Tauri command handlers,按功能域拆分:
  - `mod.rs` — shared types (DownloadState, DownloadParams, YtdlpStatus etc.)
  - `setup.rs` — platform info, yt-dlp/Deno installation management
  - `video.rs` — video info fetching (`-J`), cookie management
  - `download.rs` — download task control (start/pause/resume/cancel/check_files_exist)
- `parser.rs` — yt-dlp `--progress-template` JSON output parsing
- `process.rs` — OS-level process control (suspend/resume/kill via Win32 API or signals)
- `utils.rs` — Path helpers (yt-dlp, Deno, cookie paths in app data dir), platform-specific download URLs, JS runtime args builder
- Binaries (yt-dlp, Deno) are downloaded to the Tauri app data directory at runtime, not bundled
- Progress events emitted to frontend via `app.emit()` (e.g., `ytdlp-download-progress`, `deno-download-progress`)
- Download progress uses `--progress-template` (structured JSON) instead of parsing stdout text
- Final output file path retrieved via `--print-to-file after_move:filepath` to avoid Windows GBK encoding issues

### Frontend-Backend Communication
- Tauri commands are invoked from Vue via `invoke<T>("command_name", { args })`
- Real-time progress uses Tauri event system (`app.emit` on Rust side)
- Shared types in `src/types/index.ts` mirror Rust structs in `commands/mod.rs`

## Releases & CI (fork constraints)

This repo (`MaximeSahuc/yt-dlp-gui`) is a **fork** of `imsyy/yt-dlp-gui`. Two things follow from that:

- **Forks suppress automatic Actions triggers.** Pushing a `v*` tag will *not* auto-run `.github/workflows/release.yml` until the one-time "enable workflows" button is clicked in the repo's **Actions** tab on github.com. Until then, dispatch manually against the tag ref:
  ```bash
  git tag vX.Y.Z && git push origin vX.Y.Z
  gh workflow run Release --ref vX.Y.Z   # dispatch on the tag so releaseName = vX.Y.Z
  ```
  After clicking the enable button once, plain tag pushes will trigger releases normally.
- **Updater signing is fork-specific.** `tauri.conf.json` has `createUpdaterArtifacts: true`, so the release build *requires* the repo secrets `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — without them the build fails. The `updater.pubkey` and `updater.endpoints` point to this fork (`MaximeSahuc/yt-dlp-gui`), not upstream. If the signing key is lost, existing installs can no longer validate updates.

Release matrix (`release.yml`) is trimmed to **Windows x64** + **Debian/Ubuntu x64** (`.msi`/`.exe`, `.deb`/`.AppImage`); macOS and Windows-arm64 targets are commented out for easy re-enabling. Releases are created as **drafts** (`releaseDraft: true`) and must be published manually from the Releases page.

## Key Conventions

- Windows builds use `CREATE_NO_WINDOW` flag (0x08000000) on all subprocess spawns to hide console windows
- All yt-dlp commands set `PYTHONUTF8=1` environment variable and use `--ignore-config --color never`
- Deno is optional — used as JS runtime for yt-dlp when installed (`--js-runtimes` flag)
- Cookie support: text (Netscape format saved to file) or direct file path
