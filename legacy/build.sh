#!/bin/sh
# Resolve this mod beside the launcher, then forward arguments to its dispatcher.
set -eu
MOD_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
exec "${PYTHON:-python3}" "$MOD_DIR/scripts/build.py" "$@"
