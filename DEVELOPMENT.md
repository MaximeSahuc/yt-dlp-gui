# Development guide

This document covers everything you need to build, run, and contribute to **MP3 Buddy**.
For what the app does and how to install it, see the [main README](./README.md).

## Tech stack

| Layer | Technology |
|-------|-----------|
| App shell | [Tauri 2](https://tauri.app/) (Rust backend, native webview) |
| Backend | [Rust](https://www.rust-lang.org/) |
| Frontend | [Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/) (`<script setup>` SFCs) |
| UI library | [Naive UI](https://www.naiveui.com/) (auto-imported) |
| State | [Pinia](https://pinia.vuejs.org/) with localStorage persistence |
| Build tool | [Vite](https://vitejs.dev/) |
| i18n | [Vue I18n](https://vue-i18n.intlify.dev/) - 12 languages |
| Download engine | [yt-dlp](https://github.com/yt-dlp/yt-dlp) (downloaded at runtime, not bundled) |
| Optional JS runtime | [Deno](https://deno.com/) (used by yt-dlp for full YouTube support) |

## Prerequisites

- [Node.js](https://nodejs.org/) **>= 22**
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- Platform build dependencies for Tauri - see the
  [Tauri prerequisites guide](https://tauri.app/start/prerequisites/)

## Quick start

```bash
# 1. Clone
git clone https://github.com/MaximeSahuc/mp3-buddy.git
cd mp3-buddy

# 2. Install frontend dependencies
pnpm install

# 3. Run the full app in development (Vite + Rust backend, hot reload)
pnpm tauri:dev
```

That's it - a development build of the app window opens. The yt-dlp and Deno binaries are
downloaded into the app's data directory on first use, so nothing extra to set up.

## Commands

| Command | What it does |
|---------|--------------|
| `pnpm install` | Install frontend dependencies |
| `pnpm tauri:dev` | Run the full app in development (Vite + Rust backend) |
| `pnpm dev` | Run the **frontend only** (Vite dev server on port 5688) |
| `pnpm build` | Type-check (`vue-tsc`) and build the frontend |
| `pnpm tauri:build` | Build the production desktop app bundle |
| `pnpm typecheck` | Type-check without building |
| `pnpm lint` / `pnpm lint:fix` | Lint `src/` (auto-fix variant) |
| `pnpm format` | Format `src/` with Prettier |

To check the Rust backend on its own:

```bash
cd src-tauri && cargo check
```

## Project structure

```
mp3-buddy/
├── src/                  # Vue 3 frontend
│   ├── pages/            # Home, Downloads, Settings, Toolbox (+ toolbox tools)
│   ├── locales/          # i18n translation files (en-US, fr-FR, …)
│   └── types/            # TypeScript types mirroring the Rust structs
├── src-tauri/            # Rust backend (Tauri)
│   └── src/
│       ├── lib.rs        # App builder, registers commands & plugins
│       ├── commands/     # Tauri command handlers (setup, video, download)
│       ├── parser.rs     # Parses yt-dlp progress JSON
│       ├── process.rs    # OS-level process control (pause/resume/kill)
│       └── utils.rs      # Paths, download URLs, runtime args
├── browser-extension/    # Companion browser extension
├── screenshot/           # Images used in the README
└── public/               # Static assets (app icon, etc.)
```

### How the frontend and backend talk

- The frontend calls Rust functions with `invoke<T>("command_name", { args })` from
  `@tauri-apps/api/core`.
- Real-time download progress is pushed from Rust to the UI via Tauri's event system
  (`app.emit(...)`), e.g. `ytdlp-download-progress`.
- Shared data shapes live in `src/types/index.ts` and mirror the Rust structs in
  `src-tauri/src/commands/mod.rs` - keep the two in sync when you change either.

## Notes & conventions

- All yt-dlp invocations run with `PYTHONUTF8=1` and `--ignore-config --color never`.
- On Windows, subprocesses are spawned with the `CREATE_NO_WINDOW` flag to hide console windows.
- Deno is optional; when installed it's passed to yt-dlp via `--js-runtimes`.
- Cookies are supported either as Netscape-format text (saved to a file) or as a direct file path.

## Contributing

Contributions are welcome. Open an
[issue](https://github.com/MaximeSahuc/mp3-buddy/issues) or submit a pull request.
Before pushing, run `pnpm lint` and `pnpm typecheck` to keep the build green.
</content>
