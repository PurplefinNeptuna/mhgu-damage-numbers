from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
#!/usr/bin/env python3
"""Build the ARM32 gateway fixture offline; no NRO, installation or game writes."""
from pathlib import Path
import hashlib
import json
import os
import subprocess

import mhgu_binary as baseline

from paths import ROOT, MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
OUT = ROOT/'work/damage-numbers/gateway'

def main():
    OUT.mkdir(parents=True, exist_ok=True)
    command = """set -eu
export PATH=/opt/devkitpro/devkitARM/bin:$PATH
arm-none-eabi-gcc --version > /output/compiler.txt
arm-none-eabi-gcc -marm -march=armv8-a+crc+crypto -mfpu=crypto-neon-fp-armv8 -mfloat-abi=softfp -c /mod/src/gateway.S -o /output/gateway.o
arm-none-eabi-gcc -marm -march=armv8-a+crc+crypto -mfpu=crypto-neon-fp-armv8 -mfloat-abi=softfp -c /mod/tests/gateway_probe.S -o /output/probe.o
arm-none-eabi-ld -Ttext=0x31000000 --entry=dn_gateway_6 --defsym=dn_gateway_enter=0x30200100 --defsym=dn_gateway_leave=0x30200104 --defsym=dn_gateway_tap=0x30200108 /output/gateway.o /output/probe.o -o /output/gateway.elf
arm-none-eabi-objcopy -O binary --only-section=.text /output/gateway.elf /output/gateway.bin
arm-none-eabi-nm -n /output/gateway.elf > /output/symbols.txt
arm-none-eabi-objdump -d /output/gateway.elf > /output/disassembly.txt
"""
    args = ['podman', '--root', CONTAINER_STORAGE, '--runroot',
            f'/tmp/mhgu-forge-container-{os.getuid()}', '--storage-driver', 'vfs', 'run', '--rm',
            '--network=none', '-v', f'{MOD}:/mod:ro', '-v', f'{ROOT}:/workspace:ro', '-v', f'{OUT}:/output:rw',
            baseline.IMAGE, 'bash', '-lc', command.replace('{diagnostics}', os.environ.get('MHGU_DIAGNOSTICS', '0'))]
    result = subprocess.run(list(map(str, args)), capture_output=True, text=True)
    (OUT/'build.log').write_text(result.stdout+result.stderr)
    if result.returncode:
        raise RuntimeError(f'ARM32 gateway fixture build failed; inspect {OUT / "build.log"}')
    names = {}
    for line in (OUT/'symbols.txt').read_text().splitlines():
        columns = line.split()
        if len(columns) == 3 and columns[2].startswith('dn_'):
            names[columns[2]] = int(columns[0], 16)
    sources = [MOD / 'src/gateway.S', MOD / 'tests/gateway_probe.S',
               SCRIPTS / 'damage_numbers_gateway_probe.py', Path(__file__)]
    report = {'kind': 'ARM32 gateway fixture only; no NRO or package', 'base': 0x31000000,
              'container_image': baseline.IMAGE, 'symbols': names,
              'apsr_restore':'NZCVQ only; GE remains as left by helper',
              'binary_sha256': hashlib.sha256((OUT/'gateway.bin').read_bytes()).hexdigest(),
              'sources': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sources}}
    (OUT/'manifest.json').write_text(json.dumps(report, indent=2)+'\n')
    print(f'Built ARM32 gateway fixture in {OUT}; installed packages untouched')

if __name__ == '__main__':
    main()
