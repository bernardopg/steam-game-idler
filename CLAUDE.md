# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commits

Never add `Co-Authored-By: Claude` or any Claude/Anthropic co-author trailer to commit messages.

The pre-commit hook blocks direct commits to `main`. Use `ALLOW_MAIN_COMMIT=1 git commit` to bypass when working in the SGI workspace context where the fork's branch policy does not apply.

## Commands

```bash
pnpm install --frozen-lockfile   # install dependencies
pnpm typecheck                   # TypeScript type check (tsc --noEmit)
pnpm lint                        # ESLint
pnpm build                       # production Next.js build
pnpm prettier                    # format all files
```

Tauri dev — run from the parent SGI workspace, not from this directory directly:
```bash
# from SGI/
./steam-game-idler/scripts/dev-linux.sh
```
The script handles binary validation, process cleanup, and env loading. Running `pnpm tauri dev` directly here will work but skips those guards.

Rust:
```bash
# from src-tauri/
cargo check
cargo test
```

## Architecture

### Stack
Next.js 16 + React 19 + TypeScript frontend, embedded in a Tauri v2 desktop shell with a Rust backend. The frontend and backend communicate exclusively through `tauri::invoke` — no HTTP server. All Steam operations are delegated to `SteamUtility.Cli` (a sibling binary in the SGI workspace), spawned as a child process by the Rust backend.

### Frontend layout

```
src/
├── features/      # one directory per app feature
│   ├── achievement-manager/
│   ├── achievement-unlocker/
│   ├── card-farming/
│   ├── customlists/
│   ├── gameslist/
│   ├── inventory-manager/
│   └── settings/
├── shared/        # cross-cutting code
│   ├── components/  # layouts, titlebar, pro-gate components
│   ├── hooks/       # notifications, sidebar, layout, titlebar
│   ├── providers/   # React context providers
│   ├── stores/      # Zustand state (see below)
│   ├── types/       # TypeScript interfaces (invoke.ts has all Tauri command types)
│   └── utils/       # tauri.ts wraps all invoke calls; handleXxx utilities
├── i18n/          # i18next setup + locales (src/i18n/locales/<lang>/)
├── pages/         # Next.js pages router (_app.tsx, index.tsx, health.tsx)
└── styles/        # global CSS
```

Each feature folder follows the same shape: `components/`, `hooks/`, `utils/`, `index.ts` barrel. Shared utilities are consumed from `src/shared/`.

### State management (Zustand)

`src/shared/stores/` holds the global stores:

| Store | Contents |
|---|---|
| `userStore` | active Steam user and session data |
| `stateStore` | app-wide UI state (active feature, modals) |
| `idleStore` | currently idling games |
| `navigationStore` | sidebar navigation state |
| `searchStore` | search query |
| `loaderStore` | loading flags |
| `updateStore` | auto-updater state |

### Tauri invoke layer

All `invoke` calls are wrapped in `src/shared/utils/tauri.ts`. TypeScript types for every command's inputs and outputs live in `src/shared/types/invoke.ts`. When adding a new Tauri command, register it in both files.

### Rust backend modules (`src-tauri/src/`)

| Module | Responsibility |
|---|---|
| `steam_utility` | Resolves `SteamUtility.Cli` path (`SGI_STEAM_UTILITY_PATH` env → `libs/` fallback) |
| `idling` | Spawns/kills idle processes; card farming loop; per-process temp dir isolation |
| `command_runner` | Applies `CREATE_NO_WINDOW` (Windows only) |
| `achievement_manager` | Unlock/lock/toggle achievements and stats via the CLI |
| `trading_cards` | Card data fetch and market price queries |
| `process_handler` | Child process monitoring and bulk kill |
| `crypto` | AES obfuscation of the Steam API key for production builds |
| `settings` | Read/write user settings file |
| `user_data`, `game_data` | Steam Web API data fetch and local cache |
| `custom_lists` | User-defined game lists persistence |
| `logging`, `automation`, `utils` | Log file management, anti-away mode, shared helpers |

### Environment variables

- `.env.dev` — dev only; must contain at least `KEY=""`. A real `STEAM_API_KEY` unlocks API-dependent features.
- `.env.prod` — release only; the key is AES-obfuscated. The build panics at startup without it.
- In debug builds the app falls back to `.env.dev` when no obfuscated key is stored.

### i18n

Translations live under `src/i18n/locales/<locale>/`. The active locale is detected via `i18next-browser-languagedetector`. Add new strings to `en-US` first; other locales are synced via Crowdin.

### Linux-specific constraints

- `next dev --webpack` is mandatory — Turbopack/HMR is unstable with WebKitGTK inside the Tauri WebView. The `dev` script in `package.json` already passes `--webpack`.
- Custom context menu (`Menu.popup()`) and native notifications are disabled in dev/Linux paths — guard these behind OS/dev checks before enabling.
- Opening `localhost:3000` in a regular browser will throw `invoke is not a function` — the Tauri API is only injected inside the WebView.
- Idle session concurrency cap: 8 on Linux, 32 on Windows (hard-coded in `idling.rs`).
- AppImage builds on Arch Linux require `NO_STRIP=1` — the `strip` binary bundled inside the linuxdeploy AppImage is too old to handle the `.relr.dyn` section in modern Arch libraries: `NO_STRIP=1 pnpm tauri build`.

## CI (`.github/workflows/ci.yml`)

Matrix build for Linux and Windows. Runs `cargo check` only — no `pnpm typecheck`. The full typecheck runs in the parent SGI workspace CI. Pinned versions: `actions/checkout@v6.0.2`, `swatinem/rust-cache@v2.9.1`.
