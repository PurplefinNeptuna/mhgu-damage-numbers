#!/usr/bin/env python3
"""Execute MHGU's original crop call/stores with an NVN double; never launch the game."""
import json
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from paths import BUILD_ROOT, input_path
from mhgu_binary import main_image
from unicorn import Uc, UC_ARCH_ARM, UC_MODE_ARM, UC_HOOK_CODE
from unicorn import arm_const as arm

image, _ = main_image(input_path('main').read_bytes())
cpu = Uc(UC_ARCH_ARM, UC_MODE_ARM)
cpu.ctl_set_cpu_model(arm.UC_CPU_ARM_MAX)
cpu.mem_map(0, (max(len(image), 0x1920000) + 4095) & ~4095)
cpu.mem_write(0, image)
engine, renderer, stack = 0x10000000, 0x20000000, 0x30010000
cpu.mem_map(engine, 4096)
cpu.mem_write(engine, struct.pack('<I', 0xe12fff1e))  # bx lr: NVN SetCrop double.
cpu.mem_map(renderer, 0x450000)
cpu.mem_map(stack - 0x10000, 0x20000)
cpu.mem_write(0x191f808, struct.pack('<I', engine))
cpu.mem_write(renderer + 0x98, struct.pack('<I', renderer + 0x38a98))
cpu.mem_write(renderer + 0x90, struct.pack('<I', 0x12345678))
calls = []

def trace(cpu, address, _size, _data):
    if address == engine:
        # Native SetCrop takes window,x,y,width in r0-r3 and height on the stack.
        args = [cpu.reg_read(r) for r in [arm.UC_ARM_REG_R0, arm.UC_ARM_REG_R1,
                                        arm.UC_ARM_REG_R2, arm.UC_ARM_REG_R3]]
        args.append(struct.unpack('<I', cpu.mem_read(cpu.reg_read(arm.UC_ARM_REG_SP), 4))[0])
        calls.append(args)

cpu.hook_add(UC_HOOK_CODE, trace)
# Reuse one native renderer across both mode changes and intermediate dimensions.
sizes = [(1280, 720), (1920, 1080), (1280, 720), (1600, 900)]
for width, height in sizes:
    cpu.mem_write(renderer + 0xb0, struct.pack('<2I', width, height))
    cpu.mem_write(renderer + 0x3bafa, b'\x01')
    cpu.reg_write(arm.UC_ARM_REG_R4, renderer)
    cpu.reg_write(arm.UC_ARM_REG_R5, renderer + 0x34100)
    cpu.reg_write(arm.UC_ARM_REG_R6, renderer + 0x3bafa)
    cpu.reg_write(arm.UC_ARM_REG_SP, stack)
    cpu.emu_start(0xbc0b74, 0xbc0bac, count=1000)
    assert calls[-1] == [renderer + 0x38840, 0, 0, width, height], calls[-1]
    assert struct.unpack('<2I', cpu.mem_read(renderer + 0xb8, 8)) == (width, height)
    assert cpu.mem_read(renderer + 0x3bafa, 1) == b'\x00'
    assert cpu.reg_read(arm.UC_ARM_REG_SP) == stack
assert len(calls) == len(sizes)
report = {'status': 'PASSED', 'original_arm': True, 'mode_sequence': sizes,
          'verified': ['SetCrop origin and dimensions', 'applied +0xb8/+0xbc stores',
                       'dirty flag clear', 'native stack balance'], 'runtime_test': 'NOT RUN'}
BUILD_ROOT.mkdir(parents=True, exist_ok=True)
(BUILD_ROOT / 'viewport.json').write_text(json.dumps(report, indent=2) + '\n')
print('PASS: original ARM crop call and applied display-size stores across four mode/size changes')
