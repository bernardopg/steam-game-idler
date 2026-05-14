Steam Game Idler expects the platform `SteamUtility` artifact in this directory
for release bundles:

- Windows: `SteamUtility.exe`
- Linux: `SteamUtility.Cli`

Local development may instead resolve the sibling SGI workspace build output or
an explicit `SGI_STEAM_UTILITY_PATH` override.

See `docs/STEAM_UTILITY_CONTRACT.md` for the command and JSON contract between
the Tauri backend and `SteamUtility.Cli`.
