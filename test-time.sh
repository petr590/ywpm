#!/bin/bash
DIR="$HOME/0x590/Imgs/sorted/anime/Смертельная игра ради еды на столе/wallpapers"
now="$(date --iso-8601=minutes)"
offset=1

for f in "$DIR"/*; do
    since="$(date -d "$now +$offset minutes" '+%Y-%m-%d %H:%M')"
    let offset++
    until="$(date -d "$now +$offset minutes" '+%Y-%m-%d %H:%M')"

    cargo run --bin ywpm -- set "$f" -s "$since" -u "$until"
done
