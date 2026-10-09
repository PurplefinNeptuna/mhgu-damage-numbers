# Run original game routines in Unicorn with explicit memory and call stubs.
# These fixtures check damage, HZV, camera, ownership, and feedback contracts.

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
"""Execute original MHGU damage normalization in bounded Unicorn fixtures.

Run with tools/mwc-analysis-venv/bin/python. This checks copied native bytes,
rounding and per-part contributions; it does not validate an Eden hook.
"""
from pathlib import Path
import hashlib
import json
import struct
import math
import ctypes
import subprocess
import itertools
import random
import unicorn

from unicorn import Uc, UC_ARCH_ARM, UC_MODE_ARM, UC_HOOK_CODE, UcError
from unicorn.arm_const import (UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_R2,
                               UC_ARM_REG_R3, UC_ARM_REG_SP, UC_ARM_REG_LR,
                               UC_ARM_REG_PC, UC_ARM_REG_C1_C0_2, UC_ARM_REG_FPEXC)
from unicorn import arm_const as arm

from paths import ROOT, MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
IMAGE_SHA = '24c208c395c65727b21b5fa91af8381396e70cf421b0fcfee0345d8743298110'
MONSTER, PARAM, VTABLE = 0x30000000, 0x30010000, 0x30020000
STACK, RETURN, EXTENDED = 0x30100000, 0x30200000, 0x30200004
START, END = 0x9d6f8, 0x9d8a8
REGS = (UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_R2, UC_ARM_REG_R3)
STUBS = {0x81c6c: 'sleep-state query', 0xa1fc8: 'rage-state query',
         0x3a8d68: 'quest-rank query', 0x710f0: 'additional part-table lookup',
         EXTENDED: 'extended-part virtual callback'}


def f32(n):
    return struct.unpack('<f', struct.pack('<f', n))[0]


def camera_cases(data):
    """Native look-at, perspective and multiplication; tanf is a named stub."""
    src = MOD
    lib_path = WORK_ROOT / 'tests/camera_bridge.so'
    lib_path.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', '-O2',
                    '-ffp-contract=off', '-shared', '-fPIC', '-I' + str(src / 'src'),
                    str(src / 'tests/camera_bridge.cpp'), '-o', str(lib_path)], check=True)
    lib = ctypes.CDLL(str(lib_path))
    fp = ctypes.POINTER(ctypes.c_float)
    lib.dn_camera_project.argtypes = [fp, fp, fp, fp]
    lib.dn_camera_project.restype = ctypes.c_int
    matrix, projection, combined, eye, target, up = [PARAM + i * 0x100 for i in range(6)]
    permitted = ((0x7c5e4c, 0x7c5fe8), (0x7c617c, 0x7c6274), (0x29d48, 0x29f24))
    passed = []

    def check(ok, label):
        if not ok:
            raise AssertionError(label)
        passed.append(label)

    class CameraProbe(Probe):
        def trace(self, u, address, size, _):
            if address == RETURN:
                u.emu_stop()
                return
            if address == 0x13f0184:  # native tanf import thunk, 0x01db9184 in Ghidra
                bits = u.reg_read(arm.UC_ARM_REG_S0)
                value = struct.unpack('<f', struct.pack('<I', bits))[0]
                u.reg_write(arm.UC_ARM_REG_S0, struct.unpack(
                    '<I', struct.pack('<f', math.tan(value)))[0])
                u.reg_write(UC_ARM_REG_PC, u.reg_read(UC_ARM_REG_LR))
                return
            if not any(a <= address < b for a, b in permitted):
                raise ValueError(f'Unexpected camera dependency {address:#x}')

        def run(self, entry, args=(), floats=()):
            u = self.uc
            u.reg_write(UC_ARM_REG_SP, STACK + 0xfff0)
            u.reg_write(UC_ARM_REG_LR, RETURN)
            for r, n in zip(REGS, args):
                u.reg_write(r, n)
            for i, n in enumerate(floats):
                u.reg_write(getattr(arm, 'UC_ARM_REG_S' + str(i)),
                            struct.unpack('<I', struct.pack('<f', n))[0])
            u.emu_start(entry, RETURN, count=5000)
            if u.reg_read(UC_ARM_REG_PC) != RETURN:
                raise ValueError('Camera instruction budget exceeded')
    # Center, right/up, rotation, translated camera and distinct FOV/aspect.
    for eye_values, target_values in (((0, 0, 10), (0, 0, 0)), ((10, 0, 0), (0, 0, 0)),
                                      ((10, 5, 10), (5, 5, 5))):
        for fov in (45, 70, 100):
            for aspect in (16 / 9, 4 / 3):
                p = CameraProbe(data)
                for ptr, values in ((eye, eye_values), (target, target_values), (up, (0, 1, 0))):
                    p.uc.mem_write(ptr, struct.pack('<3f', *values))
                p.run(0x7c5e4c, (matrix, eye, target, up))
                p.run(0x7c617c, (projection,), (.1, 1000, math.radians(fov), aspect))
                p.run(0x29d48, (combined, matrix, projection))
                vm = struct.unpack('<16f', p.uc.mem_read(matrix, 64))
                pm = struct.unpack('<16f', p.uc.mem_read(projection, 64))
                vp = struct.unpack('<16f', p.uc.mem_read(combined, 64))
                for values in (target_values, (target_values[0] + 1, target_values[1], target_values[2]),
                               (target_values[0], target_values[1] + 1, target_values[2]), eye_values):
                    world = tuple(values) + (1,)
                    clip = [sum(world[k] * vp[k * 4 + i] for k in range(4)) for i in range(4)]
                    x, y, z, w = clip
                    expected = (w > 0.00001 and -w <= x <= w and -w <= y <= w and 0 <= z <= w)
                    output = (ctypes.c_float * 2)()
                    actual = lib.dn_camera_project((ctypes.c_float * 16)(*vm), (ctypes.c_float * 16)(*pm),
                                                   (ctypes.c_float * 3)(*values), output)
                    check(bool(actual) == expected, 'native matrix visibility matches C++ projection')
                    if expected:
                        check(abs(output[0] - (x / w + 1) * 640) < 0.01 and abs(output[1] - (1 - y / w) * 360) < 0.01,
                              'native composed matrix matches C++ screen coordinates')
                output = (ctypes.c_float * 2)()
                actual = lib.dn_camera_project((ctypes.c_float * 16)(*vm), (ctypes.c_float * 16)(*pm),
                                               (ctypes.c_float * 3)(*target_values), output)
                check(actual and abs(output[0] - 640) < 0.01 and abs(output[1] - 360) < 0.01,
                      'native camera target appears at center')
    return {'cases': len(passed), 'passed': passed, 'bridge_sha256': hashlib.sha256(lib_path.read_bytes()).hexdigest(),
            'native_ranges': [[hex(a), hex(b)] for a, b in permitted],
            'stub': 'tanf import uses math.tan rounded to float32; no camera selection or live viewport acceptance'}


# Exercise player-table and palico-master resolution with explicit readable regions.
def ownership_cases(data):
    src = MOD
    lib_path = WORK_ROOT / 'tests/ownership_bridge.so'
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', '-O2', '-shared', '-fPIC',
                    '-I' + str(src / 'src'), str(src / 'tests/ownership_bridge.cpp'), '-o', str(lib_path)], check=True)
    lib = ctypes.CDLL(str(lib_path))
    read_type = ctypes.CFUNCTYPE(ctypes.c_uint32, ctypes.c_uint32, ctypes.c_uint32)
    lib.dn_local_player.argtypes = [read_type]
    lib.dn_local_player.restype = ctypes.c_uint32
    lib.dn_owner.argtypes = [read_type, ctypes.c_uint32]
    lib.dn_owner.restype = ctypes.c_uint32
    ranges = ((0x276278, 0x27629c), (0x3f8398, 0x3f83a0),
              (0x285c6c, 0x285cd4), (0x274ce4, 0x274cf0))
    table, session, cat, work, hit = 0x30030000, 0x30040000, 0x30060000, 0x30080000, 0x30090000
    passed = []

    def check(ok, label):
        if not ok:
            raise AssertionError(label)
        passed.append(label)

    class OwnerProbe(Probe):
        def trace(self, u, address, size, _):
            if address == RETURN:
                u.emu_stop()
                return
            if address == 0x27bcc0:  # native quest-context query: fixture is in a quest
                u.reg_write(UC_ARM_REG_R0, 1)
                u.reg_write(UC_ARM_REG_PC, u.reg_read(UC_ARM_REG_LR))
                return
            if not any(a <= address < b for a, b in ranges):
                raise ValueError(f'Unexpected owner dependency {address:#x}')

        def run(self, entry, args):
            u = self.uc
            u.reg_write(UC_ARM_REG_SP, STACK + 0xfff0)
            u.reg_write(UC_ARM_REG_LR, RETURN)
            for r, n in zip(REGS, args):
                u.reg_write(r, n)
            u.emu_start(entry, RETURN, count=5000)
            if u.reg_read(UC_ARM_REG_PC) != RETURN:
                raise ValueError('Ownership instruction budget exceeded')
            return u.reg_read(UC_ARM_REG_R0)
    p = OwnerProbe(data)

    def got(pc, literal):
        return p.read(pc + 8 + p.read(literal))
    check(got(0x276284, 0x27629c) == 0x188ca48, 'native local-session GOT cell')
    check(got(0x285ca8, 0x285cd8) == 0x1885b58, 'native palico master-player table GOT cell')
    p.write(0x188ca48, session)
    p.write(0x1885b58, table)
    players = [MONSTER + 0x5000 * i for i in range(4)]
    for i, player in enumerate(players):
        p.write(table + 0x1c + 4 * i, player)
    p.write(cat + 0x13c8, 0)
    p.write(cat + 0x3ac4, work)

    def read(a, n):
        try:
            if n & 0x80000000:
                if not a:
                    return 0
                p.uc.mem_read(a, n & 0x7fffffff)
                return 1
            return int.from_bytes(p.uc.mem_read(a, n), 'little')
        except UcError:
            return 0
    callback = read_type(read)
    for local in range(4):
        p.write(session + 0x30, local)
        check(lib.dn_local_player(callback) == p.run(0x276278, (table,)) == players[local],
              'C++ local identity agrees with native main-player getter')
        for owner, player in enumerate(players):
            for kind in (0, 3):  # includes a player-controlled Prowler
                p.write(hit + 0x10, kind, '<B')
                p.write(hit + 0xc, player)
                check(lib.dn_owner(callback, hit) == (0 if owner == local else 2),
                      'hunter/Prowler ownership across local slots')
        for master in range(4):
            p.write(work + 1, master, '<B')
            native = p.run(0x285c6c, (cat,))
            check(native == players[master], 'native palico current work pointer and master index')
            for primary in (3, 2):
                p.write(hit + 0x10, primary, '<B')
                p.write(hit + 0xc, cat if primary == 3 else PARAM)
                p.write(hit + 0x6e, 3, '<B')
                p.write(hit + 0x70, cat)
                check(lib.dn_owner(callback, hit) == (1 if native == players[local] else 3),
                      'direct/projectile palico ownership agrees with native master getter')
    for master in (4, 255):
        p.write(work + 1, master, '<B')
        check(lib.dn_owner(callback, hit) == 4, 'invalid master hidden')
    p.write(work + 1, 0, '<B')
    p.write(cat + 0x3ac4, 0)
    check(lib.dn_owner(callback, hit) == 4, 'missing current palico work hidden')
    p.write(cat + 0x3ac4, work)
    p.write(table + 0x1c, 0)
    check(lib.dn_owner(callback, hit) == 4, 'missing master hunter hidden')
    for index in (4, 255):
        p.write(session + 0x30, index)
        check(lib.dn_local_player(callback) == 0, 'invalid local-session index hidden')
    return {'cases': len(passed), 'passed': passed, 'bridge_sha256': hashlib.sha256(lib_path.read_bytes()).hexdigest(),
            'native_ranges': [[hex(a), hex(b)] for a, b in ranges],
            'stub': 'quest-context query returns true; attack snapshots are constructed fixtures, not multiplayer acceptance'}


# Compare original critical resolution with positive/negative/unknown native outcomes.
def critical_cases(data):
    hit, hurtbox, hzv, player, output = PARAM, PARAM + 0x100, PARAM + 0x200, MONSTER, PARAM + 0x300
    skills_query, skill_effect, affinity_mode = 0x2864c4, 0x2864e0, 0x27c90c
    permitted = (0x17885c, 0x178afc)
    passed = []

    def check(ok, label):
        if not ok:
            raise AssertionError(label)
        passed.append(label)

    class CriticalProbe(Probe):
        def trace(self, u, address, size, _):
            if address == RETURN:
                u.emu_stop()
                return
            if address in (0xbab60, skills_query, skill_effect, affinity_mode, 0x4a1b38):
                value = {0xbab60: hzv, skills_query: int(u.reg_read(UC_ARM_REG_R1) in self.skills),
                         skill_effect: 0, affinity_mode: 0, 0x4a1b38: 0}.get(address)
                u.reg_write(UC_ARM_REG_R0, value)
                u.reg_write(UC_ARM_REG_PC, u.reg_read(UC_ARM_REG_LR))
                return
            if not permitted[0] <= address < permitted[1]:
                raise ValueError(f'Unexpected critical dependency {address:#x}')

        def resolve(self, affinity, rng, zone=44, skills=(), flags=0x20):
            self.skills = set(skills)
            self.write(hit + 0x30, 0x20000 | flags)
            self.write(hit + 0x90, affinity, '<h')
            self.write(hit + 0x92, rng, '<H')
            self.write(hit + 0x54, 0, '<B')
            self.write(hit + 0x10, 0, '<B')
            self.write(hit + 0xc, player)
            self.uc.mem_write(hzv, bytes((zone, zone, zone)))
            self.write(player + 0x2b70, 100)  # native cut/blunt weights 100/0
            self.write(output, 1.0, '<f')
            u = self.uc
            sp = STACK + 0xfff0
            self.write(sp, player)
            self.write(sp + 4, output)
            u.reg_write(UC_ARM_REG_SP, sp)
            u.reg_write(UC_ARM_REG_LR, RETURN)
            for r, n in zip(REGS, (0, hit, MONSTER, hurtbox)):
                u.reg_write(r, n)
            try:
                u.emu_start(permitted[0], RETURN, count=5000)
            except UcError as error:
                pc = u.reg_read(UC_ARM_REG_PC)
                raise RuntimeError(
                    f'Critical {affinity}/{rng}: {error} at {pc:#x}, bytes={bytes(u.mem_read(pc, 4)).hex()}') from error
            if u.reg_read(UC_ARM_REG_PC) != RETURN:
                raise ValueError('Critical instruction budget exceeded')
            check(self.read(hit + 0x92, '<H') == rng, 'stored RNG remains unchanged')
            return self.read(hit + 0x54, '<B'), self.read(output, '<f')
    p = CriticalProbe(data)
    for affinity in (-100, -50, -1, 0, 1, 25, 50, 100):
        for rng in (0, 1, 24, 25, 49, 50, 99, 65535):
            flag, modifier = p.resolve(affinity, rng)
            expected = 2 if affinity > 0 and rng % 100 < affinity else 1 if affinity < 0 and rng % 100 < -affinity else 0
            check(flag == expected, 'native critical roll and sign')
            check(modifier == {0: 1.0, 1: .75, 2: 1.25}[expected], 'native critical multiplier')
    for zone in (44, 45):
        for flags in (0x20, 1):
            flag, modifier = p.resolve(0, 49, zone=zone, skills=(0x92,), flags=flags)
            check(flag == (2 if zone == 45 else 0), 'Weakness Exploit physical HZV boundary')
    flag, modifier = p.resolve(100, 99, skills=(0x96,))
    check(flag == 2 and abs(modifier - 1.4) < 0.000001, 'Critical Boost native positive outcome')
    for rng in range(100):
        flag, modifier = p.resolve(-100, rng, skills=(0x97,))
        swapped = ((rng & 0x3333) << 2 | (rng & 0xcccc) >> 2) % 100
        beneficial = swapped <= 24
        check(flag == (2 if beneficial else 1), 'Bitter Affinity native outcome classification')
        check(modifier == (2.0 if beneficial else .75), 'Bitter Affinity native multiplier')
    return {'cases': len(passed), 'passed': passed, 'native_range': [hex(n) for n in permitted],
            'stub': 'HZV table and skill queries return explicit fixtures; special affinity-mode query disabled; no live crit capture'}


def feedback_cases(data):
    """Original two-attack feedback getter; not per-impact bounce attribution."""
    src = MOD
    lib_path = WORK_ROOT / 'tests/feedback_bridge.so'
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', '-O2',
                    '-shared', '-fPIC', '-I' + str(src / 'src'),
                    str(src / 'tests/feedback_bridge.cpp'), '-o', str(lib_path)], check=True)
    lib = ctypes.CDLL(str(lib_path))
    lib.dn_feedback.argtypes = [ctypes.POINTER(ctypes.c_uint32)]
    lib.dn_feedback.restype = ctypes.c_uint32
    status_stub, stance_stub, move_stub = RETURN + 0x10, RETURN + 0x14, RETURN + 0x18
    skill_ids = (0x57, 0x12f, 0x116, 0x128, 0x58)
    passed = []

    def check(ok, label):
        if not ok:
            raise AssertionError(label)
        passed.append(label)

    class FeedbackProbe(Probe):
        def trace(self, u, address, size, _):
            if address == RETURN:
                u.emu_stop()
                return
            args = [u.reg_read(r) for r in REGS]
            if address == 0xdf814:
                if args[:2] != [PARAM + 4, 0]:
                    raise ValueError('Unexpected status accessor ABI')
                result = PARAM + 0x100
            elif address == 0x2864c4:
                if args[0] != MONSTER or args[1] not in skill_ids:
                    raise ValueError('Unexpected feedback skill ABI')
                result = int(bool(self.values[6] & (1 << skill_ids.index(args[1]))))
            elif address == 0xdd4c4:
                if args[:2] != [PARAM + 0x100, 0x16]:
                    raise ValueError('Unexpected status query ABI')
                result = self.values[8]
            elif address == status_stub:
                if args[:2] != [MONSTER, 2]:
                    raise ValueError('Unexpected status virtual ABI')
                result = self.values[9]
            elif address == stance_stub:
                if args[:2] != [MONSTER, 1]:
                    raise ValueError('Unexpected stance virtual ABI')
                result = self.values[10]
            elif address == move_stub:
                if args[:2] != [MONSTER, move_stub]:
                    raise ValueError('Unexpected move virtual ABI')
                if args[2:] == [8, 0]:
                    result = self.values[12]
                elif args[2:] == [0, 14]:
                    result = self.values[13]
                else:
                    raise ValueError('Unexpected move query arguments')
            elif 0x2a87a8 <= address < 0x2a8a50:
                return
            else:
                raise ValueError(f'Unexpected feedback dependency {address:#x}')
            u.reg_write(UC_ARM_REG_R0, result)
            u.reg_write(UC_ARM_REG_PC, u.reg_read(UC_ARM_REG_LR))

        def resolve(self, values):
            self.values = values
            for active, effect, flags, base in ((values[0], values[1], values[2], MONSTER + 0x3020),
                                                (values[3], values[4], values[5], MONSTER + 0x30d8)):
                self.write(base + 5, active, '<B')
                self.write(base + 6, effect, '<B')
                self.write(base + 0x30, flags)
            self.write(MONSTER + 0x2522, values[7], '<H')
            self.write(MONSTER + 0x29dc, values[11], '<B')
            before = bytes(self.uc.mem_read(MONSTER, 0x4000))
            sp = STACK + 0xfff0
            self.uc.reg_write(UC_ARM_REG_SP, sp)
            self.uc.reg_write(UC_ARM_REG_LR, RETURN)
            self.uc.reg_write(UC_ARM_REG_R0, MONSTER)
            saved = [(getattr(arm, 'UC_ARM_REG_R' + str(i)), 0x87654000 + i) for i in range(4, 12)]
            for reg, value in saved:
                self.uc.reg_write(reg, value)
            self.uc.emu_start(0x2a87a8, RETURN, count=5000)
            if self.uc.reg_read(UC_ARM_REG_PC) != RETURN:
                raise ValueError('Feedback instruction budget exceeded')
            check(self.uc.reg_read(UC_ARM_REG_SP) == sp and
                  all(self.uc.reg_read(r) == v for r, v in saved), 'native feedback preserves stack and callee-saved registers')
            check(bytes(self.uc.mem_read(MONSTER, 0x4000)) == before,
                  'native feedback does not mutate fixture player memory')
            return self.uc.reg_read(UC_ARM_REG_R0)
    p = FeedbackProbe(data)
    p.write(MONSTER + 0x13c4, PARAM)
    for slot, stub in ((0x218, status_stub), (0x208, stance_stub), (0x1b8, move_stub)):
        p.write(VTABLE + slot, stub)
    flags = (0, 1 << 18, 1 << 27, (1 << 18) | (1 << 27))
    cases = []
    for first, second, f1, f2 in itertools.product((0, 2, 3, 4, 255), (0, 2, 3, 4, 255), flags, flags):
        cases.append((1, first, f1, 1, second, f2, 0, 0, 0, 0, 0, 0, 0, 0))
    for skills, flag, counter in itertools.product(range(32), flags, (0, 1, 2, 3, 65535)):
        cases.append((1, 0, flag, 0, 255, 0, skills, counter, 0, 0, 0, 0, 0, 0))
        cases.append((1, 2, flag, 0, 255, 0, skills, counter, 0, 0, 0, 0, 0, 0))
    for flag, status, prevent, stance, level, move8, move14 in itertools.product(
            flags, (0, 1, 2), (0, 1, 2), (0, 1, 2), (0, 1, 2), (0, 1), (0, 1)):
        cases.append((1, 0, flag, 0, 255, 0, 0, 0, status, prevent, stance, level, move8, move14))
    rng = random.Random(0x2a87a8)
    for _ in range(512):
        cases.append((rng.randrange(2), rng.choice((0, 1, 2, 3, 4, 255)), rng.getrandbits(32),
                      rng.randrange(2), rng.choice((0, 1, 2, 3, 4, 255)), rng.getrandbits(32),
                      rng.randrange(32), rng.randrange(65536), rng.randrange(3), rng.randrange(3),
                      rng.randrange(3), rng.randrange(256), rng.randrange(3), rng.randrange(3)))
    for values in cases:
        native = p.resolve(values)
        actual = lib.dn_feedback((ctypes.c_uint32 * 14)(*values))
        check(native == actual,
              f'C++ feedback agrees with original getter: {values}, native {native}, mirror {actual}')
    return {'cases': len(passed), 'native_calls': len(cases), 'passed': passed,
            'native_range': ['0x2a87a8', '0x2a8a50'], 'skills': [hex(s) for s in skill_ids],
            'bridge_sha256': hashlib.sha256(lib_path.read_bytes()).hexdigest(),
            'stub': 'Status accessor, skill/status queries and three virtual services use explicit fixtures',
            'limit': 'The native getter combines both active attacks; per-impact bounce correlation remains unvalidated'}


# Map the pinned game image and synthetic state, then intercept named dependencies.
class Probe:
    def __init__(self, data, defense=1.0, asleep=False, already_woken=False,
                 raging=False, rank=0, additional=False, monster_kind=0):
        self.uc = u = Uc(UC_ARCH_ARM, UC_MODE_ARM)
        u.ctl_set_cpu_model(arm.UC_CPU_ARM_MAX)  # includes ARMv8 AArch32 FP select instructions
        u.mem_map(0, 0x2000000)
        u.mem_write(0, data)
        u.mem_map(0x30000000, 0x210000)
        u.reg_write(UC_ARM_REG_C1_C0_2, 0xf00000)
        u.reg_write(UC_ARM_REG_FPEXC, 0x40000000)
        self.write(MONSTER, VTABLE)
        self.write(MONSTER + 0x1428, PARAM)
        self.write(VTABLE + 0x310, EXTENDED)
        self.write(PARAM + 0x388, defense, '<f')
        self.write(PARAM + 0x4e0, int(already_woken))
        self.write(PARAM + 0x484, monster_kind, '<B')
        self.asleep, self.raging, self.rank, self.additional = asleep, raging, rank, additional
        self.calls = []
        u.hook_add(UC_HOOK_CODE, self.trace)

    def write(self, a, n, fmt='<I'):
        self.uc.mem_write(a, struct.pack(fmt, n))

    def read(self, a, fmt='<I'):
        return struct.unpack(fmt, self.uc.mem_read(a, struct.calcsize(fmt)))[0]

    def trace(self, u, address, size, _):
        if address == RETURN:
            u.emu_stop()
            return
        if address in STUBS:
            args = [u.reg_read(r) for r in REGS]
            self.calls.append((address, args))
            result = {0x81c6c: int(self.asleep), 0xa1fc8: int(self.raging),
                      0x3a8d68: self.rank, 0x710f0: 0 if self.additional else 0xffffffff,
                      EXTENDED: 0}.get(address)
            u.reg_write(UC_ARM_REG_R0, result)
            u.reg_write(UC_ARM_REG_PC, u.reg_read(UC_ARM_REG_LR))
            return
        if not START <= address < END:
            raise ValueError(f'Unexpected native dependency {address:#x}')

    # Invoke the original finalized contribution routine with fixture parameters.
    def damage(self, amount, mode=1, part=0):
        u = self.uc
        u.reg_write(UC_ARM_REG_SP, STACK + 0xfff0)
        u.reg_write(UC_ARM_REG_LR, RETURN)
        for reg, value in zip(REGS, (MONSTER, part, amount & 0xffffffff, mode)):
            u.reg_write(reg, value)
        u.emu_start(START, RETURN, count=5000)
        if u.reg_read(UC_ARM_REG_PC) != RETURN:
            raise ValueError('Native instruction budget exceeded')
        return struct.unpack('<i', struct.pack('<I', u.reg_read(UC_ARM_REG_R0)))[0]


# Run native/helper comparisons and record case counts with their input fingerprints.
def main():
    if unicorn.__version__ != '2.1.4':
        raise ValueError('Use the pinned Unicorn 2.1.4 analysis environment')
    data = (WORK_ROOT / 'inputs/main-memory.bin').read_bytes()
    if hashlib.sha256(data).hexdigest() != IMAGE_SHA:
        raise ValueError('Native image identity differs')
    results = []

    def check(ok, label):
        if not ok:
            raise AssertionError(label)
        results.append(label)
    for defense in (0.0, 0.5, 0.75, 1.0, 1.25):
        for amount in (-123, 0, 1, 2, 17, 123, 1000):
            for part in (0, 3, 7):
                p = Probe(data, defense=defense)
                expected = max(1, int(f32(amount * defense))) if amount > 0 else amount
                p.write(PARAM + 0x488 + 2 * part, 11, '<h')
                actual = p.damage(amount, part=part)
                check(actual == expected, f'normalized return {defense}/{amount}/{part}')
                check(p.read(PARAM + 0x488 + 2 * part, '<h') ==
                      11 + expected, 'part accumulator contribution')
                check(all(p.read(PARAM + 0x488 + 2 * i, '<h') ==
                      0 for i in range(8) if i != part), 'other parts untouched')
                check(p.read(PARAM + 0x370) == 0, 'normalization does not prematurely mutate HP')
    for mode, multiplier in ((0, 3), (1, 2)):
        p = Probe(data, defense=.75, asleep=True)
        check(p.damage(101, mode) == int(101 * .75 * multiplier), 'native first sleep multiplier')
        check(p.read(PARAM + 0x4e0) & 1, 'sleep first-hit flag consumed')
        check(p.damage(101, mode) == int(101 * .75), 'second simultaneous sleep impact unboosted')
    p = Probe(data, asleep=True, monster_kind=ord('J'))
    check(p.damage(100) == 100, 'special native monster-kind sleep exception')
    p = Probe(data, additional=True)
    check(p.damage(75, part=2) == 75, 'additional part-table return')
    check(p.read(PARAM + 0x498 + 4, '<h') == 150, 'both native additional-table lookups applied')
    p = Probe(data)
    check(p.damage(123, mode=2) == 10000, 'native special damage mode')
    check(p.read(PARAM + 0x488, '<h') == 10000,
          'special damage enters the HP contribution accumulator')
    p = Probe(data)
    check(p.damage(-50) == -50, 'healing remains negative for capture rejection')
    check(p.read(PARAM + 0x488, '<h') == -50, 'healing native behavior preserved')
    p = Probe(data)
    check(p.damage(123, part=8) == 0, 'extended part returns zero on generic path')
    check(any(a == EXTENDED and args[:3] == [MONSTER, 8, 123]
          for a, args in p.calls), 'extended native virtual dispatch')
    check(p.read(PARAM + 0x488, '<h') == 0, 'extended path has no generic accumulator')
    # Rage constants are read from the original literal pool, not copied from a wiki.
    low = struct.unpack_from('<f', data, 0x9d8b0)[0]
    high = struct.unpack_from('<f', data, 0x9d8ac)[0]
    for rank, modifier in ((0, low), (1, high)):
        p = Probe(data, raging=True, rank=rank)
        check(p.damage(1000) == int(f32(1000 * modifier)), f'native rage rank modifier {rank}')
    out = ROOT / 'build/damage-numbers'
    out.mkdir(parents=True, exist_ok=True)
    camera = camera_cases(data)
    ownership = ownership_cases(data)
    critical = critical_cases(data)
    feedback = feedback_cases(data)
    report = {'kind': 'original ARM32 routines in explicit Unicorn fixtures; not Eden',
              'image_sha256': IMAGE_SHA, 'engine': 'Unicorn 2.1.4, ARM_MAX CPU model',
              'native_range': [hex(START), hex(END)], 'stubbed_services': STUBS,
              'cases': len(results), 'passed': results,
              'camera': camera,
              'ownership': ownership,
              'critical': critical,
              'feedback': feedback,
              'limits': ['No live collision, HZV, ownership, camera or runtime-hook acceptance',
                         'Extended monster parts require a separately verified adapter',
                         'Sleep/rage/quest/table services use explicit fixture return values']}
    (out / 'native-probe.json').write_text(json.dumps(report, indent=2) + '\n')
    print(
        f'Passed {len(results)} damage, {camera["cases"]} camera, {ownership["cases"]} ownership, {critical["cases"]} critical and {feedback["cases"]} feedback checks in original ARM32 fixtures')


if __name__ == '__main__':
    main()
