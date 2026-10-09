#!/usr/bin/env bash
set -euo pipefail
APP_NAME=clavis
case "${1:-}" in
  --user) BIN_DIR="$HOME/.local/bin"; DESKTOP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"; ROOT=() ;;
  '') PREFIX="${PREFIX:-/usr/local}"; BIN_DIR="$PREFIX/bin"; DESKTOP_DIR="$PREFIX/share/applications"; ROOT=(); if (( EUID != 0 )); then ROOT=(sudo); fi ;;
  *) echo "Usage: ./uninstall.sh [--user]" >&2; exit 1 ;;
esac
"${ROOT[@]}" rm -f "$BIN_DIR/$APP_NAME" "$DESKTOP_DIR/$APP_NAME.desktop" "$(dirname "$DESKTOP_DIR")/icons/hicolor/scalable/apps/$APP_NAME.svg" "$(dirname "$DESKTOP_DIR")/icons/hicolor/256x256/apps/$APP_NAME.png"
rm -f "${XDG_CONFIG_HOME:-$HOME/.config}/autostart/$APP_NAME.desktop"
echo "Uninstalled $APP_NAME. Stop any running instance separately."
