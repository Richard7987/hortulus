#!/bin/sh
# Paints the README's images into package/images. Needs typst and ImageMagick.
set -eu
cd "$(dirname "$0")/.."
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

sheet() {
  typst compile --root . --ppi 51 "tools/gallery/$1.typ" "$tmp/$1-{0p}.png"
  magick montage "$tmp/$1"-*.png -tile 4x -geometry +6+6 -background '#d8d8d8' \
    -strip -quality 82 "package/images/$1.jpg"
}

sheet colourways
sheet tuning
sheet content

typst compile --root . --ppi 150 tools/gallery/detail.typ "$tmp/detail.png"
magick "$tmp/detail.png" -crop 1240x620+0+0 +repage -strip -quality 85 package/images/detail.jpg
