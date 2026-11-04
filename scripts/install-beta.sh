#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OLD_APP="/Applications/Kairomo Beta.app"
NEW_APP="$ROOT/target/release/bundle/macos/Kairomo Beta.app"
DATA_DIR="$HOME/Library/Application Support/com.jacorycyjin.kairomo"
REQUEST="$DATA_DIR/install-handoff-request.json"
TOKEN="$DATA_DIR/install-handoff-token.json"
RESULT="$DATA_DIR/install-handoff-result.json"
MARKER="Contents/Resources/install-handoff-v1"
HANDOFF_ACTIVE=0

trap 'if [[ "${HANDOFF_ACTIVE:-0}" == "1" ]]; then cleanup_handoff; fi' EXIT

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  echo 'Build, replace local Kairomo Beta, and resume the active timer when supported.'
  exit 0
fi
[[ $# -eq 0 ]] || { echo "Unknown argument" >&2; exit 2; }

plist() {
  /usr/libexec/PlistBuddy -c "Print :$2" "$1/Contents/Info.plist" 2>/dev/null
}

wait_for_exit() {
  local attempts="$1"
  local i
  for ((i = 0; i < attempts; i++)); do
    if ! pgrep -f "$OLD_APP/Contents/MacOS/kairomo-desktop" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.25
  done
  return 1
}

cleanup_handoff() {
  rm -f "$REQUEST" "$TOKEN" "$RESULT"
}

relaunch_old_without_resume() {
  if [[ -d "$OLD_APP" ]]; then
    open "$OLD_APP"
  fi
}

cleanup_handoff

PATH="$HOME/.cargo/bin:$PATH" bash "$ROOT/scripts/package-beta.sh"

[[ -d "$NEW_APP" ]] || { echo "Built Beta app is missing." >&2; exit 1; }
NEW_VERSION="$(plist "$NEW_APP" CFBundleShortVersionString)"
NEW_ID="$(plist "$NEW_APP" CFBundleIdentifier)"
[[ -n "$NEW_VERSION" && "$NEW_ID" == "com.jacorycyjin.kairomo" ]] || {
  echo "Built Beta metadata is invalid." >&2
  exit 1
}
codesign --verify --deep --strict "$NEW_APP" >/dev/null
[[ -f "$NEW_APP/$MARKER" ]] || {
  echo "Built Beta does not contain the handoff capability marker." >&2
  exit 1
}

OLD_HANDOFF=0
if [[ -f "$OLD_APP/$MARKER" ]]; then
  OLD_HANDOFF=1
fi

OLD_RUNNING=0
if pgrep -f "$OLD_APP/Contents/MacOS/kairomo-desktop" >/dev/null 2>&1; then
  OLD_RUNNING=1
fi

NONCE=""
TOKEN_READY=0
RESULT_OK=0
HANDOFF_REASON=""
if [[ "$OLD_RUNNING" == "1" && "$OLD_HANDOFF" == "1" ]]; then
  HANDOFF_ACTIVE=1
  NONCE="$(node -e 'process.stdout.write(crypto.randomUUID())')"
  CREATED_AT="$(node -e 'process.stdout.write(String(Date.now()))')"
  node -e '
    const fs = require("fs");
    const [path, nonce, version, createdAt] = process.argv.slice(1);
    fs.writeFileSync(path, JSON.stringify({
      protocol: 1,
      nonce,
      target_version: version,
      created_at_ms: Number(createdAt),
    }));
  ' "$REQUEST" "$NONCE" "$NEW_VERSION" "$CREATED_AT"

  osascript -e 'tell application "Kairomo Beta" to quit' >/dev/null 2>&1 || true
  if ! wait_for_exit 60; then
    cleanup_handoff
    echo "Kairomo Beta did not exit; replacement stopped without killing it." >&2
    exit 1
  fi

  TOKEN_READY="$(node -e '
    const fs = require("fs");
    try {
      const token = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
      const [nonce, version] = process.argv.slice(2);
      if (token.protocol === 1 && token.nonce === nonce && token.target_version === version &&
          typeof token.session_id === "string" && typeof token.checkpoint_hash === "string") {
        process.stdout.write("1");
      }
    } catch {}
  ' "$TOKEN" "$NONCE" "$NEW_VERSION")"
  if [[ "$TOKEN_READY" != "1" ]]; then
    RESULT_OK="$(node -e '
      const fs = require("fs");
      try {
        const result = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
        const nonce = process.argv[2];
        if (result.protocol === 1 && result.nonce === nonce && result.status === "not_resumable") {
          process.stdout.write("1");
        }
      } catch {}
    ' "$RESULT" "$NONCE")"
    if [[ "$RESULT_OK" != "1" ]]; then
      cleanup_handoff
      relaunch_old_without_resume
      echo "Beta supports handoff but did not produce a valid response; replacement stopped." >&2
      exit 1
    fi
    HANDOFF_REASON="$(node -e '
      const fs = require("fs");
      try {
        const result = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
        process.stdout.write(result.reason ?? "");
      } catch {}
    ' "$RESULT")"
  fi
elif [[ "$OLD_RUNNING" == "1" ]]; then
  osascript -e 'tell application "Kairomo Beta" to quit' >/dev/null 2>&1 || true
  if ! wait_for_exit 60; then
    echo "Kairomo Beta did not exit; replacement stopped without killing it." >&2
    exit 1
  fi
fi

BACKUP="/tmp/Kairomo Beta.previous.$(date +%s).app"
if [[ -d "$OLD_APP" ]]; then
  mv "$OLD_APP" "$BACKUP"
fi
if ! ditto "$NEW_APP" "$OLD_APP"; then
  if [[ -d "$BACKUP" ]]; then
    mv "$BACKUP" "$OLD_APP"
  fi
  cleanup_handoff
  relaunch_old_without_resume
  echo "Copy failed; old Beta was restored and relaunched without auto-resume." >&2
  exit 1
fi

if [[ "$TOKEN_READY" == "1" ]]; then
  open -a "$OLD_APP" --args "--install-handoff=$NONCE"
  HANDOFF_ACTIVE=0
else
  open "$OLD_APP"
fi

if [[ "$TOKEN_READY" == "1" ]]; then
  for ((i = 0; i < 80; i++)); do
    [[ -f "$RESULT" ]] && break
    sleep 0.25
  done
  STATUS="$(node -e '
    const fs = require("fs");
    try {
      const result = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
      process.stdout.write(`${result.status}:${result.reason ?? ""}`);
    } catch {}
  ' "$RESULT")"
  cleanup_handoff
  case "$STATUS" in
    resumed:*) echo "Beta replaced; the same running session was resumed." ;;
    not_resumed:*) echo "Beta replaced; session was not resumed (${STATUS#not_resumed:}). Continue manually." ;;
    *) echo "Beta replaced and launched; automatic resume was not confirmed." ;;
  esac
elif [[ "$RESULT_OK" == "1" ]]; then
  cleanup_handoff
  echo "Beta replaced; session was not resumable (${HANDOFF_REASON}). Continue manually."
elif [[ "$OLD_RUNNING" == "1" ]]; then
  echo "Beta replaced and launched; this legacy install cannot auto-resume. Continue manually once."
else
  echo "Beta replaced and launched."
fi

[[ -n "$BACKUP" && -d "$BACKUP" ]] && echo "Previous app backup: $BACKUP"
