# AGENTS.md

## Overview

Input Pilot — Windows input automation desktop app (keyboard/mouse macro dispatch to other windows).  
Tauri v2 app: Vue 3 + Quasar frontend, Rust backend using Win32 APIs.

## Architecture

| Layer | Path | Tech |
|-------|------|------|
| Frontend | `src/` | Vue 3 Composition API, Quasar 2, Pinia (persisted), vue-i18n, vue-router (hash mode) |
| Backend | `src-tauri/` | Rust, Tauri 2, `windows` crate (Win32), tokio, parking_lot |

Backend modules (`src-tauri/src/`):
- `commands.rs` — Tauri command handlers (IPC surface)
- `win32/` — Win32 dispatchers: SendInput, WindowMessage, Logitech; window finder
- `service/` — macro executor, parser, serialization, config loader, hotkey daemon
- `model/` — data types (AppConfig, MacroSequence, TargetSpec, DispatchMode)
- `driver/` — low-level driver interaction

Frontend state lives in `src/stores/` (Pinia with `pinia-plugin-persistedstate`, auto-persist, key prefix `ParticleG.input-pilot.<id>`).

## Commands

Package manager: **bun** (lockfile is `bun.lock`; `tauri.conf.json` uses `bun dev` / `bun build`).

```bash
# Frontend dev (Quasar SPA on localhost:9000)
bun dev

# Full Tauri dev (compiles Rust + launches app with frontend hot-reload)
bun tauri-dev

# Production build
bun tauri-build

# Lint (ESLint flat config, type-checked)
bun lint

# Format (oxfmt — NOT prettier)
bun format
```

No test suite exists yet (`"test"` script is a no-op).

## Tooling Quirks

- **Formatter**: oxfmt (config in `.oxfmtrc.json`): single quotes, 100 char print width. Do NOT use Prettier.
- **ESLint**: flat config (`eslint.config.js`), includes `@vue/eslint-config-typescript` with `recommendedTypeChecked`. Enforces `consistent-type-imports` (use `import type`).
- **ESLint ignores `src-tauri/`** — Rust code is not linted by ESLint.
- **TypeScript**: strict mode, config extends `.quasar/tsconfig.json` (generated, do not edit).
- **Quasar auto-imports**: components/directives are auto-imported; plugins `Notify` and `Dialog` are registered globally.
- **Window decorations disabled** — app uses custom titlebar (`decorations: false` in `tauri.conf.json`).
- **Close-to-tray** — window close is intercepted; app hides instead of quitting.

## Rust / Tauri Notes

- Rust edition 2021, MSRV 1.77.2.
- Heavy use of `windows` crate for Win32 FFI — features are explicitly listed in `Cargo.toml`.
- `AppState` is Tauri managed state with `Mutex<Option<MacroRepository>>` + `HotkeyDaemon`.
- All Tauri commands are registered in `lib.rs` via `generate_handler![]`.
- Capabilities defined in `src-tauri/capabilities/default.json`.

## Conventions

- Code, comments, and docs in English; UI text may be localized (i18n in `src/i18n/`).
- Use `type` imports in TypeScript (`import type { Foo } from ...`).
- Frontend follows Quasar CLI conventions: boot files in `src/boot/`, layouts/pages/components split.
