#!/bin/sh
set -e

file_path="$1"
media_type="$2"
fit_mode="$3"
alignment="$4"
#image_size="${5}x${6}"
#screen_size="${7}x${8}"

case "$media_type" in
    image)   scale=spline36 ;;
    video|*) scale=bilinear ;;
esac

case "$fit_mode" in
    stretch) mode_opt="--no-keepaspect" ;;
    contain) mode_opt="--panscan=0" ;;
    cover|*) mode_opt="--panscan=1" ;;
esac

case "$alignment" in
    left)  align_x=-1 ;;
    right) align_x=1 ;;
    *)     align_x=0 ;;
esac

case "$alignment" in
    top)    align_y=-1 ;;
    bottom) align_y=1 ;;
    *)      align_y=0 ;;
esac

echo mpvpaper -o "
    --loop=inf --image-display-duration=inf --ao=null
    --vo=gpu --hwdec=auto --video-sync=display-resample
    --scale=$scale --cscale=$scale --dscale=$scale
    --stop-screensaver=no --osc=no --config=no --load-scripts=no
    --term-status-msg= --video-unscaled=no
    $mode_opt --video-align-x=$align_x --video-align-y=$align_y
" ALL "$file_path"
