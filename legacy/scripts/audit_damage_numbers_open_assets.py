# Verify licensed/procedural asset provenance and decoded payload bytes in
# the built NRO, including compiler-exported asset symbols.

from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
#!/usr/bin/env python3
"""Verify the supported font/art provenance and block proprietary visual inputs."""
from pathlib import Path
import hashlib
import json
import re
import sys
from damage_numbers_assets import ROOT, paths

WORK, OUT = paths('open')


# Reject known proprietary visual input paths through Python's file-open audit hook.
def audit_open(event, args):
    if event == 'open' and isinstance(args[0], (str, bytes, Path)):
        name = str(args[0]).lower()
        if any(part in name for part in ('wilds-assets/', 'monsterhunterwilds/', 'frutiger', 'damage-numbers/native-capture/', 'monsterhunterrise/', 'rise-research/', 'gp-udkakugo')):
            raise RuntimeError('Proprietary visual input blocked: ' + name)


# Cross-check asset licenses, procedural provenance, symbols, and embedded pixel data.
def main():
    sys.addaudithook(audit_open)
    from prepare_damage_numbers_sprites import prepare
    manifest = prepare('open')
    from prepare_damage_numbers_rise import prepare as prepare_rise
    rise = prepare_rise()
    font = json.loads((WORK / 'font-manifest.json').read_text())
    if font['license_kind'] != 'SIL Open Font License 1.1' or manifest['source_sha256'] is not None:
        raise ValueError('Unexpected open asset provenance')
    if manifest['asset_profile'] != 'open' or font['family'] != 'Open Sans':
        raise ValueError('Mixed build profiles')
    nro = (OUT / 'candidate-package/romfs/nativeNX/plugins/damage_numbers_capture.nro').read_bytes()
    symbols = dict((name, int(offset, 16)) for offset, name in re.findall(
        r'^([0-9a-f]+) r (\S+)$', (WORK / 'symbols.txt').read_text(), re.M))
    font_data = (WORK / 'wilds-digits.ttf').read_bytes()
    license_data = (MOD / 'fonts/OFL.txt').read_bytes() + b'\0'
    header = (WORK / 'wilds_sprite_data.h').read_text()
    array = header.split('wilds_sprite_lz[]={\n', 1)[1].split('};', 1)[0]
    packed = bytes(map(int, re.findall(r'\d+', array)))
    from damage_numbers_sprite_codec import decode
    decoded = decode(packed, manifest['decoded_bytes'])
    if hashlib.sha256(decoded).hexdigest() != manifest['rgba_sha256']:
        raise ValueError('Lossless decoded artwork differs')
    rise_header = (WORK / 'rise_sprite_data.h').read_text()
    rise_packed = bytes(map(int, re.findall(
        r'\d+', rise_header.split('rise_sprite_lz[] __attribute__((used))={', 1)[1].split('};', 1)[0])))
    if hashlib.sha256(decode(rise_packed, rise['decoded_bytes'])).hexdigest() != rise['rgba_sha256']:
        raise ValueError('Rise lossless art differs')
    for symbol, expected in [('_ZL15wilds_font_data', font_data),
                             ('_ZL18wilds_font_license', license_data),
                             ('_ZL15wilds_sprite_lz', packed), ('_ZL14rise_sprite_lz', rise_packed)]:
        at = symbols[symbol]
        if nro[at:at + len(expected)] != expected:
            raise ValueError('Compiled asset differs: ' + symbol)
    # Compare complete generated payloads, not selected visual samples.
    checks = ['Open asset generation succeeds with proprietary visual reads blocked',
              'Pinned Open Sans source and OFL hashes verified',
              'Compiled renamed digit subset equals open font payload',
              'Compiled full OFL notice is embedded',
              'Compiled DN-LZ1 sprite equals independently generated art/effect payload',
              'Lossless decoding reproduces every accepted RGBA byte']
    report = {'status': 'PASSED_OPEN_ASSET_PROVENANCE',
              'nro_sha256': hashlib.sha256(nro).hexdigest(),
              'font_sha256': hashlib.sha256(font_data).hexdigest(),
              'sprite_rgba_sha256': manifest['rgba_sha256'], 'rise_rgba_sha256': rise['rgba_sha256'], 'checks': checks + ['Compiled procedural Rise beam equals complete open RGBA payload; Rise proprietary reads blocked'],
              'scope': 'Visual asset provenance; no blanket legal conclusion or in-game acceptance'}
    (OUT / 'asset-audit.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Passed open asset provenance and compiled payload audit; proprietary visual reads blocked')


if __name__ == '__main__':
    main()
