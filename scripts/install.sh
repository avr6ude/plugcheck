#!/bin/sh
# Build PlugCheck and install it to /Applications, replacing any running copy.
set -e
cd "$(dirname "$0")/.."

npm run tauri build -- --bundles app

APP="src-tauri/target/release/bundle/macos/PlugCheck.app"
[ -d "$APP" ] || { echo "build produced no .app at $APP" >&2; exit 1; }

pkill -x plugcheck 2>/dev/null || true
sleep 1
rm -rf /Applications/plugcheck.app /Applications/PlugCheck.app # old lowercase name too
cp -R "$APP" /Applications/PlugCheck.app

open /Applications/PlugCheck.app
echo "installed /Applications/PlugCheck.app"
