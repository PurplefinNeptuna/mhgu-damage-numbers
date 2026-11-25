#!/bin/sh
set -eu
export PYTHONDONTWRITEBYTECODE=1
MOD_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
exec "${PYTHON:-python3}" "$MOD_DIR/scripts/candidate_guard.py" "$@"
