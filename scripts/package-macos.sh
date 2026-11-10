#!/usr/bin/env bash
# Package RuneDesk for macOS: RuneDesk.app and a dmg in dist/.
#
# Build first:
#   python3 res/inline-sciter.py
#   cargo build --release --features no-clipboard,inline
#
# Optional environment:
#   SIGN_IDENTITY   codesign identity, e.g. "Developer ID Application: ..."
#   APPLE_ID, APPLE_TEAM_ID, APP_SPECIFIC_PASSWORD
#                   notarize and staple the dmg (needs SIGN_IDENTITY)
#   VERSION         defaults to the VERSION file
#   BIN             defaults to target/release/runedesk
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=${VERSION:-$(tr -d '[:space:]' < VERSION)}
BIN=${BIN:-target/release/runedesk}
ARCH=$(lipo -archs "$BIN" | tr ' ' '-' | sed 's/x86_64/x64/')
# Oldest macOS the build supports (match MACOSX_DEPLOYMENT_TARGET of the build).
MIN_MACOS=${MACOSX_DEPLOYMENT_TARGET:-11.0}
OUT=dist
APP=$OUT/RuneDesk.app
DMG=$OUT/RuneDesk-$VERSION-macos-$ARCH.dmg

rm -rf "$APP" "$DMG" "$OUT/dmg"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Frameworks" "$APP/Contents/Resources"

cp "$BIN" "$APP/Contents/MacOS/runedesk"
cp libsciter.dylib "$APP/Contents/Frameworks/libsciter.dylib"

# App icon from the 1024px master.
ICONSET=$(mktemp -d)/RuneDesk.iconset
mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
  sips -z $s $s res/icon.png --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
  sips -z $((s * 2)) $((s * 2)) res/icon.png --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/RuneDesk.icns"

cat > "$APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key><string>en</string>
	<key>CFBundleDisplayName</key><string>RuneDesk</string>
	<key>CFBundleExecutable</key><string>runedesk</string>
	<key>CFBundleIconFile</key><string>RuneDesk</string>
	<key>CFBundleIdentifier</key><string>com.runetale.runedesk</string>
	<key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
	<key>CFBundleName</key><string>RuneDesk</string>
	<key>CFBundlePackageType</key><string>APPL</string>
	<key>CFBundleShortVersionString</key><string>$VERSION</string>
	<key>CFBundleVersion</key><string>$VERSION</string>
	<key>LSApplicationCategoryType</key><string>public.app-category.utilities</string>
	<key>LSMinimumSystemVersion</key><string>$MIN_MACOS</string>
	<key>NSHighResolutionCapable</key><true/>
	<key>NSPrincipalClass</key><string>NSApplication</string>
	<key>NSMicrophoneUsageDescription</key><string>RuneDesk sends your audio to the remote device during a session.</string>
	<key>NSAppleEventsUsageDescription</key><string>RuneDesk installs and controls its background service.</string>
</dict>
</plist>
EOF

if [[ -n "${SIGN_IDENTITY:-}" ]]; then
  codesign --force --timestamp --options runtime -s "$SIGN_IDENTITY" \
    "$APP/Contents/Frameworks/libsciter.dylib"
  codesign --force --timestamp --options runtime -s "$SIGN_IDENTITY" \
    --entitlements res/macos/RuneDesk.entitlements "$APP"
  codesign --verify --deep --strict --verbose=2 "$APP"
fi

mkdir -p "$OUT/dmg"
cp -R "$APP" "$OUT/dmg/"
ln -s /Applications "$OUT/dmg/Applications"
hdiutil create -quiet -volname "RuneDesk" -srcfolder "$OUT/dmg" -ov -format UDZO "$DMG"
rm -rf "$OUT/dmg"

if [[ -n "${SIGN_IDENTITY:-}" ]]; then
  codesign --force --timestamp -s "$SIGN_IDENTITY" "$DMG"
  if [[ -n "${APPLE_ID:-}" && -n "${APPLE_TEAM_ID:-}" && -n "${APP_SPECIFIC_PASSWORD:-}" ]]; then
    xcrun notarytool submit "$DMG" --apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" \
      --password "$APP_SPECIFIC_PASSWORD" --wait --timeout 30m
    xcrun stapler staple "$DMG"
    xcrun stapler validate "$DMG"
  fi
fi

echo "$DMG"
