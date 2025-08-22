#!/usr/bin/env bash
set -e

# Usage: ./create_app.sh <AppName> <BinaryPath>
APP_NAME="$1"
BINARY_PATH="$2"

if [[ -z "$APP_NAME" || -z "$BINARY_PATH" ]]; then
    echo "Usage: $0 <AppName> <BinaryPath>"
    exit 1
fi

# Extract version from Cargo.toml
VERSION=$(grep -E '^\s*version\s*=' Cargo.toml | head -n1 | sed -E 's/.*"([^"]+)".*/\1/')
if [[ -z "$VERSION" ]]; then
    echo "Could not find version in Cargo.toml"
    exit 1
fi

# Use timestamp for CFBundleVersion
BUNDLE_VERSION=$(date +%Y%m%d.%H%M%S)

# Create folder structure
APP_DIR="dist/${APP_NAME}.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"

mkdir -p "$MACOS_DIR"

# Create Info.plist
cat > "$CONTENTS_DIR/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8" ?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>English</string>
  <key>CFBundleDisplayName</key>
  <string>${APP_NAME}</string>
  <key>CFBundleExecutable</key>
  <string>runtime</string>
  <key>CFBundleIdentifier</key>
  <string>com.example.${APP_NAME}</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>${APP_NAME}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>${VERSION}</string>
  <key>CFBundleVersion</key>
  <string>${BUNDLE_VERSION}</string>
  <key>CSResourcesFileMapped</key>
  <true/>
  <key>LSRequiresCarbon</key>
  <true/>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
EOF

# Copy binary
cp "$BINARY_PATH" "$MACOS_DIR/runtime"
chmod +x "$MACOS_DIR/runtime"

echo "Created $APP_DIR with binary $BINARY_PATH and version $VERSION ($BUNDLE_VERSION)"
