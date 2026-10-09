#!/usr/bin/env python3
"""Build and verify a standalone Rust candidate. No command installs or launches the game."""
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import zipfile
from html import escape
from pathlib import Path
from paths import MOD, ROOT, CONFIG, BUILD_ROOT, WORK_ROOT, PYTHON


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def record(name, value):
    BUILD_ROOT.mkdir(parents=True, exist_ok=True)
    (BUILD_ROOT / name).write_text(json.dumps(value, indent=2) + '\n')


def environment():
    env = os.environ.copy()
    runtime = ROOT / 'tools/mhgu-dti-dumper'
    rust = Path(CONFIG.get('rust_bin', runtime / 'rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin'))
    cargo_home = Path(CONFIG.get('cargo_home', runtime / 'cargo'))
    compiler = CONFIG.get('devkitarm') or os.environ.get('DEVKITARM')
    if not compiler:
        raise ValueError('Set devkitarm in build.local.json or DEVKITARM')
    env.update(CARGO_HOME=str(cargo_home), RUSTUP_HOME=str(runtime / 'rustup'),
               RUSTUP_TOOLCHAIN='nightly', CARGO_NET_OFFLINE='true', DEVKITARM=str(compiler),
               DN_ASSETS=str(WORK_ROOT / 'open-assets'))
    env['PATH'] = os.pathsep.join([str(rust), str(Path(compiler) / 'bin'), env.get('PATH', '')])
    return env, rust


def converter():
    return Path(CONFIG.get('cargo_forge', ROOT / 'tools/mhgu-dti-dumper/cargo/bin/cargo-forge'))


def run(command, **kwargs):
    subprocess.run([str(value) for value in command], check=True, **kwargs)


def doctor():
    env, rust = environment()
    version = subprocess.check_output([rust / 'rustc', '--version'], env=env, text=True).strip()
    lock = json.loads((MOD / 'dependencies.lock.json').read_text())
    if version != lock['rust_version']:
        raise ValueError('Use nightly-2026-10-02 with rust-src')
    for name, path in [('rustc', rust / 'rustc'), ('cargo_forge', converter())]:
        if digest(path) != lock['tool_sha256'][name]:
            raise ValueError('Pinned tool binary differs: ' + name)
    for name, expected in lock['files'].items():
        if digest(MOD / name) != expected:
            raise ValueError('Pinned dependency differs: ' + name)
    print(version)
    print('Pinned compiler, forge-rs, registry sources, fonts and native declarations checked')


def prepare(fetch=False):
    env, rust = environment()
    if fetch:
        from vendor_sysroot import fetch as fetch_sysroot
        fetch_sysroot(rust)
        env['CARGO_NET_OFFLINE'] = 'false'
        run([rust / 'cargo', 'fetch', '--locked', '--manifest-path', MOD / 'Cargo.toml'], cwd=MOD, env=env)
    run([PYTHON, MOD / 'scripts/prepare_assets.py'], cwd=MOD)


def build():
    doctor()
    prepare()
    env, _ = environment()
    lock_before = digest(MOD / 'Cargo.lock')
    run([converter(), 'forge', 'build', '--release'], cwd=MOD, env=env)
    if digest(MOD / 'Cargo.lock') != lock_before:
        raise ValueError('Cargo.lock changed during build; pin dependencies before retrying')
    plugins = BUILD_ROOT / 'package/romfs/nativeNX/plugins'
    plugins.mkdir(parents=True, exist_ok=True)
    shutil.copy2(MOD / 'damage_numbers_rs.nro', plugins / 'damage_numbers_rs.nro')
    assets = plugins / 'damage-numbers-rs'
    assets.mkdir(exist_ok=True)
    shutil.copy2(WORK_ROOT / 'open-assets/wilds-digits.ttf', assets / 'wilds-digits.ttf')
    record('build.json', {'status': 'BUILT', 'nro_sha256': digest(plugins / 'damage_numbers_rs.nro'),
                         'font_sha256': digest(assets / 'wilds-digits.ttf'),
                         'forge_rs_revision': '7f572ed8d0e0a6cfc562571fdbc1ad57441a8438', 'runtime_test': 'NOT RUN'})


def test(host=False):
    env, rust = environment()
    # Source-only config keeps host commands free of the ARM target/build-std configuration.
    for manifest in ['core/Cargo.toml', 'tests/config-harness/Cargo.toml', 'tests/hooks-harness/Cargo.toml']:
        run([rust / 'cargo', 'test', '--locked', '--config', MOD / '.cargo/vendor.toml',
             '--manifest-path', MOD / manifest, '--target', 'x86_64-unknown-linux-gnu'], cwd=MOD.parent, env=env)
    run([PYTHON, MOD / 'tests/parity.py'], env=env)
    run([PYTHON, MOD / 'tests/visual.py'], env=env)
    nro = BUILD_ROOT / 'package/romfs/nativeNX/plugins/damage_numbers_rs.nro'
    if not host:
        run([CONFIG.get('arm_python', ROOT / 'tools/mwc-analysis-venv/bin/python'), MOD / 'tests/gateways.py'])
        verify()
    record('tests.json', {'status': 'PASSED', 'host': True, 'arm_verified': not host,
                          'nro_sha256': digest(nro) if not host else None, 'runtime_test': 'NOT RUN'})


def verify():
    run([PYTHON, MOD / 'scripts/verify.py'])


def package():
    verify()
    source = BUILD_ROOT / 'package'
    nro = source / 'romfs/nativeNX/plugins/damage_numbers_rs.nro'
    tests = json.loads((BUILD_ROOT / 'tests.json').read_text())
    if not tests['arm_verified'] or tests['nro_sha256'] != digest(nro):
        raise ValueError('Run the complete test command for this exact NRO before packaging')
    dest = BUILD_ROOT / 'delivery'
    dest.mkdir(parents=True, exist_ok=True)
    payloads = {str(path.relative_to(source)): digest(path) for path in source.rglob('*') if path.is_file()}
    package_dir = dest / 'damage-numbers-rs'
    existing = {str(path.relative_to(package_dir)) for path in package_dir.rglob('*') if path.is_file()}
    if existing - payloads.keys():
        raise ValueError('Unexpected old candidate payload files; preserve them before repackaging')
    shutil.copytree(source, package_dir, dirs_exist_ok=True)
    guide = (MOD / 'DISTRIBUTION_README.md').read_bytes()
    archive = dest / 'damage-numbers-rs-r1-candidate.zip'
    with zipfile.ZipFile(archive, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as zipped:
        zipped.writestr('README.md', guide)
        for name in sorted(payloads):
            zipped.write(source / name, 'damage-numbers-rs/' + name)
    (dest / 'README.md').write_bytes(guide)
    import mod_distribution as distribution
    spec = json.loads((MOD / 'DISTRIBUTION.json').read_text())
    (dest / 'README.html').write_bytes(distribution.readme_html(guide))
    (dest / 'GAMEBANANA_README.html').write_bytes(distribution.gamebanana_readme_html(guide, spec['gamebanana']))
    licenses = ['DISTRIBUTION_NOTICES.md', 'LICENSE', 'vendor/forge-rs/LICENSE', 'fonts/OFL.txt',
                'vendor/libnx/LICENSE-MIT', 'vendor/crates/cfg-if-1.0.5/LICENSE-MIT',
                'vendor/crates/log-0.4.34/LICENSE-MIT', 'vendor/crates/ini_core-0.2.0/license.txt',
                'licenses/RUST-MIT.txt', 'licenses/COMPILER_BUILTINS.txt',
                'vendor/cimgui/LICENSE', 'vendor/cimgui/imgui/LICENSE.txt']
    notices = '\n\n'.join((MOD / name).read_text() for name in licenses)
    (dest / 'NOTICES.txt').write_text(notices)
    (dest / 'NOTICES.html').write_text('<pre>' + escape(notices) + '</pre>\n')
    evidence = {name: json.loads((BUILD_ROOT / (name + '.json')).read_text())
                for name in ['tests', 'parity', 'visual', 'gateways', 'verify']}
    (dest / 'VALIDATION.json').write_text(json.dumps(evidence, indent=2) + '\n')
    manifest = {'mod': 'damage-numbers-rs', 'version': 'r1', 'status': 'DEVELOPMENT', 'runtime_test': 'NOT RUN',
                'package_name': 'damage-numbers-rs', 'payloads': payloads, 'archive_sha256': digest(archive)}
    (dest / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    (dest / 'SHA256SUMS').write_text(''.join(digest(path) + '  ' + str(path.relative_to(dest)) + '\n'
                                          for path in sorted(dest.rglob('*')) if path.is_file() and path.name != 'SHA256SUMS'))
    with zipfile.ZipFile(archive) as zipped:
        assert zipped.testzip() is None and zipped.read('README.md') == guide
        assert set(zipped.namelist()) == {'README.md', *('damage-numbers-rs/' + name for name in payloads)}
        for name, expected in payloads.items():
            assert hashlib.sha256(zipped.read('damage-numbers-rs/' + name)).hexdigest() == expected
            assert digest(package_dir / name) == expected
    for line in (dest / 'SHA256SUMS').read_text().splitlines():
        expected, name = line.split('  ', 1)
        assert digest(dest / name) == expected
    print('Verified candidate:', archive)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['doctor', 'prepare', 'build', 'test', 'verify', 'package', 'export'])
    parser.add_argument('--host', action='store_true')
    parser.add_argument('--fetch', action='store_true')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    if args.command == 'prepare':
        prepare(args.fetch)
    elif args.command == 'test':
        test(args.host)
    elif args.command == 'export':
        if not args.output:
            parser.error('export needs --output')
        run([sys.executable, MOD / 'scripts/export_source.py', '--output', args.output])
    else:
        globals()[args.command]()


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
