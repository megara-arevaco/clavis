#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
if (( EUID == 0 )); then echo "Run as your desktop user, not root." >&2; exit 1; fi
BINARY="./clavis"
if [[ ! -x "$BINARY" ]]; then BINARY="target/release/clavis"; fi
if [[ ! -x "$BINARY" ]]; then BINARY="$(command -v clavis || true)"; fi
[[ -x "$BINARY" ]] || { echo "Install first: ./install.sh" >&2; exit 1; }
if [[ $# == 0 ]]; then
  echo "Usage: ./enable-keyboard.sh --all | /dev/input/PATH [MORE_PATHS...]" >&2
  echo "Available Ctrl-capable keyboards:" >&2
  "$BINARY" --list-keyboards
  exit 1
fi
if [[ "$1" == --all ]]; then
  mapfile -t PATHS < <("$BINARY" --list-keyboards | cut -f1)
else
  PATHS=("$@")
fi
(( ${#PATHS[@]} > 0 )) || { echo "No keyboards found." >&2; exit 1; }
command -v setfacl >/dev/null || { echo "Install ACL tools: sudo apt install acl" >&2; exit 1; }
DEVICES=(); NAMES=()
for path in "${PATHS[@]}"; do
  [[ -c "$path" ]] || { echo "Not an input device: $path" >&2; exit 1; }
  device="$(readlink -f -- "$path")"
  [[ "$device" == /dev/input/event* ]] || { echo "Expected an input event device." >&2; exit 1; }
  name="$(< "/sys/class/input/$(basename "$device")/device/name")"
  DEVICES+=("$device"); NAMES+=("$name")
  printf '  %s — %s\n' "$device" "$name"
done
USER_ID="$(id -u)"
echo "This grants your user read access to these keyboards until permissions reset."
echo "Any process running as your user can then read them, including password keystrokes."
echo "No keystrokes are saved by Clavis. No input group membership is added."
read -r -p "Allow keyboard access? [y/N] " answer
[[ "$answer" == y || "$answer" == Y ]] || exit 1
sudo setfacl -m "u:$USER_ID:r" "${DEVICES[@]}"
for device in "${DEVICES[@]}"; do
  if ! CLAVIS_KEYBOARD="$device" "$BINARY" --check-input; then
    echo "Validation failed. Permissions were granted; revoke with sudo setfacl -x u:$USER_ID '$device'." >&2
    exit 1
  fi
done
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/clavis"
mkdir -p "$CONFIG_DIR"
TEMP_CONFIG="$(mktemp "$CONFIG_DIR/keyboards.XXXXXX")"
trap 'rm -f "$TEMP_CONFIG"' EXIT
for i in "${!PATHS[@]}"; do printf '%s\t%s\n' "${PATHS[$i]}" "${NAMES[$i]}"; done > "$TEMP_CONFIG"
mv "$TEMP_CONFIG" "$CONFIG_DIR/keyboards"
echo "Saved keyboard selection. Restart Clavis. Reconnected keyboards are matched by name; ACLs may need renewing."
