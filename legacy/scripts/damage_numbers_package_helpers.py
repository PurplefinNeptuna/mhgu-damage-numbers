# Shared deterministic serialization and ZIP helpers. Existing output must
# match byte-for-byte before it can be reused.

from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
#!/usr/bin/env python3
"""Deterministic, non-overwriting package helpers for open damage numbers."""
import hashlib
import io
import json
from pathlib import Path
import zipfile

from paths import ROOT, MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path


def digest(blob):
    return hashlib.sha256(blob).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def load(path):
    return json.loads(path.read_text())


def json_bytes(value):
    return (json.dumps(value, indent=2, sort_keys=True) + '\n').encode()


# Sort entries and fix timestamp/permissions so the same payload makes the same ZIP.
def make_zip(payloads):
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, 'w') as archive:
        for path, blob in sorted(payloads.items()):
            info = zipfile.ZipInfo('damage-number/' + path, (1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, blob, compresslevel=9)
    return buffer.getvalue()


# Reuse identical regular files; refuse symlinks or differing existing output.
def preserve_or_write(path, blob):
    require(not path.is_symlink(), f'Refusing symlink output: {path}')
    if path.exists():
        require(path.is_file() and path.read_bytes() == blob,
                f'Preserving differing existing release output: {path}')
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(blob)
