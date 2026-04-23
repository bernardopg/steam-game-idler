#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
UTILITY_BIN_DEFAULT="$REPO_ROOT/../steam-utility-multiplataform/src/SteamUtility.Cli/bin/Release/net10.0/SteamUtility.Cli"

export SGI_STEAM_UTILITY_PATH="${SGI_STEAM_UTILITY_PATH:-$UTILITY_BIN_DEFAULT}"

if [[ ! -x "$SGI_STEAM_UTILITY_PATH" ]]; then
  echo "SteamUtility binary not found or not executable:" >&2
  echo "  $SGI_STEAM_UTILITY_PATH" >&2
  echo >&2
  echo "Build it first with:" >&2
  echo "  cd ../steam-utility-multiplataform && dotnet build steam-utility-multiplataform.sln -c Release" >&2
  exit 1
fi

if [[ ! -f "$REPO_ROOT/.env.dev" ]]; then
  echo 'KEY=""' > "$REPO_ROOT/.env.dev"
  echo "Created placeholder .env.dev at $REPO_ROOT/.env.dev" >&2
fi

cd "$REPO_ROOT"
exec pnpm tauri dev
