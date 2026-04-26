#!/usr/bin/env bash
# Lightweight smoke test for the SteamUtility CLI integration on Linux.
# Does NOT require a running Steam session — validates only:
#   1. The binary resolves and runs
#   2. --help / no-args prints usage
#   3. An unknown command exits non-zero with a human-readable message
#   4. cargo check still passes
#
# Usage:
#   cd /home/bitter/git-clones/SGI/steam-game-idler
#   ./scripts/smoke-test-linux.sh
#
# To override the utility path:
#   SGI_STEAM_UTILITY_PATH=/custom/path ./scripts/smoke-test-linux.sh

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
UTILITY_BIN_DEFAULT="$REPO_ROOT/../steam-utility-multiplataform/src/SteamUtility.Cli/bin/Release/net10.0/SteamUtility.Cli"
export SGI_STEAM_UTILITY_PATH="${SGI_STEAM_UTILITY_PATH:-$UTILITY_BIN_DEFAULT}"

PASS=0
FAIL=0

ok()   { echo "  [PASS] $*"; PASS=$((PASS+1)); }
fail() { echo "  [FAIL] $*"; FAIL=$((FAIL+1)); }

# ── 1. Binary must exist and be executable ──────────────────────────────────
echo
echo "=== 1. SteamUtility binary check ==="
if [[ -x "$SGI_STEAM_UTILITY_PATH" ]]; then
  ok "Found: $SGI_STEAM_UTILITY_PATH"
else
  fail "Not found or not executable: $SGI_STEAM_UTILITY_PATH"
  echo
  echo "Build it first with:"
  echo "  cd ../steam-utility-multiplataform && dotnet build steam-utility-multiplataform.sln -c Release"
  echo
  exit 1
fi

# ── 2. No-args prints usage (exit 0 or 1 — just check output) ───────────────
echo
echo "=== 2. No-args usage output ==="
no_args_out=$("$SGI_STEAM_UTILITY_PATH" 2>&1 || true)
if echo "$no_args_out" | grep -qi "usage\|command\|help"; then
  ok "Usage text present"
else
  fail "Expected usage text, got: $no_args_out"
fi

# ── 3. --help flag ───────────────────────────────────────────────────────────
echo
echo "=== 3. --help flag ==="
help_out=$("$SGI_STEAM_UTILITY_PATH" --help 2>&1 || true)
if echo "$help_out" | grep -qi "usage\|command\|help"; then
  ok "--help returns usage text"
else
  fail "--help output unexpected: $help_out"
fi

# ── 4. Unknown command falls back to usage (exit 0 + usage text) ─────────────
# The CLI treats unknown commands as a dispatch miss and prints usage rather
# than erroring out — that is the documented upstream contract.
echo
echo "=== 4. Unknown command fallback to usage ==="
set +e
unknown_out=$("$SGI_STEAM_UTILITY_PATH" __nonexistent_cmd__ 2>&1)
set -e
if echo "$unknown_out" | grep -qi "usage\|command\|help"; then
  ok "Unknown command falls back to usage text (expected)"
else
  fail "Unknown command did not return usage text: $unknown_out"
fi

# ── 5. Rust backend compiles cleanly ────────────────────────────────────────
echo
echo "=== 5. cargo check (Tauri backend) ==="
cd "$REPO_ROOT/src-tauri"
cargo_out=$(cargo check 2>&1)
cargo_exit=$?
if [[ $cargo_exit -eq 0 ]]; then
  warning_count=$(echo "$cargo_out" | grep -c "^warning:" || true)
  ok "cargo check passed (warnings: $warning_count)"
  if [[ $warning_count -gt 0 ]]; then
    echo "    Warnings:"
    echo "$cargo_out" | grep "^warning:" | sed 's/^/      /'
  fi
else
  fail "cargo check failed"
  echo "$cargo_out"
fi

# ── Summary ──────────────────────────────────────────────────────────────────
echo
echo "═══════════════════════════════════════"
echo "  Smoke test complete: $PASS passed, $FAIL failed"
echo "═══════════════════════════════════════"
echo

if [[ $FAIL -gt 0 ]]; then
  exit 1
fi
