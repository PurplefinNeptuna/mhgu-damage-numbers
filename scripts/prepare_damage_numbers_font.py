# Validate the licensed source font, pin its axes, subset required glyphs, and
# write an embedded header with the scale needed to match Forge digit height.

from paths import (
    MOD,
    SCRIPTS,
    FORGE,
    DEPS,
    PYTHON,
    CONTAINER_STORAGE,
    input_path,
    BUILD_ROOT,
    WORK_ROOT,
)
#!/usr/bin/env python3
"""Prepare the pinned digit font and C++ payload; never touch installed packages."""
from pathlib import Path
import hashlib
import io
import json
import fontTools
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont
from fontTools import subset

from paths import ROOT, MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
OUT = WORK_ROOT / 'open-assets'


# Keep font axes, glyph inventory, naming, and timestamps deterministic.
# Validate the emitted subset before embedding its bytes in a C++ header.
def prepare(profile='open'):
    from damage_numbers_assets import paths
    global OUT
    OUT, _ = paths(profile)
    recipe = MOD / 'fonts'
    manifest = json.loads((recipe / 'open-source.json').read_text())
    if fontTools.__version__ != manifest['fonttools_version']:
        raise ValueError('Use the pinned FontTools version recorded in fonts/open-source.json')
    source = MOD / 'vendor/fonts/OpenSans.ttf'
    raw = source.read_bytes()
    if hashlib.sha256(raw).hexdigest() != manifest['source_sha256']:
        raise ValueError('Digit font source fingerprint changed')
    license_data = (MOD / 'fonts/OFL.txt').read_bytes()
    if hashlib.sha256(license_data).hexdigest() != manifest['license_sha256']:
        raise ValueError('Open font license fingerprint changed')
    font = TTFont(io.BytesIO(raw), recalcTimestamp=False)
    if manifest['axes']:
        font = instantiateVariableFont(font, manifest['axes'], inplace=True)
    options = subset.Options()
    options.recalc_timestamp = False
    options.name_IDs = ['*']
    options.name_legacy = True
    sub = subset.Subsetter(options=options)
    sub.populate(text=manifest['glyphs'])
    sub.subset(font)
    for record in font['name'].names:
        names = {1: manifest['subset_family'], 2: 'Semibold',
                 3: 'DamageDigitsOpen-600-1', 4: 'Damage Digits Open Semibold',
                 6: 'DamageDigitsOpen-Semibold', 16: manifest['subset_family'], 17: 'Semibold'}
        if record.nameID in names:
            record.string = names[record.nameID].encode(record.getEncoding())
    # Match visible digit height to the existing Forge font at the same size.

    def cap_fraction(f):
        g = f['glyf'][f.getBestCmap()[ord('0')]]
        return (g.yMax - g.yMin) / (f['hhea'].ascent - f['hhea'].descent)
    classic_path = MOD / 'vendor/fonts/Roboto-Medium.ttf'
    classic = TTFont(classic_path)
    scale = cap_fraction(classic) / cap_fraction(font)
    output = io.BytesIO()
    font.save(output)
    data = output.getvalue()
    verify = TTFont(io.BytesIO(data))
    if verify['OS/2'].usWeightClass != manifest['weight']:
        raise ValueError('Instantiated font weight differs')
    if 'fvar' in verify or any(ord(c) not in verify.getBestCmap() for c in '0123456789'):
        raise ValueError('Subset is not a static complete digit font')
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / 'wilds-digits.ttf').write_bytes(data)

    def array(name, value):
        rows = [','.join(str(b) for b in value[i:i + 24]) for i in range(0, len(value), 24)]
        return f'static const unsigned char {name}[] __attribute__((used))={{\n' + ',\n'.join(rows) + '\n};\n'
    header = '#pragma once\n' + array('wilds_font_data', data)
    header += array('wilds_font_license', license_data + b'\0')
    header += f'static constexpr float wilds_font_scale={scale:.9f}f;\n'
    (OUT / 'wilds_font_data.h').write_text(header)
    report = dict(manifest, subset_sha256=hashlib.sha256(data).hexdigest(),
                  subset_bytes=len(data), height_scale=scale,
                  classic_height_reference_sha256=hashlib.sha256(classic_path.read_bytes()).hexdigest())
    (OUT / 'font-manifest.json').write_text(json.dumps(report, indent=2) + '\n')
    return report


if __name__ == '__main__':
    from damage_numbers_assets import argument
    print(json.dumps(prepare(argument()), indent=2))
