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
from paths import MOD, ROOT, CONFIG, BUILD_ROOT, WORK_ROOT, CARGO_TARGET, PYTHON


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
               CARGO_TARGET_DIR=str(CARGO_TARGET))
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
    if not (MOD/'vendor/crates').is_dir(): raise ValueError('Dependencies missing: run ./build.sh fetch')
    for name, expected in lock['files'].items():
        if digest(MOD / name) != expected:
            raise ValueError('Pinned dependency differs: ' + name)
    print(version)
    print('Pinned compiler, forge-rs, registry sources, fonts and native declarations checked')


def asset_hashes():
    """Prepared runtime files are copied verbatim; no image/font tools run at build time."""
    manifest = json.loads((MOD / 'assets/manifest.json').read_text())
    records = [manifest['font'], *manifest['artwork']]
    result = {}
    for item in records:
        path = MOD / 'assets' / item['file']
        if path.stat().st_size != item['bytes'] or digest(path) != item['sha256']:
            raise ValueError('Prepared asset differs: ' + item['file'])
        result[item['file']] = item['sha256']
    return result


def prepare():
    environment()  # an export without build.local.json stops here with an input error
    asset_hashes()
    print('Prepared asset files checked; no generation required')


def build():
    doctor()
    prepare()
    env, _ = environment()
    lock_before = digest(MOD / 'Cargo.lock')
    # Pinned cargo-forge looks up ./target and writes its NRO beside Cargo.toml.
    # Bridge that lookup only while it runs; keep Cargo's cache and converter
    # output in work/ even when conversion fails. Do not replace unknown files.
    target_link = MOD / 'target'
    converter_output = MOD / 'damage_numbers.nro'
    if target_link.exists() or target_link.is_symlink() or converter_output.exists():
        raise ValueError('Move existing target/ or loose NRO out of the source folder before building')
    cargo_target = Path(env['CARGO_TARGET_DIR'])
    cargo_target.mkdir(parents=True, exist_ok=True)
    generated_nro = cargo_target / 'damage_numbers.nro'
    target_link.symlink_to(cargo_target, target_is_directory=True)
    try:
        run([converter(), 'forge', 'build', '--release'], cwd=MOD, env=env)
    finally:
        target_link.unlink()
        if converter_output.exists():
            shutil.move(converter_output, generated_nro)
    if digest(MOD / 'Cargo.lock') != lock_before:
        raise ValueError('Cargo.lock changed during build; pin dependencies before retrying')
    plugins = BUILD_ROOT / 'package/romfs/nativeNX/plugins'
    plugins.mkdir(parents=True, exist_ok=True)
    shutil.copy2(generated_nro, plugins / 'damage_numbers.nro')
    assets = plugins / 'damage-numbers'
    assets.mkdir(exist_ok=True)
    hashes = asset_hashes()
    for name in hashes:
        shutil.copy2(MOD / 'assets' / name, assets / name)
    # Mod managers read manifest.txt beside romfs/ for the name, version and authors.
    shutil.copy2(MOD / 'manifest.txt', BUILD_ROOT / 'package/manifest.txt')
    record('build.json', {'status': 'BUILT', 'nro_sha256': digest(plugins / 'damage_numbers.nro'),
                         'asset_sha256': hashes,
                         'forge_rs_revision': 'd1865b68e9b194eb3eacc8bd71bbf601a5fac63e', 'runtime_test': 'NOT RUN'})


def test(host=False):
    env, rust = environment()
    # Source-only config keeps host commands free of the ARM target/build-std configuration.
    for manifest in ['core/Cargo.toml', 'tests/config-harness/Cargo.toml', 'tests/hooks-harness/Cargo.toml']:
        run([rust / 'cargo', 'test', '--locked', '--config', MOD / '.cargo/vendor.toml',
             '--manifest-path', MOD / manifest, '--target', 'x86_64-unknown-linux-gnu'], cwd=MOD.parent, env=env)
    run([PYTHON, MOD / 'tests/native_pool.py'], env=env)
    run([PYTHON, MOD / 'tests/parity.py'], env=env)
    run([PYTHON, MOD / 'tests/visual.py'], env=env)
    nro = BUILD_ROOT / 'package/romfs/nativeNX/plugins/damage_numbers.nro'
    if not host:
        run([CONFIG.get('arm_python', ROOT / 'tools/mwc-analysis-venv/bin/python'), MOD / 'tests/gateways.py'])
        run([CONFIG.get('arm_python', ROOT / 'tools/mwc-analysis-venv/bin/python'), MOD / 'tests/viewport.py'])
        verify()
    record('tests.json', {'status': 'PASSED', 'host': True, 'arm_verified': not host,
                          'nro_sha256': digest(nro) if not host else None, 'asset_sha256': asset_hashes(), 'runtime_test': 'NOT RUN'})


def verify():
    run([PYTHON, MOD / 'scripts/verify.py'])


def package():
    verify()
    source = BUILD_ROOT / 'package'
    nro = source / 'romfs/nativeNX/plugins/damage_numbers.nro'
    tests = json.loads((BUILD_ROOT / 'tests.json').read_text())
    if not tests['arm_verified'] or tests['nro_sha256'] != digest(nro) or tests['asset_sha256'] != asset_hashes():
        raise ValueError('Run the complete test command for this exact NRO before packaging')
    spec = json.loads((MOD / 'DISTRIBUTION.json').read_text())
    if spec['status'] in ('READY_TO_DISTRIBUTE', 'PUBLISHED'):
        # The same distribution engine owns workspace and standalone releases.
        # Its payload pins keep approval tied to these exact tested bytes.
        import mod_distribution as distribution
        print('Verified release:', distribution.package(spec['mod'], ROOT))
        return
    dest = BUILD_ROOT / 'delivery'
    dest.mkdir(parents=True, exist_ok=True)
    payloads = {str(path.relative_to(source)): digest(path) for path in source.rglob('*') if path.is_file()}
    package_dir = dest / 'damage-numbers'
    existing = {str(path.relative_to(package_dir)) for path in package_dir.rglob('*') if path.is_file()}
    if existing - payloads.keys():
        raise ValueError('Unexpected old candidate payload files; preserve them before repackaging')
    shutil.copytree(source, package_dir, dirs_exist_ok=True)
    guide = (MOD / 'README.md').read_bytes()
    archive = dest / 'damage-numbers-r43-candidate.zip'
    with zipfile.ZipFile(archive, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as zipped:
        zipped.writestr('README.md', guide)
        for name in sorted(payloads):
            zipped.write(source / name, 'damage-numbers/' + name)
    (dest / 'README.md').write_bytes(guide)
    import mod_distribution as distribution
    spec = json.loads((MOD / 'DISTRIBUTION.json').read_text())
    (dest / 'README.html').write_bytes(distribution.readme_html(guide))
    (dest / 'GAMEBANANA_README.html').write_bytes(distribution.gamebanana_readme_html(guide, spec['gamebanana']))
    licenses = ['DISTRIBUTION_NOTICES.md', 'LICENSE', 'licenses/FORGE-RS-MIT.txt', 'fonts/OFL.txt',
                'vendor/libnx/LICENSE-MIT', 'licenses/CFG-IF-MIT.txt',
                'licenses/LOG-MIT.txt', 'licenses/INI-CORE-MIT.txt',
                'licenses/RUST-MIT.txt', 'licenses/UNICODE-3.0.txt', 'licenses/COMPILER_BUILTINS.txt',
                'licenses/CIMGUI-MIT.txt', 'licenses/IMGUI-MIT.txt']
    notices = '\n\n'.join((MOD / name).read_text() for name in licenses)
    (dest / 'NOTICES.txt').write_text(notices)
    (dest / 'NOTICES.html').write_text('<pre>' + escape(notices) + '</pre>\n')
    evidence = {name: json.loads((BUILD_ROOT / (name + '.json')).read_text())
                for name in ['tests', 'native-pool', 'parity', 'visual', 'gateways', 'viewport', 'verify']}
    (dest / 'VALIDATION.json').write_text(json.dumps(evidence, indent=2) + '\n')
    manifest = {'mod': 'damage-numbers', 'version': 'r43', 'status': 'DEVELOPMENT', 'runtime_test': 'NOT RUN',
                'package_name': 'damage-numbers', 'payloads': payloads, 'archive_sha256': digest(archive)}
    (dest / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    (dest / 'SHA256SUMS').write_text(''.join(digest(path) + '  ' + str(path.relative_to(dest)) + '\n'
                                          for path in sorted(dest.rglob('*')) if path.is_file() and path.name != 'SHA256SUMS'))
    with zipfile.ZipFile(archive) as zipped:
        assert zipped.testzip() is None and zipped.read('README.md') == guide
        assert set(zipped.namelist()) == {'README.md', *('damage-numbers/' + name for name in payloads)}
        for name, expected in payloads.items():
            assert hashlib.sha256(zipped.read('damage-numbers/' + name)).hexdigest() == expected
            assert digest(package_dir / name) == expected
    for line in (dest / 'SHA256SUMS').read_text().splitlines():
        expected, name = line.split('  ', 1)
        assert digest(dest / name) == expected
    print('Verified candidate:', archive)


def fetch():
    import fetch_vendor;fetch_vendor.main()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['doctor', 'fetch', 'prepare', 'build', 'test', 'verify', 'package', 'export'])
    parser.add_argument('--host', action='store_true')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    if args.command == 'test':
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
