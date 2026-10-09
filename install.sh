#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
APP_NAME=clavis
case "${1:-}" in
  --user) BIN_DIR="$HOME/.local/bin"; DESKTOP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"; ROOT=() ;;
  '') PREFIX="${PREFIX:-/usr/local}"; BIN_DIR="$PREFIX/bin"; DESKTOP_DIR="$PREFIX/share/applications"; ROOT=(); if (( EUID != 0 )); then ROOT=(sudo); fi ;;
  *) echo "Usage: ./install.sh [--user]" >&2; exit 1 ;;
esac
BINARY="${INSTALL_BINARY:-}"
if [[ -z "$BINARY" && -x "./$APP_NAME" ]]; then BINARY="./$APP_NAME"; fi
if [[ -z "$BINARY" ]]; then
  if ! command -v cargo >/dev/null || ! command -v pkg-config >/dev/null; then
    echo "Missing Rust build tools. On Ubuntu/Debian run:" >&2
    echo "  sudo apt install cargo rustc pkg-config libgtk-4-dev libx11-dev libxi-dev libxtst-dev libdbus-1-dev" >&2
    exit 1
  fi
  if ! pkg-config --exists gtk4 x11 xi xtst dbus-1; then
    echo "Missing development libraries: sudo apt install libgtk-4-dev libx11-dev libxi-dev libxtst-dev libdbus-1-dev" >&2
    exit 1
  fi
  cargo test --locked
  cargo build --release --locked
  BINARY="target/release/$APP_NAME"
fi
"$BINARY" --version
ICON_DIR="$(dirname "$DESKTOP_DIR")/icons/hicolor/scalable/apps"
"${ROOT[@]}" install -d "$BIN_DIR" "$DESKTOP_DIR" "$ICON_DIR"
"${ROOT[@]}" install -m 0755 "$BINARY" "$BIN_DIR/$APP_NAME"
"${ROOT[@]}" install -m 0644 "packaging/$APP_NAME.svg" "$ICON_DIR/$APP_NAME.svg"
PNG_DIR="$(dirname "$DESKTOP_DIR")/icons/hicolor/256x256/apps"
"${ROOT[@]}" install -d "$PNG_DIR"
"${ROOT[@]}" install -m 0644 "packaging/$APP_NAME.png" "$PNG_DIR/$APP_NAME.png"
TMP_DESKTOP="$(mktemp)"
trap 'rm -f "$TMP_DESKTOP"' EXIT
# The absolute command also works when ~/.local/bin is not on the desktop PATH.
EXECUTABLE="$BIN_DIR/$APP_NAME"
[[ "$EXECUTABLE" != *'"'* && "$EXECUTABLE" != *'\\'* ]] || { echo "Unsupported install path" >&2; exit 1; }
while IFS= read -r line; do
  if [[ "$line" == Exec=* ]]; then printf 'Exec="%s" --show\n' "$EXECUTABLE"; else printf '%s\n' "$line"; fi
done < packaging/"$APP_NAME".desktop > "$TMP_DESKTOP"
"${ROOT[@]}" install -m 0644 "$TMP_DESKTOP" "$DESKTOP_DIR/$APP_NAME.desktop"
if (( EUID != 0 )); then
  AUTOSTART_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/autostart"
  mkdir -p "$AUTOSTART_DIR"
  # Autostart listens silently; the menu launcher opens the overlay.
  while IFS= read -r line; do
    if [[ "$line" == Exec=* ]]; then printf 'Exec="%s"\n' "$EXECUTABLE"; else printf '%s\n' "$line"; fi
  done < "$TMP_DESKTOP" > "$AUTOSTART_DIR/$APP_NAME.desktop"
fi
printf 'Installed %s\nRun: %s --show\n' "$APP_NAME" "$BIN_DIR/$APP_NAME"
