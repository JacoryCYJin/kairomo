#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE="$ROOT/.dev"
PIDFILE="$STATE/desktop.pid"
LOGFILE="$STATE/desktop.log"
export PATH="$HOME/.cargo/bin:$PATH"
usage() { cat <<'HELP'
Kairomo · local development
Usage: bash scripts/dev.sh <action>
  start       Start this desktop app in development mode (port 1420)
  stop        Stop only the development process started by this script
  restart     Stop, then start
  status      Show development process state
  logs        Last 120 log lines
  list        List this project's development target
  build       Build a macOS debug .app
  help        Show help
HELP
}
running() {
  [[ -f "$PIDFILE" ]] || return 1
  local pid command
  pid="$(cat "$PIDFILE")"
  [[ "$pid" =~ ^[0-9]+$ ]] || return 1
  kill -0 "$pid" 2>/dev/null || return 1
  command="$(ps -p "$pid" -o args=)"
  [[ "$command" == *"npm run tauri"*"dev"* ]]
}
stop_tree() { local parent="$1" child; while read -r child; do [[ -n "$child" ]] && stop_tree "$child"; done < <(pgrep -P "$parent" || true); kill -TERM "$parent" 2>/dev/null || true; }
start() {
  if running; then echo 'Kairomo development app is already running.'; return; fi
  if lsof -nP -iTCP:1420 -sTCP:LISTEN >/dev/null 2>&1; then echo 'Port 1420 is in use. Check the existing process before starting Kairomo.' >&2; return 1; fi
  if pgrep -x kairomo-desktop >/dev/null 2>&1; then echo 'Kairomo is already open. Quit the existing instance before development.' >&2; return 1; fi
  command -v cargo >/dev/null || { echo 'Rust is missing; see docs/development.md.' >&2; return 1; }
  mkdir -p "$STATE"
  cd "$ROOT"
  nohup npm run tauri -- dev >"$LOGFILE" 2>&1 &
  echo "$!" > "$PIDFILE"
  echo "Kairomo is starting. Logs: $LOGFILE"
}
stop() { if running; then stop_tree "$(cat "$PIDFILE")"; fi; rm -f "$PIDFILE"; echo 'Kairomo development process stopped.'; }
if [[ $# -gt 2 ]]; then usage >&2; exit 2; fi
if [[ $# -eq 2 ]]; then
  case "$1" in start|stop|restart|status|logs|list|build|help) ;; *) usage >&2; exit 2;; esac
  case "$2" in --help|-h) usage; exit 0;; *) usage >&2; exit 2;; esac
fi
case "${1:-help}" in
  help|-h|--help) usage;;
  list) echo 'desktop · Tauri + Vue · port 1420';;
  status) if running; then echo "desktop running (PID $(cat "$PIDFILE"))"; else echo 'desktop stopped'; fi;;
  logs) if [[ -f "$LOGFILE" ]]; then tail -n 120 "$LOGFILE"; else echo 'No development logs yet.'; fi;;
  start) start;;
  stop) stop;;
  restart) stop; start;;
  build) cd "$ROOT"; npm run tauri -- build --debug --bundles app;;
  *) usage >&2; exit 2;;
esac
