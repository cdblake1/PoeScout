#!/usr/bin/env bash
# Kill zombie PoeScout processes before relaunching
set -e
cd "$(dirname "$0")/.."

echo "[dev-restart] Killing poe-scout.exe..."
taskkill //F //IM poe-scout.exe 2>/dev/null || true

echo "[dev-restart] Killing Vite dev server (port 3000)..."
pid=$(netstat -ano 2>/dev/null | grep ':3000 ' | grep 'LISTENING' | awk '{print $5}' | head -1)
if [ -n "$pid" ]; then
  taskkill //F //PID "$pid" 2>/dev/null || true
fi

echo "[dev-restart] Launching tauri dev..."
# Run the tauri CLI from repo root (where src-tauri/ lives)
# using the binary installed in packages/ui
./packages/ui/node_modules/.bin/tauri dev
