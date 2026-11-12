#!/bin/sh
set -e

file_path="$1"
media_type="$2"
fit_mode="$3"
alignment="$4"
image_size="${5}x${6}"
screen_size="${7}x${8}"

if [ "$media_type" != image ]; then
    echo "No known way to set video as wallpaper in KDE: $media_type"
    exit 0
fi

# Конвертируем картинку, так как KDE не умеет делать это сам
target_path="$(
    "$(dirname "$0")/convert-image.sh" \
        "$file_path" "$fit_mode" "$alignment" "$image_size" "$screen_size"
)"

# Обновление рабочего стола
plasma-apply-wallpaperimage "$target_path"
