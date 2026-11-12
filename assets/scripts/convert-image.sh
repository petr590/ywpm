#!/bin/sh
# Скрипт конвертирует картинку в заданное разрешение и выводит имя файла с
# итоговой картинкой. Внимание! Если размер картинки уже совпадает с размером
# экрана, то конвертация не произойдёт и будет выведено исходное имя файла.
# Имя файла уникально для каждого запуска. Файл после конвертации находится
# на /tmp.
#
# Использование:
#   convert-image.sh <file_path> <fit_mode> <alignment> <image_size> <screen_size>

set -e


if [ $# -ne 5 ]; then
    echo "$0: exactly 5 arguments required" >&2
    exit 1
fi

if command -v magick >/dev/null 2>&1; then
    convert_cmd=magick
elif command -v convert >/dev/null 2>&1; then
    convert_cmd=convert
else
    echo "$0: neither the 'magick' nor 'convert' command was found" >&2
    exit 1
fi


file_path="$1"
fit_mode="$2"
alignment="$3"
image_size="$4"
screen_size="$5"

rm -f /tmp/ywpm_tmp_*.png

if [ "$image_size" = "$screen_size" ] || [ "$image_size" = 0x0 ] || [ "$screen_size" = 0x0 ]; then
    echo "$file_path"

else
    target_path="$(mktemp /tmp/ywpm_tmp_XXXXXX --suffix=.png)"

    case "$alignment" in
        top)      gravity=North ;;
        bottom)   gravity=South ;;
        left)     gravity=West ;;
        right)    gravity=East ;;
        center|*) gravity=Center ;;
    esac

    case "$fit_mode" in
        stretch)
            "$convert_cmd" "$file_path" -resize "$screen_size!" "$target_path"
            ;;

        contain)
            "$convert_cmd" "$file_path" -resize "$screen_size" -background gray \
                -gravity "$gravity" -extent "$screen_size" "$target_path"
            ;;

        cover|*)
            "$convert_cmd" "$file_path" -resize "$screen_size^" -gravity "$gravity" \
                -crop "$screen_size+0+0" +repage "$target_path"
            ;;
    esac

    echo "$target_path"
fi
