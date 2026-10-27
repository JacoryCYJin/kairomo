#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ $# -gt 0 ]]; then
  case "$1" in --help|-h) [[ $# -eq 1 ]] || exit 2; echo 'Build a local macOS Beta app, DMG and signed updater files. No install or upload.'; exit 0;; *) echo 'Unknown argument' >&2; exit 2;; esac
fi
[[ "$(uname -s)" == Darwin ]] || { echo 'Beta macOS packaging requires macOS.' >&2; exit 1; }
SIGNING_KEY="${KAIROMO_UPDATER_KEY:-$HOME/.local/share/kairomo-release/updater.key}"
[[ -f "$SIGNING_KEY" ]] || { echo 'Updater signing key is missing. See docs/releases.md.' >&2; exit 1; }
cmp -s "$SIGNING_KEY.pub" "$ROOT/src-tauri/updater.pub" || { echo 'Signing key does not match the embedded public key.' >&2; exit 1; }
export TAURI_SIGNING_PRIVATE_KEY="$SIGNING_KEY"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="${TAURI_SIGNING_PRIVATE_KEY_PASSWORD:-}"
cd "$ROOT"
npm run tauri -- build --config src-tauri/tauri.beta.conf.json --bundles app,dmg
node scripts/prepare-beta.mjs
