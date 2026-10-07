from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
#!/usr/bin/env python3
"""Package the supported open-only damage numbers; never install."""
import argparse
from mod_distribution import package
from source_provenance import verify as verify_source
from damage_numbers_package_helpers import digest, require, load, ROOT

RECIPE = MOD
BUILD = ROOT/'build/damage-numbers/open-assets'
PLUGIN = 'romfs/nativeNX/plugins/damage_numbers_capture.nro'


def main(refresh_support=False):
    built = load(BUILD/'candidate-manifest.json')
    probe = load(BUILD/'compiled-capture-probe.json')
    audit = load(BUILD/'asset-audit.json')
    codec = load(BUILD/'sprite-codec-test.json')
    comparison = load(BUILD/'compression-render-comparison.json')
    ge = load(BUILD/'ge-compatibility.json')
    gateway = load(BUILD/'gateway-probe.json')
    gateway_manifest = load(ROOT/'work/damage-numbers/gateway/manifest.json')
    require(built['asset_profile'] == 'open', 'Expected isolated open build')
    require(audit['status'] == 'PASSED_OPEN_ASSET_PROVENANCE', 'Open asset audit missing')
    for name, expected in built['sources'].items():
        verify_source(name, expected)
    plugin = (BUILD/'candidate-package'/PLUGIN).read_bytes()
    require(digest(plugin) == built['nro_sha256'] == probe['nro_sha256'] == audit['nro_sha256'],
            'Built, tested and audited plugin hashes differ')
    required={'Open Wilds compiled stamp starts at 1.2x',
              'Open Wilds compiled stamp settles at 0.2 seconds',
              'Open Wilds compiled hold stays stationary and opaque beyond 0.5 seconds',
              'Open Wilds compiled exit rises 15 pixels and halves opacity in 0.1 seconds',
              'Open edition compiled Classic keeps its early continuous drift',
              'R15 default 1x settles at 20 pixels at 720p',
              'R15 default 1x settles at 30 pixels at 1080p',
              'R15 size slider retains 24 pixels at 1.2x',
              'Lossless sprite decode and texture registration run once per ImGui context',
              'Damage capture and rendering do not call the sprite decoder',
              'Invalid embedded stream destroys the unregistered texture and retains crisp-digit fallback'}
    required.update('Compiled dn_gateway_'+name+' restores NZCVQ without unsupported APSR_g'
                    for name in [str(i) for i in range(8)]+['after'])
    require(probe['checks'] >= 340 and required.issubset(probe['passed']),
            'Baseline and new compiled animation checks are required')
    require(built['recovery_revision']==18 and built['base_font_pixels_720p']==20, 'Expected r18 settings/log cleanup build with 20px base size')
    require(ge['status']=='PASSED_NZCVQ_ONLY_GATEWAYS' and ge['nro_sha256']==digest(plugin) and
            len(ge['restore_sites'])==9 and gateway['checks']>=9287 and
            gateway['gateway_sha256']==gateway_manifest['binary_sha256'] and
            gateway['GE_boundary_experiments']=={'GE_preserving_helper':128,'GE_clobbering_helper':128},
            'Production gateway compatibility and GE boundary fixtures required')
    for name,expected in gateway_manifest['sources'].items():
        verify_source(name, expected)
    require(codec['status']=='PASSED_LOSSLESS_SPRITE_CODEC' and
            codec['rgba_sha256']==audit['sprite_rgba_sha256']==built['sprites']['rgba_sha256'] and
            codec['compressed_bytes']==built['sprites']['compressed_bytes'] and
            not codec['per_frame_decode'] and codec['extra_persistent_bytes']==0,
            'Lossless codec and memory verification required')
    require(comparison['status']=='PASSED_R15_COMPRESSION_EQUIVALENCE' and
            comparison['nro_sha256']==digest(plugin) and
            comparison['rgba_sha256']==audit['sprite_rgba_sha256'] and
            comparison['bss_bytes']==built['nro_bss_bytes'] and comparison['bss_not_increased'] and
            all(v['identical'] for v in comparison['real_imgui'].values()),
            'Exact r15 rendering and memory equivalence required')
    for name,record in comparison['real_imgui'].items():
        require(digest((ROOT/'work/damage-numbers/open-visual'/name).read_bytes())==record['sha256'],
                'Verified real ImGui output changed: '+name)
    release = load(RECIPE/'RELEASE.json')
    require(release['status'] in ('READY_TO_DISTRIBUTE', 'PUBLISHED') and
            release['nro_sha256'] == digest(plugin) and release['compiled_fixture_checks'] == probe['checks'],
            'Finished release record differs from validated binary')
    font = built['font']
    require(font['license_kind'] == 'SIL Open Font License 1.1' and
            font['source_kind'] == 'Pinned upstream Google Fonts, no game-derived font' and
            built['sprites']['source_sha256'] is None, 'Unexpected visual provenance')
    require(font['subset_sha256'] == audit['font_sha256'] and
            built['sprites']['rgba_sha256'] == audit['sprite_rgba_sha256'], 'Asset audit differs')
    require(digest((RECIPE/'fonts/OFL.txt').read_bytes()) == font['license_sha256'], 'Font license changed')
    target = package('damage-numbers', refresh_support=refresh_support)
    print(f'Open-only damage-number ZIP verified: {target}\nSHA-256: {digest(target.read_bytes())}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--refresh-support', action='store_true')
    main(parser.parse_args().refresh_support)
