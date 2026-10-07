#!/usr/bin/env bash
# Build a signed + notarized plugcheck.dmg.
#
# One-time setup:
#   1. A "Developer ID Application" certificate in the login keychain
#      (Xcode → Settings → Accounts → your team → Manage Certificates → + ).
#   2. Notary credentials stored in the keychain under the profile "plugcheck":
#        xcrun notarytool store-credentials plugcheck --apple-id <apple-id> --team-id <TEAMID>
#      (it prompts for an app-specific password from account.apple.com).
set -euo pipefail
cd "$(dirname "$0")/.."

IDENTITY=$(security find-identity -v -p codesigning | grep -o '"Developer ID Application: [^"]*"' | head -1 | tr -d '"' || true)
if [ -z "$IDENTITY" ]; then
  echo "No 'Developer ID Application' certificate in the keychain (see setup above)." >&2
  exit 1
fi
echo "Signing as: $IDENTITY"

# Tauri signs the .app (hardened runtime) when this is set.
APPLE_SIGNING_IDENTITY="$IDENTITY" npm run tauri build -- --bundles app,dmg

APP=src-tauri/target/release/bundle/macos/plugcheck.app
DMG=$(ls src-tauri/target/release/bundle/dmg/plugcheck_*.dmg)

codesign --sign "$IDENTITY" --timestamp "$DMG"
xcrun notarytool submit "$DMG" --keychain-profile plugcheck --wait
xcrun stapler staple "$DMG"

# Fails loudly if Gatekeeper would still block it.
codesign --verify --deep --strict "$APP"
spctl --assess --type open --context context:primary-signature -v "$DMG"
echo "Ready: $DMG"
