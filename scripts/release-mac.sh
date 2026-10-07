#!/usr/bin/env bash
# Build a Developer ID–signed, notarized plugcheck .dmg.
#
# Signing and notarization go through Xcode's signed-in account (Xcode →
# Settings → Accounts) with Apple's cloud-managed Developer ID certificate, so
# no local certificate, password or API key is needed. TEAM is the paid team.
set -euo pipefail
cd "$(dirname "$0")/.."
TEAM=${TEAM:-KTXT2JL8G5}
BUNDLE_ID=com.avrdude.plugcheck
VERSION=$(node -p 'require("./src-tauri/tauri.conf.json").version')
W=src-tauri/target/release/notarize

npm run tauri build -- --bundles app

# Xcode only distributes archives, so wrap the Tauri .app in one.
rm -rf "$W" && mkdir -p "$W/plugcheck.xcarchive/Products/Applications"
APP="$W/plugcheck.xcarchive/Products/Applications/plugcheck.app"
cp -R src-tauri/target/release/bundle/macos/plugcheck.app "$APP"
# Notarization requires the hardened runtime; Xcode keeps these flags when it re-signs.
codesign --force --deep --options runtime --identifier "$BUNDLE_ID" --sign - "$APP"
cat > "$W/plugcheck.xcarchive/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>ApplicationProperties</key><dict>
    <key>ApplicationPath</key><string>Applications/plugcheck.app</string>
    <key>Architectures</key><array><string>arm64</string></array>
    <key>CFBundleIdentifier</key><string>$BUNDLE_ID</string>
    <key>CFBundleShortVersionString</key><string>$VERSION</string>
    <key>CFBundleVersion</key><string>$VERSION</string>
    <key>SigningIdentity</key><string>-</string>
    <key>Team</key><string>$TEAM</string>
  </dict>
  <key>ArchiveVersion</key><integer>2</integer>
  <key>CreationDate</key><date>$(date -u +%Y-%m-%dT%H:%M:%SZ)</date>
  <key>Name</key><string>plugcheck</string>
  <key>SchemeName</key><string>plugcheck</string>
</dict></plist>
EOF
cat > "$W/ExportOptions.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>method</key><string>developer-id</string>
  <key>destination</key><string>upload</string>
  <key>teamID</key><string>$TEAM</string>
  <key>signingStyle</key><string>automatic</string>
</dict></plist>
EOF

# Sign with Developer ID and submit to Apple's notary service.
xcodebuild -exportArchive -archivePath "$W/plugcheck.xcarchive" -exportOptionsPlist "$W/ExportOptions.plist" \
  -exportPath "$W/export" -allowProvisioningUpdates

# Wait for the notarized, stapled app (usually a few minutes).
for _ in $(seq 1 90); do
  xcodebuild -exportNotarizedApp -archivePath "$W/plugcheck.xcarchive" -exportPath "$W/notarized" >/dev/null 2>&1 && break
  sleep 20
done
NOTARIZED="$W/notarized/plugcheck.app"
[ -d "$NOTARIZED" ] || { echo "Notarization did not finish; check Xcode Organizer." >&2; exit 1; }

# Fails loudly if Gatekeeper would still block it.
spctl --assess --type execute -v "$NOTARIZED"
xcrun stapler validate "$NOTARIZED"

DMG="src-tauri/target/release/bundle/dmg/plugcheck_${VERSION}_aarch64.dmg"
mkdir -p "$(dirname "$DMG")" && rm -f "$DMG"
STAGE=$(mktemp -d) && cp -R "$NOTARIZED" "$STAGE/" && ln -s /Applications "$STAGE/Applications"
hdiutil create -volname plugcheck -srcfolder "$STAGE" -ov -format UDZO "$DMG" >/dev/null
rm -rf "$STAGE"
echo "Ready: $DMG"
