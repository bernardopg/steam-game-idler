# SGI <-> SteamUtility.Cli Contract

This document defines the command and output contract that the Tauri backend in
`steam-game-idler` depends on when spawning `SteamUtility.Cli`.

## Binary Resolution

SGI resolves the utility path in this order:

1. `SGI_STEAM_UTILITY_PATH`, when set and non-empty.
2. `libs/SteamUtility.exe` on Windows or `libs/SteamUtility.Cli` on Linux.
3. The executable directory itself.
4. The sibling SGI workspace build output:
   `../steam-utility-multiplataform/src/SteamUtility.Cli/bin/Release/net10.0/SteamUtility.Cli`
   or `SteamUtility.Cli.exe` on Windows.

Release bundles must ship the utility under `src-tauri/libs/` using the platform
filename above. Development can use the sibling workspace path.

## General Process Rules

- SGI treats stdout as the machine-readable channel.
- SGI includes stderr in parse-error diagnostics where possible.
- `SteamUtility.Cli` must not print non-JSON text to stdout for commands that SGI
  parses as JSON.
- Diagnostics and native Steam IPC logs should go to stderr.
- Commands may return JSON errors on stdout even when the process exit code is `0`.
- SGI currently checks payload fields, not exit codes, for legacy commands.

## Commands Used By SGI

### `check_ownership --json <output_path>`

Caller:

```text
SteamUtility.Cli check_ownership --json <cache>/<steam_id>/temp_owned_games.json
```

Stdout success payload:

```json
{
  "success": true,
  "totalChecked": 123,
  "ownedCount": 45,
  "outputPath": "/path/to/temp_owned_games.json",
  "error": null,
  "suggestion": null,
  "failureReason": null
}
```

Stdout error payload:

```json
{
  "success": false,
  "totalChecked": 0,
  "ownedCount": 0,
  "outputPath": "/path/to/temp_owned_games.json",
  "error": "Steam installation not found.",
  "suggestion": "Make sure Steam is installed for the current user.",
  "failureReason": "InstallPathNotFound"
}
```

Output file contract:

```json
{
  "games": [
    {
      "appid": 730,
      "name": "Counter-Strike 2"
    }
  ]
}
```

SGI requires `success === true`, then reads `outputPath` indirectly from the path
it supplied and expects `games` to be an array.

### `idle <app_id> <app_name>`

Caller:

```text
SteamUtility.Cli idle 730 "Counter-Strike 2"
```

SGI starts this as a long-running child process with:

- current directory set to a per-app temporary idler directory;
- `steam_appid.txt` written in that directory;
- `SteamAppId=<app_id>`;
- `SteamGameId=<app_id>`.

Initial stdout success payload:

```json
{
  "success": "Steam API initialized",
  "appId": 730,
  "appName": "Counter-Strike 2",
  "mode": "linux",
  "note": "This platform keeps the Steam API session alive until Ctrl+C."
}
```

Initial stdout error payload:

```json
{
  "error": "Steam API initialization failed",
  "failureReason": "SteamNotRunning"
}
```

SGI considers the process healthy when it remains alive after startup. Process
listing can be reconstructed from SGI's child-process table or from the process
command line/window title.

### `get_achievement_data <app_id> <cache_dir>`

Caller:

```text
SteamUtility.Cli get_achievement_data 730 <cache_dir>
```

Stdout success payload:

```json
{
  "success": "<cache_dir>/<steam_id>/achievement_data/730.json"
}
```

Stdout error payload:

```json
{
  "error": "Steam API initialization failed",
  "failureReason": "SteamNotRunning"
}
```

Output file contract:

```json
{
  "achievements": [
    {
      "id": "ACH_ID",
      "name": "Achievement name",
      "description": "Achievement description",
      "iconNormal": "https://...",
      "iconLocked": "https://...",
      "permission": 0,
      "hidden": false,
      "achieved": false,
      "percent": 12.3,
      "protected_achievement": false,
      "flags": "None"
    }
  ],
  "stats": [
    {
      "id": "STAT_ID",
      "name": "Stat name",
      "stat_type": "INT",
      "permission": 0,
      "value": 0,
      "increment_only": false,
      "protected_stat": false,
      "flags": "None"
    }
  ]
}
```

SGI reads `<cache_dir>/<steam_id>/achievement_data/<app_id>.json` after a success
payload and wraps the parsed file as `{ "achievement_data": ... }` for the
frontend.

### Achievement Mutation Commands

Callers:

```text
SteamUtility.Cli unlock_achievement <app_id> <achievement_id>
SteamUtility.Cli lock_achievement <app_id> <achievement_id>
SteamUtility.Cli toggle_achievement <app_id> <achievement_id>
SteamUtility.Cli unlock_all_achievements <app_id>
SteamUtility.Cli lock_all_achievements <app_id>
```

Stdout success payload:

```json
{
  "success": "Successfully unlocked achievement"
}
```

Stdout error payload:

```json
{
  "error": "Failed to get achievement data. The achievement might not exist"
}
```

SGI forwards stdout to the frontend as a string today. Frontend handlers expect a
JSON object with either `success` or `error` once parsed by the Tauri invoke
layer.

### Stat Mutation Commands

Callers:

```text
SteamUtility.Cli update_stats <app_id> <json_array>
SteamUtility.Cli reset_all_stats <app_id>
```

`update_stats` receives a JSON array string. Each item is expected to contain a
stat identifier and value using the `StatUpdate` shape accepted by
`SteamUtility.Cli`.

Stdout success payload:

```json
{
  "success": "Successfully updated all stats"
}
```

Stdout error payload:

```json
{
  "error": "Invalid stats format: <details>"
}
```

SGI forwards stdout to the frontend as a string today, matching the achievement
mutation command behavior.

## Known Follow-Up

The current SGI backend still has legacy parsing in a few places, especially
`get_achievement_data`, where stdout is checked with string containment. The
next integration hardening step is to parse stdout as JSON for every command and
fail closed when the payload is malformed.
