#!/bin/sh
# Build plugcheck and install it to /Applications, replacing any running copy.
set -e
cd "$(dirname "$0")/.."

npm run tauri build

APP="src-tauri/target/release/bundle/macos/plugcheck.app"
[ -d "$APP" ] || { echo "build produced no .app at $APP" >&2; exit 1; }

osascript -e 'quit app "plugcheck"' 2>/dev/null || true
sleep 1
rm -rf /Applications/plugcheck.app
cp -R "$APP" /Applications/plugcheck.app

open /Applications/plugcheck.app
echo "installed /Applications/plugcheck.app"
