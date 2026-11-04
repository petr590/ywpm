#!/bin/sh
FLAG_FILE="$1/ywpm-restore.flag"
ENV_FILE="$1/ywpm.env"

if [ ! -f "$FLAG_FILE" ]; then
    echo "YWPM_RESTORE_FLAG=" > "$ENV_FILE"
    touch "$FLAG_FILE"
else
    echo "YWPM_RESTORE_FLAG=restore" > "$ENV_FILE"
fi
