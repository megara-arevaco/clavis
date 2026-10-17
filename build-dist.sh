#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
ROOT_DIR="$PWD"
APP=clavis
if [[ "${CLAVIS_SKIP_TESTS:-0}" != "1" ]]; then cargo test --locked; fi
cargo build --release --locked
VERSION="$(target/release/$APP --version | awk '{print $2}')"
ARCH="$(dpkg --print-architecture)"
NAME="$APP-$VERSION-linux-$ARCH"
mkdir -p dist
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
ARCHIVE="$WORK/$NAME"
mkdir -p "$ARCHIVE/packaging" "$ARCHIVE/scripts" "$ARCHIVE/docs/screenshots"
install -m 0755 "target/release/$APP" "$ARCHIVE/$APP"
install -m 0755 install.sh uninstall.sh enable-keyboard.sh "$ARCHIVE/"
install -m 0644 README.md RESEARCH.md "$ARCHIVE/"
find packaging -maxdepth 1 -type f -exec install -m 0644 {} "$ARCHIVE/packaging/" \;
install -m 0755 scripts/render-icon.sh "$ARCHIVE/scripts/"
find docs/screenshots -maxdepth 1 -type f -exec install -m 0644 {} "$ARCHIVE/docs/screenshots/" \;
if [[ -f LICENSE ]]; then install -m 0644 LICENSE "$ARCHIVE/"; fi
tar -C "$WORK" -czf "dist/$NAME.tar.gz" "$NAME"

# Calculate ABI-aware runtime dependencies from the actual ELF executable.
STAGE="$WORK/package"
mkdir -p "$STAGE/DEBIAN" "$STAGE/usr/bin" "$STAGE/usr/share/applications" \
  "$STAGE/usr/share/icons/hicolor/scalable/apps" "$STAGE/usr/share/doc/$APP" \
  "$STAGE/usr/lib/$APP" "$STAGE/etc/xdg/autostart" "$WORK/debian" \
  "$STAGE/usr/share/icons/hicolor/256x256/apps" \
  "$STAGE/usr/share/doc/$APP/packaging" "$STAGE/usr/share/doc/$APP/docs/screenshots"
install -m 0755 "target/release/$APP" "$STAGE/usr/bin/$APP"
install -m 0755 enable-keyboard.sh "$STAGE/usr/lib/$APP/enable-keyboard.sh"
install -m 0644 "packaging/$APP.svg" "$STAGE/usr/share/icons/hicolor/scalable/apps/"
install -m 0644 "packaging/$APP.png" "$STAGE/usr/share/icons/hicolor/256x256/apps/"
install -m 0644 "packaging/$APP.desktop" "$STAGE/usr/share/applications/"
# The login entry starts silently, unlike the menu launcher.
awk '/^Exec=/{print "Exec=/usr/bin/clavis"; next} {print}' \
  "packaging/$APP.desktop" > "$STAGE/etc/xdg/autostart/$APP.desktop"
install -m 0644 README.md RESEARCH.md "$STAGE/usr/share/doc/$APP/"
install -m 0644 "packaging/$APP.svg" "$STAGE/usr/share/doc/$APP/packaging/"
find docs/screenshots -maxdepth 1 -type f -exec install -m 0644 {} "$STAGE/usr/share/doc/$APP/docs/screenshots/" \;
if [[ -f LICENSE ]]; then install -m 0644 LICENSE "$STAGE/usr/share/doc/$APP/copyright"; fi
cat > "$WORK/debian/control" <<EOF
Source: $APP
Section: utils
Priority: optional
Maintainer: Local Build <local@localhost>

Package: $APP
Architecture: $ARCH
Description: Compact terminal reference
EOF
DEPENDENCIES="$(cd "$WORK" && dpkg-shlibdeps -O -e"$STAGE/usr/bin/$APP" | awk '/^shlibs:Depends=/{sub(/^shlibs:Depends=/, ""); print}')"
[[ -n "$DEPENDENCIES" ]] || { echo 'Cannot determine runtime dependencies' >&2; exit 1; }
cat > "$STAGE/DEBIAN/control" <<EOF
Package: $APP
Version: $VERSION
Architecture: $ARCH
Installed-Size: $(du -sk "$STAGE" | awk '{print $1}')
Section: utils
Priority: optional
Maintainer: Local Build <local@localhost>
Depends: $DEPENDENCIES, acl
Conflicts: ghostty-ee-overlay
Replaces: ghostty-ee-overlay
Recommends: gnome-shell-extension-appindicator
Description: Clavis compact terminal reference with a tray icon
 Keyboard and command references for Ghostty and ten terminal tools.
 No commands are automatically executed and no input access is granted.
EOF
printf '/etc/xdg/autostart/%s.desktop\n' "$APP" > "$STAGE/DEBIAN/conffiles"
(cd "$STAGE" && find usr etc -type f -print0 | sort -z | xargs -0 md5sum) > "$STAGE/DEBIAN/md5sums"
chmod -R go-w "$STAGE"
dpkg-deb --root-owner-group --build "$STAGE" "dist/${APP}_${VERSION}_${ARCH}.deb"
(cd dist && sha256sum "$NAME.tar.gz" "${APP}_${VERSION}_${ARCH}.deb" > SHA256SUMS)
printf 'Built artifacts in %s/dist\n' "$ROOT_DIR"
