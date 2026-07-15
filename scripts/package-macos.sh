#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"

if [ "$(uname -m)" != "arm64" ]; then
    echo "Steamy v0.1 only packages Apple Silicon builds" >&2
    exit 1
fi

VERSION=$(awk -F '"' '/^version = / { print $2; exit }' Cargo.toml)
APP="$ROOT/dist/Steamy.app"
ZIP="$ROOT/dist/Steamy-$VERSION-macos-arm64.zip"

if [ -n "${GITHUB_REF_NAME:-}" ] && [ "$GITHUB_REF_NAME" != "v$VERSION" ]; then
    echo "tag $GITHUB_REF_NAME does not match Cargo version $VERSION" >&2
    exit 1
fi

cargo build --release --locked

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"
install -m 755 target/release/steamy "$APP/Contents/MacOS/steamy"
sed "s/@VERSION@/$VERSION/g" resources/Info.plist > "$APP/Contents/Info.plist"

plutil -lint "$APP/Contents/Info.plist"
file "$APP/Contents/MacOS/steamy" | grep -q 'arm64'
codesign --force --options runtime --sign - "$APP"
codesign --verify --deep --strict "$APP"

ditto -c -k --norsrc --noextattr --keepParent "$APP" "$ZIP"
unzip -tq "$ZIP"

echo "$ZIP"
