from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
#!/usr/bin/env python3
"""Exercise compiled ARM32 gateways with hostile callbacks and native damage.

Use the pinned mwc-analysis-venv. This is an emulator fixture, not Eden.
"""
from pathlib import Path
import hashlib
import json
import random
import struct
import itertools
import ctypes
import subprocess
import unicorn
from unicorn import UcError
from unicorn import arm_const as arm
from damage_numbers_arm_probe import Probe, ROOT, IMAGE_SHA, STACK, RETURN, START, END, STUBS, f32

WORK = ROOT/'work/damage-numbers/gateway'
NZCVQ_MASK = 0xf8000000
GE_MASK = 0x000f0000
FLAG_MASK = NZCVQ_MASK | GE_MASK
REGISTERS = [getattr(arm, 'UC_ARM_REG_R'+str(i)) for i in range(13)] + [arm.UC_ARM_REG_LR, arm.UC_ARM_REG_SP]
DS = [getattr(arm, 'UC_ARM_REG_D'+str(i)) for i in range(32)]

class GatewayProbe(Probe):
    def __init__(self, image, code, names, hooked=False, capacity=64, damage=False, callback_ge=10):
        super().__init__(image, defense=.75)
        self.names, self.capacity = names, capacity
        self.scope, self.entries, self.exits, self.taps, self.overflow = [], [], [], [], 0
        self.damage_hook = damage
        self.callback_ge = callback_ge
        self.uc.mem_map(0x31000000, 0x10000); self.uc.mem_write(0x31000000, code)
        recursive = names['dn_probe_recursive']
        self.originals = {6: names['dn_probe_leaf'], 7: 0x31004000,
                          1: 0x31004010, 4: names['dn_probe_leaf'], 5: names['dn_probe_leaf']}
        for target, trampoline, gateway in ((recursive, 0x31004000, names['dn_gateway_7']),
                                            (START, 0x31004010, names['dn_gateway_1'])):
            displaced = bytes(self.uc.mem_read(target, 8))
            self.uc.mem_write(trampoline, displaced+struct.pack('<II', 0xe51ff004, target+8))
            if hooked and (target == recursive or damage):
                self.uc.mem_write(target, struct.pack('<II', 0xe51ff004, gateway))
    def frame(self, ptr):
        return bytes(self.uc.mem_read(ptr, 328))
    def clobber_callback(self, return_pc, result):
        # The callback is allowed to use floating point and change flags. Even
        # callee-saved registers are destroyed here to stress the full frame.
        for i, r in enumerate(REGISTERS[:-1]): self.uc.reg_write(r, 0xdeed0000+i)
        for i, r in enumerate(DS): self.uc.reg_write(r, 0x7ff0000012340000+i)
        self.uc.reg_write(arm.UC_ARM_REG_FPSCR, 0xa1000000)
        cpsr = self.uc.reg_read(arm.UC_ARM_REG_CPSR)
        ge = cpsr & GE_MASK if self.callback_ge is None else self.callback_ge << 16
        self.uc.reg_write(arm.UC_ARM_REG_CPSR, (cpsr & ~FLAG_MASK) | 0x58000000 | ge)
        self.uc.reg_write(arm.UC_ARM_REG_R0, result)
        self.uc.reg_write(arm.UC_ARM_REG_PC, return_pc)
    def trace(self, u, address, size, _):
        if address == RETURN:
            u.emu_stop(); return
        if address in (self.names['dn_gateway_enter'], self.names['dn_gateway_tap']):
            kind, ptr = u.reg_read(arm.UC_ARM_REG_R0), u.reg_read(arm.UC_ARM_REG_R1)
            raw = self.frame(ptr)
            if ptr & 7: raise ValueError('Gateway callback stack is not aligned')
            self.entries.append((kind, ptr, raw))
            ret = u.reg_read(arm.UC_ARM_REG_LR)
            if address == self.names['dn_gateway_tap']:
                self.taps.append(kind)
            elif len(self.scope) < self.capacity:
                caller = struct.unpack_from('<I', raw, 324)[0]
                self.scope.append((kind, caller, ptr+328))
                self.write(ptr+324, self.names['dn_gateway_after'])
            else:
                self.overflow += 1
            self.clobber_callback(ret, self.originals[kind]); return
        if address == self.names['dn_gateway_leave']:
            ptr = u.reg_read(arm.UC_ARM_REG_R0)
            raw = self.frame(ptr)
            if not self.scope: raise ValueError('After gateway has no scope')
            kind, caller, original_sp = self.scope.pop()
            if ptr+328 != original_sp: raise ValueError('Scope popped at a different native stack position')
            self.exits.append((kind, ptr, raw))
            # A leaf retains its incoming lr. Replace the synthetic return lr
            # in that case; preserve any lr the native routine computed itself.
            if struct.unpack_from('<I', raw, 324)[0] == self.names['dn_gateway_after']:
                self.write(ptr+324, caller)
            ret = u.reg_read(arm.UC_ARM_REG_LR)
            self.clobber_callback(ret, caller); return
        if 0x31000000 <= address < 0x31010000:
            return
        if self.damage_hook and (START <= address < END or address in STUBS):
            super().trace(u, address, size, _); return
        raise ValueError(f'Unexpected gateway dependency {address:#x}')
    def execute(self, entry, values, doubles, flags, fpscr, stack_args):
        self.scope.clear(); self.entries.clear(); self.exits.clear(); self.taps.clear(); self.overflow = 0
        for reg, value in zip(REGISTERS, values): self.uc.reg_write(reg, value)
        for reg, value in zip(DS, doubles): self.uc.reg_write(reg, value)
        cpsr = self.uc.reg_read(arm.UC_ARM_REG_CPSR)
        self.uc.reg_write(arm.UC_ARM_REG_CPSR, (cpsr & ~FLAG_MASK) | flags)
        self.uc.reg_write(arm.UC_ARM_REG_FPSCR, fpscr)
        self.uc.mem_write(values[-1], struct.pack('<4I', *stack_args))
        try:
            self.uc.emu_start(entry, RETURN, count=100000)
        except UcError as error:
            raise RuntimeError(f'Gateway execution failed at {self.uc.reg_read(arm.UC_ARM_REG_PC):#x}: {error}') from error
        if self.uc.reg_read(arm.UC_ARM_REG_PC) != RETURN or self.scope:
            raise ValueError('Gateway budget exceeded or scopes leaked')
        return {'registers': [self.uc.reg_read(r) for r in REGISTERS],
                'doubles': [self.uc.reg_read(r) for r in DS],
                'flags': self.uc.reg_read(arm.UC_ARM_REG_CPSR) & NZCVQ_MASK,
                'fpscr': self.uc.reg_read(arm.UC_ARM_REG_FPSCR),
                'stack_args': bytes(self.uc.mem_read(values[-1], 16)).hex()}

def hitzone_cases(image, code, names, check):
    src = MOD
    lib_path = WORK/'hitzone_bridge.so'
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', '-O2',
                    '-shared', '-fPIC', '-I'+str(src/'src'), str(src/'tests/hitzone_bridge.cpp'),
                    '-o', str(lib_path)], check=True)
    lib = ctypes.CDLL(str(lib_path)); lib.dn_melee_hitzone.argtypes = [ctypes.c_void_p]
    lib.dn_melee_hitzone.restype = ctypes.c_int32
    monster, table, owner, output = 0x30000000, 0x30090000, 0x30091000, 0x30092000
    modifier_stub = RETURN+0x20
    def scalar(u, reg):
        return struct.unpack('<f', struct.pack('<I', u.reg_read(reg)))[0]
    class HitzoneProbe(GatewayProbe):
        def __init__(self, hooked):
            super().__init__(image, code, names)
            self.state, self.delta, self.modifier = 0, 5, 1.0
            self.write(0x30020000+0x18c, modifier_stub)
            self.originals[5] = 0x31004020
            if hooked:
                displaced = bytes(self.uc.mem_read(0x16dd7c, 8))
                self.uc.mem_write(0x31004020, displaced+struct.pack('<II', 0xe51ff004, 0x16dd84))
                self.uc.mem_write(0x16dd7c, struct.pack('<II', 0xe51ff004, names['dn_gateway_5']))
        def trace(self, u, address, size, _):
            if 0x16dc6c <= address < 0x16dd88:
                return
            if address == 0xbad1c:
                if u.reg_read(arm.UC_ARM_REG_R0) != monster: raise ValueError('HZV monster-state ABI differs')
                u.reg_write(arm.UC_ARM_REG_R0, self.state)
            elif address == 0x2ad1c8:
                if u.reg_read(arm.UC_ARM_REG_R0) != owner: raise ValueError('HZV player adjustment ABI differs')
                value = f32(scalar(u, arm.UC_ARM_REG_S0)+self.delta)
                u.reg_write(arm.UC_ARM_REG_S0, struct.unpack('<I', struct.pack('<f', value))[0])
            elif address == modifier_stub:
                if u.reg_read(arm.UC_ARM_REG_R0) != monster or u.reg_read(arm.UC_ARM_REG_R1) != 3:
                    raise ValueError('HZV monster-part modifier ABI differs')
                u.reg_write(arm.UC_ARM_REG_S0, struct.unpack('<I', struct.pack('<f', self.modifier))[0])
            else:
                super().trace(u, address, size, _); return
            u.reg_write(arm.UC_ARM_REG_PC, u.reg_read(arm.UC_ARM_REG_LR))
    baseline, tapped = HitzoneProbe(False), HitzoneProbe(True)
    calls = 0
    for cut, blunt, weights, state, player, modifier in itertools.product(
            (1, 20, 44, 45, 46, 80), (20, 44, 45, 80),
            ((100, 0), (0, 100), (50, 50), (30, 70), (70, 30)), (0, 1), (0, 1), (.5, 1.0, 1.5)):
        values = [0, monster, table, output]+[0x98765000+i for i in range(4, 13)]+[RETURN, STACK+0xfff0]
        doubles = [0x1234000012340000+i for i in range(32)]
        doubles[0] = struct.unpack('<Q', struct.pack('<2f', *weights))[0]
        args = [owner if player else 0, 3, 0, 0]
        for p in (baseline, tapped):
            p.state, p.modifier = state, modifier
            p.write(table, cut, '<B'); p.write(table+1, blunt, '<B')
            p.write(monster+0xb720, modifier != 1.0, '<B')
        expected = baseline.execute(0x16dc6c, values, doubles, 0, 0, args)
        actual = tapped.execute(0x16dc6c, values, doubles, 0, 0, args)
        check(actual == expected and baseline.read(output, '<B') == tapped.read(output, '<B'),
              'Mid-routine HZV tap preserves all original return state and selected attack type')
        modified_cut, modified_blunt = float(cut), float(blunt)
        if state == 1: modified_cut, modified_blunt = max(0, cut-15), max(0, blunt-15)
        if player:
            if weights[0] > weights[1]: modified_cut = f32(modified_cut+5)
            else: modified_blunt = f32(modified_blunt+5)
        if modifier != 1.0:
            modified_cut = max(0, f32(modified_cut*modifier))
            modified_blunt = max(0, f32(modified_blunt*modifier))
        cut_factor = f32(f32(weights[0]/100)*f32(modified_cut/100))
        blunt_factor = f32(f32(weights[1]/100)*f32(modified_blunt/100))
        selected_blunt = cut_factor < blunt_factor
        hzv = int(modified_blunt if selected_blunt else modified_cut)
        check(len(tapped.entries) == 1 and tapped.entries[0][0] == 5, 'Exactly one final HZV tap per native query')
        raw = ctypes.create_string_buffer(tapped.entries[0][2])
        check(lib.dn_melee_hitzone(raw) == hzv, 'C++ tap extracts adjusted physical HZV before attack weighting')
        check(tapped.read(output, '<B') == selected_blunt, 'Weighted cut/blunt comparison including native tie behavior')
        native_factor = scalar(tapped.uc, arm.UC_ARM_REG_S0)
        check(native_factor == max(cut_factor, blunt_factor), 'Original weighted damage factor remains exact')
        calls += 1
    return {'calls': calls, 'native_range': ['0x16dc6c', '0x16dd88'], 'tap': '0x16dd7c',
            'stub': 'Monster state query, player HZV adjustment and part-modifier virtual use explicit fixture values',
            'limit': 'Active collision-to-HZV scope and live monster-state services remain unvalidated'}

def main():
    if unicorn.__version__ != '2.1.4': raise ValueError('Use pinned Unicorn 2.1.4')
    image = (ROOT/'work/damage-numbers/inputs/main-memory.bin').read_bytes()
    if hashlib.sha256(image).hexdigest() != IMAGE_SHA: raise ValueError('Native image identity differs')
    manifest = json.loads((WORK/'manifest.json').read_text())
    code = (WORK/'gateway.bin').read_bytes()
    if hashlib.sha256(code).hexdigest() != manifest['binary_sha256']: raise ValueError('Gateway binary differs')
    for path, sha in manifest['sources'].items():
        if hashlib.sha256((ROOT/path).read_bytes()).hexdigest() != sha:
            raise ValueError(f'Gateway fixture source changed; rebuild: {path}')
    names = manifest['symbols']
    labels = []
    def check(ok, label):
        if not ok: raise AssertionError(label)
        labels.append(label)
    baseline = GatewayProbe(image, code, names)
    hooked = GatewayProbe(image, code, names, hooked=True)
    rng = random.Random(0x168960)
    cases = 0
    for i in range(96):
        values = [rng.getrandbits(32) for _ in range(13)] + [RETURN, STACK+0xfff0]
        doubles = [rng.getrandbits(64) for _ in range(32)]
        flags = rng.getrandbits(32) & FLAG_MASK
        fpscr = rng.choice((0, 0x01000000, 0x02000000, 0xa100009f))
        args = [rng.getrandbits(32) for _ in range(4)]
        expected = baseline.execute(names['dn_probe_leaf'], values, doubles, flags, fpscr, args)
        for kind in (6, 4, 5):
            actual = hooked.execute(names['dn_gateway_'+str(kind)], values, doubles, flags, fpscr, args)
            check(actual == expected, f'All return registers/VFP/flags/stack arguments preserved: case {i}, gateway {kind}')
            check(len(hooked.exits) == (1 if kind == 6 else 0), 'Around and before-only entry contracts')
            raw = hooked.entries[0][2]
            check(list(struct.unpack_from('<14I', raw, 272)) == values[:-1], 'Captured original integer registers and lr')
            check(list(struct.unpack_from('<16Q', raw, 16))+list(struct.unpack_from('<16Q', raw, 144)) == doubles[16:]+doubles[:16],
                  'Captured all original floating-point register bits')
            check(struct.unpack_from('<I', raw, 8)[0] & FLAG_MASK == flags and
                  struct.unpack_from('<I', raw, 12)[0] == fpscr, 'Captured original APSR and FPSCR')
            check(hooked.entries[0][1]+328 == values[-1], 'Original native stack argument address retained')
            cases += 1
    for capacity in (0, 1, 4, 64):
        nested = GatewayProbe(image, code, names, hooked=True, capacity=capacity)
        for depth in (0, 1, 2, 8, 65):
            values = [depth]+[0xabcdef00+i for i in range(1, 13)]+[RETURN, STACK+0xfff0]
            expected = baseline.execute(names['dn_probe_recursive'], values, [0]*32, 0, 0, [0]*4)
            actual = nested.execute(names['dn_probe_recursive'], values, [0]*32, 0, 0, [0]*4)
            check(actual == expected and actual['registers'][0] == depth*(depth+1)//2, 'Nested gateways preserve native recursion and all return state')
            check(len(nested.exits) == min(depth+1, capacity) and nested.overflow == max(0, depth+1-capacity),
                  'Scope capacity failure leaves the original call intact')
            cases += 1
    native = GatewayProbe(image, code, names, hooked=True, damage=True)
    direct = GatewayProbe(image, code, names, damage=True)
    for amount in (-50, 0, 1, 2, 17, 123, 1000):
        for mode in (0, 1, 2):
            values = [0x30000000, 3, amount & 0xffffffff, mode]+[0x98765000+i for i in range(4, 13)]+[RETURN, STACK+0xfff0]
            expected = direct.execute(START, values, [0]*32, 0, 0, [0]*4)
            actual = native.execute(START, values, [0]*32, 0, 0, [0]*4)
            check(actual == expected, 'Gateway leaves original damage return state unchanged')
            check(native.read(0x30010000+0x48e, '<h') == direct.read(0x30010000+0x48e, '<h'), 'Hooked native accumulator matches direct native execution')
            raw = native.exits[0][2]
            check(struct.unpack_from('<I', raw, 272)[0] == actual['registers'][0], 'After callback captures the completed native contribution')
            cases += 1
    hitzones = hitzone_cases(image, code, names, check)
    cases += hitzones['calls']
    # GE is explicitly outside restoration, not silently dropped from all
    # testing. Every entry must retain helper GE while preserving all other
    # state, for all 16 input patterns with preserving and clobbering helpers.
    ge_calls = 0
    values = [0x98765000+i for i in range(13)]+[RETURN, STACK+0xfff0]
    doubles = [0x1234000012340000+i for i in range(32)]
    args = [0x11223344, 0x55667788, 0x99aabbcc, 0xddeeff00]
    preserving = GatewayProbe(image, code, names, callback_ge=None)
    clobbering = GatewayProbe(image, code, names)
    for p in (preserving,clobbering):
        p.originals = {kind:names['dn_probe_leaf'] for kind in range(8)}
    for pattern in range(16):
        flags = 0xa8000000 | (pattern << 16)
        expected = baseline.execute(names['dn_probe_leaf'], values, doubles, flags, 0x01000000, args)
        for kind in range(8):
            for p,label in ((preserving,'preserving'),(clobbering,'clobbering')):
                if p is clobbering:p.callback_ge=pattern^15
                actual = p.execute(names['dn_gateway_'+str(kind)],values,doubles,flags,0x01000000,args)
                ge = (p.uc.reg_read(arm.UC_ARM_REG_CPSR) & GE_MASK) >> 16
                wanted = pattern if p is preserving else pattern^15
                check(actual==expected and ge==wanted,
                      f'NZCVQ restore leaves helper GE untouched: {label}, pattern {pattern}, gateway {kind}')
                ge_calls += 1
    cases += ge_calls
    out = ROOT/'build/damage-numbers/open-assets'; out.mkdir(parents=True, exist_ok=True)
    report = {'kind': 'Compiled ARM32 gateway and copied native damage in Unicorn; not Eden',
              'image_sha256': IMAGE_SHA, 'gateway_sha256': manifest['binary_sha256'],
              'calls': cases, 'checks': len(labels), 'passed': labels,
              'hitzones': hitzones,
              'scope': 'All r0-r12, d0-d31, NZCVQ, FPSCR, lr and stack arguments; nested/capacity-failed scopes; native normalization; GE is not restored',
              'GE_boundary_experiments':{'GE_preserving_helper':ge_calls//2,'GE_clobbering_helper':ge_calls//2},
              'limits': ['Callbacks are hostile emulator stubs, not the final runtime/TLS adapter',
                         'No CodeMemory publication, live thread scheduling or game acceptance']}
    (out/'gateway-probe.json').write_text(json.dumps(report, indent=2)+'\n')
    print(f'Passed {len(labels)} compiled ARM32 gateway checks across {cases} calls; no Eden trial')

if __name__ == '__main__':
    main()
