"""Resolve this mod's own inputs and candidate paths, in either workspace or exported form."""
from pathlib import Path
import json, sys
MOD=Path(__file__).resolve().parents[1]
SCRIPTS=MOD/'scripts'
WORKSPACE=MOD.parent.name=='mods' and (MOD.parent.parent/'scripts/mod_distribution.py').is_file()
ROOT=MOD.parent.parent if WORKSPACE else MOD
CONFIG=json.loads((MOD/'build.local.json').read_text()) if (MOD/'build.local.json').exists() else {}
BUILD_ROOT=ROOT/'build/damage-numbers/candidates/r29-diagnostics'
WORK_ROOT=ROOT/'work/damage-numbers/candidates/r29-diagnostics'
CARGO_TARGET=WORK_ROOT/'cargo-target'
FORGE=MOD/'vendor/forge-rs'
DEPS=MOD/'vendor'
PYTHON=Path(CONFIG.get('python',sys.executable))
CONTAINER_STORAGE=Path(CONFIG.get('container_storage',ROOT/'tools/forge-container/storage'))
def input_path(name):return Path(CONFIG.get('inputs',ROOT/'work/damage-numbers/inputs'))/name
