from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
"""Only the licensed font and procedural art profile is supported."""
from pathlib import Path
from paths import ROOT, MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path

def paths(profile='open'):
    if profile == 'open':
        return ROOT/'work/damage-numbers/open-assets', ROOT/'build/damage-numbers/open-assets'
    raise ValueError('Unknown asset profile')

def argument():
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('--assets', choices=('open',), default='open')
    return parser.parse_args().assets
