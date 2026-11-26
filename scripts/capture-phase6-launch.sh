#!/usr/bin/env bash
# Run under an isolated Xvfb display. Captures startup without delaying the app.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
profile="$(mktemp -d /tmp/firescene-launch-profile.XXXXXX)"
export XDG_CONFIG_HOME="$profile/config" XDG_DATA_HOME="$profile/data" XDG_CACHE_HOME="$profile/cache"
mkdir -p "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_CACHE_HOME"
cd "$root/frontend"
npm run dev -- --port 1420 --strictPort >/tmp/firescene-launch-vite.log 2>&1 & vite=$!
app=''
cleanup() { test -z "$app" || kill "$app" 2>/dev/null || true; kill "$vite" 2>/dev/null || true; }
trap cleanup EXIT
sleep 2
ffmpeg -loglevel error -y -f x11grab -video_size 1920x1080 -framerate 30 -i "$DISPLAY" -t 5 -c:v libx264 -pix_fmt yuv420p /tmp/firescene-phase6-launch.mp4 & capture=$!
sleep .3
"$root/src-tauri/target/debug/firescene-gcs" >/tmp/firescene-launch-app.log 2>&1 & app=$!
wait "$capture"
