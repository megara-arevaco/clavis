#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
command -v rsvg-convert >/dev/null || { echo 'Install librsvg2-bin first.' >&2; exit 1; }
command -v ffmpeg >/dev/null || { echo 'Install ffmpeg first.' >&2; exit 1; }
SOURCE=packaging/clavis.svg
rsvg-convert --width 256 --height 256 "$SOURCE" --output packaging/clavis.png
rsvg-convert --width 32 --height 32 "$SOURCE" --output packaging/clavis-tray.png
ffmpeg -loglevel error -i packaging/clavis-tray.png -frames:v 1 -threads 1 \
  -pix_fmt argb -f rawvideo -y packaging/clavis-tray.argb
[[ "$(wc -c < packaging/clavis-tray.argb)" -eq 4096 ]]
HASH="$(sha256sum "$SOURCE" | awk '{print $1}')"
for FILE in clavis.png clavis-tray.png clavis-tray.argb; do
  cat > "packaging/$FILE.json" <<EOF
{
  "source": "packaging/clavis.svg",
  "sourceSha256": "$HASH",
  "generator": "scripts/render-icon.sh",
  "method": "Authored SVG rendered with librsvg; ARGB derived with ffmpeg",
  "purpose": "Clavis application and StatusNotifier icon"
}
EOF
done
echo 'Rendered application PNG, tray PNG and embedded ARGB pixmap from the same SVG.'
