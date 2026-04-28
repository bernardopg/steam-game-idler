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
  echo 'STEAM_API_KEY=""' > "$REPO_ROOT/.env.dev"
  echo "Created placeholder .env.dev at $REPO_ROOT/.env.dev" >&2
fi

pkill -TERM -f "$SGI_STEAM_UTILITY_PATH idle" 2>/dev/null || true
sleep 1
pkill -KILL -f "$SGI_STEAM_UTILITY_PATH idle" 2>/dev/null || true
rm -rf /tmp/steam-game-idler

# Avoid stale Turbopack chunks in the Tauri WebView after backend rebuilds
# or interrupted dev sessions.
rm -rf "$REPO_ROOT/.next/dev"

# Export env vars from .env.dev into the process so Rust can use them at runtime
# as fallback when no API key is configured in app Settings.
set -o allexport
# shellcheck disable=SC1090
source "$REPO_ROOT/.env.dev" 2>/dev/null || true
set +o allexport

cd "$REPO_ROOT"
exec pnpm tauri dev
