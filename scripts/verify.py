#!/usr/bin/env python3
"""Check staged ARM32 code against exact game windows, native exports and compiler-derived layouts."""
import hashlib
import json
import re
import struct
import subprocess
from pathlib import Path
from paths import MOD, BUILD_ROOT, WORK_ROOT, CARGO_TARGET, CONFIG, input_path
from mhgu_binary import Nro, main_image
from native_support import nso_exports
from build import doctor, digest


def symbols(compiler, elf):
    result = {}
    for line in subprocess.check_output([compiler / 'arm-none-eabi-nm', '-n', elf], text=True).splitlines():
        fields = line.split()
        if len(fields) == 3:
            result[fields[2]] = int(fields[0], 16)
    return result


def native_layouts(compiler):
    # Compile the pinned native declarations instead of reconstructing their layouts by hand.
    jit = re.search(r'typedef enum \{.*?\} Jit;', (MOD / 'research/native-jit.h').read_text(), re.S).group(0)
    hook = re.search(r'typedef struct Hook \{.*?\} Hook;', (MOD / 'research/native-hook.h').read_text(), re.S).group(0)
    expressions = [
        'sizeof(Hook)', 'offsetof(Hook,context)', 'sizeof(damage_numbers::RawFrame)',
        *[f'offsetof(damage_numbers::RawFrame,{field})' for field in ['cpsr', 'fpscr', 'high_d', 'low_d', 'r']],
        *[f'offsetof(ImGuiIO,{field})' for field in ['Fonts', 'BackendFlags']],
        'offsetof(ImFontConfig,FontDataOwnedByAtlas)', 'offsetof(ImFontBaked,Size)',
        'sizeof(ImFontGlyph)', 'offsetof(ImFontGlyph,AdvanceX)', 'offsetof(ImFontGlyph,X0)',
        'sizeof(ImTextureData)',
        *[f'offsetof(ImTextureData,{field})' for field in ['Pixels', 'Status', 'RefCount']],
        'sizeof(ImGuiContextHook)', 'offsetof(ImGuiContextHook,Callback)', 'offsetof(ImGuiContextHook,UserData)',
        'sizeof(ImDrawVert)', 'offsetof(ImDrawList,VtxBuffer)',
    ]
    folder = WORK_ROOT / 'abi'
    folder.mkdir(parents=True, exist_ok=True)
    source = folder / 'native.cpp'
    source.write_text('#include <stdarg.h>\n#include <stdint.h>\n#include <stddef.h>\n'
                      'typedef uint32_t Handle;struct VirtmemReservation;\n' + jit + '\n' + hook + '\n'
                      '#define CIMGUI_DEFINE_ENUMS_AND_STRUCTS\n#include <cimgui.h>\n#include "gateway.h"\n'
                      'static_assert(sizeof(Jit)==28);\n'
                      'extern "C" __attribute__((used,section(".dn_native")))const uint32_t native[]={'
                      + ','.join(expressions) + '};\n')
    subprocess.run([compiler / 'arm-none-eabi-g++', '-std=c++17', '-march=armv8-a', '-marm', '-mfloat-abi=soft',
                    '-I' + str(MOD / 'vendor/cimgui'), '-I' + str(MOD / 'tests/oracle'),
                    '-c', source, '-o', folder / 'native.o'], check=True)
    subprocess.run([compiler / 'arm-none-eabi-objcopy', '-O', 'binary', '--only-section=.dn_native',
                    folder / 'native.o', folder / 'native.bin'], check=True)
    return expressions, (folder / 'native.bin').read_bytes()


def verify():
    doctor()
    package = BUILD_ROOT / 'package/romfs/nativeNX/plugins'
    data = (package / 'damage_numbers_rs.nro').read_bytes()
    nro = Nro(data)
    exports = {name.decode() for name, entry, *_ in nro.symbols if name and entry[5] and entry[3] >> 4 == 1}
    contract = (MOD / 'link/exported.txt').read_text().split('global:')[1].split('local:')[0]
    expected = set(re.findall(r'\b(\w+)\s*;', contract))
    if exports != expected:
        raise ValueError('Callback exports changed: ' + str(exports))
    imports = {name.decode() for name, entry, *_ in nro.symbols if name and not entry[5] and entry[3] >> 4 == 1}
    available, inputs = set(), {}
    for name in ['main', 'sdk', 'forge']:
        path = input_path(name)
        inputs[name] = digest(path)
        available |= nso_exports(path)
    if imports - available:
        raise ValueError('Missing native imports: ' + str(imports - available))
    lock = json.loads((MOD / 'dependencies.lock.json').read_text())
    for name, expected in lock['inputs'].items():
        if digest(input_path(name)) != expected:
            raise ValueError('Native input hash changed: ' + name)
    image, _ = main_image(input_path('main').read_bytes())
    windows = []
    for offset, raw in re.findall(r'\(\s*0x([0-9a-f]+),\s*\[([^\]]+)\]', (MOD / 'src/signatures.rs').read_text()):
        address = int(offset, 16)
        expected = bytes(int(value.strip(), 16) for value in raw.split(',') if value.strip())
        if image[address:address + 64] != expected:
            raise ValueError('Game signature mismatch: ' + offset)
        windows.append(hex(address))
    if len(windows) != 11:
        raise ValueError('Signature count changed')
    if digest(MOD / 'src/gateway.S') != lock['gateway_sha256']:
        raise ValueError('Audited gateway source changed')
    compiler = Path(CONFIG['devkitarm']) / 'bin'
    elf = CARGO_TARGET / 'armv7a-none-eabi/release/libdamage_numbers_rs.so'
    names = symbols(compiler, elf)
    for name in [*[f'dn_gateway_{site}' for site in range(8)], 'dn_gateway_after']:
        address = names[name]
        words = struct.unpack('<25I', data[address:address + 100])
        if words.count(0xe128f002) != 1 or 0xe12cf002 in words or 0xe124f002 in words:
            raise ValueError('Incorrect NZCVQ gateway restore: ' + name)
    disassembly = subprocess.check_output([compiler / 'arm-none-eabi-objdump', '-d', elf], text=True)
    msr = [line for line in disassembly.splitlines() if '\tmsr\t' in line]
    if len(msr) != 9 or any('CPSR_f, r2' not in line for line in msr):
        raise ValueError('Unexpected status-register write in compiled plugin')
    expressions, native = native_layouts(compiler)
    address = names['dn_abi']
    actual = data[address:address + len(native)]
    if actual != native:
        raise ValueError('Rust/native ARM32 layout mismatch: ' + str((struct.unpack('<24I', actual), struct.unpack('<24I', native))))
    build = json.loads((BUILD_ROOT / 'build.json').read_text())
    if digest(package / 'damage_numbers_rs.nro') != build['nro_sha256'] or digest(package / 'damage-numbers-rs/wilds-digits.ttf') != build['font_sha256']:
        raise ValueError('Staged bytes differ from build receipt')
    report = {'status': 'PASSED', 'nro_sha256': digest(package / 'damage_numbers_rs.nro'), 'elf_sha256': digest(elf),
              'exports': sorted(exports), 'strong_imports': sorted(imports), 'native_inputs': inputs,
              'signature_windows': windows, 'layouts': dict(zip(expressions, struct.unpack('<24I', native))),
              'gateway_sha256': lock['gateway_sha256'], 'nzcvq_restores': 9, 'compiled_status_writes': msr,
              'nro_bytes': len(data), 'bss_bytes': struct.unpack_from('<I', data, 56)[0],
              'ordinary_hook_jit_bytes': 8 * 4096, 'ordinary_hook_code_memory_objects': 8, 'runtime_test': 'NOT RUN'}
    (BUILD_ROOT / 'verify.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f'PASS: ARM32 layouts, {len(imports)} resolved imports, callbacks, 11 game windows and nine NZCVQ restores')


if __name__ == '__main__':
    verify()
