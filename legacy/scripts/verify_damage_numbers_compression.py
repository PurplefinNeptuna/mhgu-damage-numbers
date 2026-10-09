#!/usr/bin/env python3
# Compare NRO/BSS size, font/atlas metadata, and retained ImGui texture hashes.

"""Verify unchanged assets and the approved r25 size metadata and historical texture baseline."""
import hashlib, json
from paths import ROOT, MOD, BUILD_ROOT, WORK_ROOT
from damage_numbers_assets import paths


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


# Check the r25 size metadata and asset hashes against the recorded texture baseline.
def main():
    work, out = paths('open')
    b = json.loads((MOD / 'research/visual-baseline.json').read_text())
    c = json.loads((out / 'candidate-manifest.json').read_text())
    assert c['nro_file_bytes'] < b['old_nro_bytes'] and c['nro_bss_bytes'] <= b['bss_bytes']
    assert c['font'] == b['font']
    assert sha(work / 'wilds-effects.rgba') == b['sprites_rgba_sha256']
    accepted = json.loads((MOD / 'research/readability-r20.json').read_text())
    assert c['recovery_revision'] == 21 and c['wilds_outline_pixels_at_base'] == .5 and not c['wilds_weakspot_glow'] and c['wilds_crit_size_multiplier'] == 1.5
    assert c['base_font_pixels_720p'] == 24 and c['rise_base_font_pixels_720p'] == 20 and c['rise_weak_font_pixels_720p'] == 28
    # r25 changes geometry intentionally; r20 remains the exact font/art texture baseline.
    expected = {name: accepted['real_imgui'][name] for name in ('texture-1.rgba', 'texture-2.rgba')}
    for name in ('texture-1.rgba', 'texture-2.rgba'):
        assert expected[name] == b['real_imgui'][name]
    comparisons = {}
    for name, h in expected.items():
        p = WORK_ROOT / 'open-visual' / name
        assert sha(p) == h, name
        comparisons[name] = {'identical': True, 'sha256': h, 'bytes': p.stat().st_size}
    report = {'status': 'PASSED_R25_SIZE_ASSET_EQUIVALENCE', 'nro_sha256': c['nro_sha256'], 'reference_nro_sha256': b['reference_nro_sha256'], 'appearance_baseline': 'research/readability-r20.json', 'appearance_baseline_sha256': sha(MOD / 'research/readability-r20.json'), 'old_nro_bytes': b['old_nro_bytes'], 'nro_bytes': c['nro_file_bytes'], 'bss_bytes': c['nro_bss_bytes'], 'bss_not_increased': True, 'font_manifest_unchanged': True, 'rgba_sha256': sha(
        work / 'wilds-effects.rgba'), 'real_imgui': comparisons, 'renderer_geometry_changed_intentionally': True, 'renderer_geometry_baseline': 'docs/R25_SIZE_CHANGE.json', 'scope': 'Unchanged font and artwork textures compared with the retained r20 baseline; r25 style sizes and outline metadata checked. Draw/preview equality with r20 is not claimed. Geometry fixtures run separately; no live GPU claim.'}
    (out / 'compression-render-comparison.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS: unchanged font/artwork textures, approved r25 size metadata and bounded BSS')


if __name__ == '__main__':
    main()
