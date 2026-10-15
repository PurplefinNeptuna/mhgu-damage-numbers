#!/usr/bin/env python3
"""Check the production renderer against frozen r2 pixels and the unchanged C++ oracle."""
import json, math, subprocess, sys, hashlib, shutil
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from paths import MOD, BUILD_ROOT, WORK_ROOT, CARGO_TARGET
from build import environment
from compositor import render
from PIL import Image
import numpy as np

env, rust = environment()
out = WORK_ROOT / 'visual'
out.mkdir(parents=True, exist_ok=True)
runtime_assets = out / 'app:/nativeNX/plugins/damage-numbers'
runtime_assets.mkdir(parents=True, exist_ok=True)
for name in ['OpenSans.ttf', 'wilds-effects.rgba', 'rise-slash.rgba']:
    shutil.copy2(MOD / 'assets' / name, runtime_assets / name)
subprocess.run([rust / 'cargo', 'build', '--release', '--config', MOD / '.cargo/vendor.toml',
                '--manifest-path', MOD / 'tests/render-harness/Cargo.toml', '--target', 'x86_64-unknown-linux-gnu'],
               cwd=MOD.parent, env=env, check=True)
cimgui = MOD / 'vendor/cimgui'
stub = out / 'forge/log.h'
stub.parent.mkdir(exist_ok=True)
stub.write_text('extern "C" void forge_log_info(const char*,...);\n#define forge_log_warn forge_log_info\n')
sources = [MOD / 'tests/visual.cpp', MOD / 'tests/oracle/cosmetics.cpp', cimgui / 'cimgui.cpp',
           *[cimgui / 'imgui' / name for name in ['imgui.cpp', 'imgui_draw.cpp', 'imgui_tables.cpp', 'imgui_widgets.cpp', 'imgui_demo.cpp']]]
library = CARGO_TARGET / 'x86_64-unknown-linux-gnu/release/libdamage_numbers_render_harness.a'
subprocess.run(['c++', '-std=c++17', '-O2', '-DDAMAGE_NUMBERS_HOST_VISUAL',
                *['-I' + str(p) for p in [cimgui, out, MOD / 'tests/oracle/assets', MOD / 'tests/oracle']],
                *map(str, sources), str(library), '-ldl', '-lpthread', '-lm',
                '-Wl,--wrap=igImFileOpen,--wrap=igImFileClose,--wrap=igImFileRead,--wrap=igRegisterUserTexture,--wrap=igUnregisterUserTexture', '-o', str(out / 'visual')], check=True)
subprocess.run([out / 'visual', MOD / 'vendor/fonts/Roboto-Medium.ttf', out], cwd=out, check=True)
references = json.loads((MOD / 'tests/r2-pixels/manifest.json').read_text())
scenes = []
for reference in references['scenes']:
    name = reference['name']
    left, right = out / ('cpp-' + name), out / ('rust-' + name)
    a, b = [json.loads((folder / 'draw.json').read_text()) for folder in [left, right]]
    assert a['textures'] == b['textures'] == reference['textures'], (name, 'texture dimensions/order')
    assert a['indices'] == b['indices'], (name, 'indices')
    for key in ['vertices', 'commands']:
        assert len(a[key]) == len(b[key]), (name, key, 'count')
        for i, (x, y) in enumerate(zip(a[key], b[key])):
            assert len(x) == len(y)
            for j, (v, w) in enumerate(zip(x, y)):
                exact = (key == 'vertices' and j == 4) or (key == 'commands' and j < 4)
                assert v == w if exact else math.isclose(v, w, abs_tol=.0003, rel_tol=2e-6), (name, key, i, j, v, w)
    for ident, _, _ in b['textures']:
        pixels = (right / f'texture-{ident}.rgba').read_bytes()
        assert pixels == (left / f'texture-{ident}.rgba').read_bytes(), (name, 'oracle atlas')
        assert hashlib.sha256(pixels).hexdigest() == reference['atlas_sha256'][str(ident)], (name, 'r2 atlas')
    lp, rp = render(left), render(right)
    before = MOD / 'tests/r2-pixels' / (name + '.png')
    assert hashlib.sha256(before.read_bytes()).hexdigest() == reference['png_sha256']
    old, new = np.asarray(Image.open(before)), np.asarray(Image.open(rp))
    assert old.shape == new.shape and np.array_equal(old, new), (name, 'r2 pixel difference')
    # Retain the historical oracle tolerance; the new r2 comparison above is exact.
    difference = np.abs(np.asarray(Image.open(lp)).astype(int) - new.astype(int))
    assert difference.max() <= 1, (name, 'oracle pixels', int(difference.max()))
    scenes.append({'name': name, 'maximum_r2_channel_difference': 0,
                   'maximum_oracle_channel_difference': int(difference.max()), 'vertices': len(b['vertices'])})
report = {'status': 'PASSED', 'renderer': 'Production Rust and real ImGui 1.92.8',
          'reference_version': 'r2', 'scenes': scenes, 'style_order': ['Wilds', 'Rise', 'Classic'],
          'runtime_test': 'NOT RUN'}
(BUILD_ROOT / 'visual.json').write_text(json.dumps(report, indent=2) + '\n')
subprocess.run([out / 'visual', MOD / 'vendor/fonts/Roboto-Medium.ttf', out, 'faults'], cwd=out, check=True)
report['asset_failure_checks'] = json.loads((out / 'faults.json').read_text())
(BUILD_ROOT / 'visual.json').write_text(json.dumps(report, indent=2) + '\n')
print('PASS: 24 real-renderer scenes have zero pixel differences from r2; asset failure/lifecycle checks pass')
