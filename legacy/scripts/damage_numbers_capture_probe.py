# Execute the compiled plugin and original native helper routines together.
# Import stubs model SDK/kernel/ImGui contracts and record calls for assertions;
# collision bodies and user-interface interactions are synthetic fixtures.

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
"""Load the compiled capture NRO in Unicorn with explicit SDK/kernel fixtures.

The collision body is synthetic. Original HZV/normalization, compiled capture,
TLS scopes, private arena publication and imports execute together. Not Eden.
"""
from pathlib import Path
import hashlib
import json
import struct
import unicorn
from unicorn import Uc, UC_ARCH_ARM, UC_MODE_ARM, UC_HOOK_CODE
from unicorn import arm_const as arm
from unicorn import UC_HOOK_MEM_WRITE
from mhgu_binary import Nro
from damage_numbers_arm_probe import ROOT, IMAGE_SHA, f32

MAIN, MODULE, DATA, STACK, RETURN = 0x04000000, 0x10000000, 0x30000000, 0x30100000, 0x30200000
MONSTER, PARAM, VTABLE, HIT, POSITION = DATA, DATA + \
    0x10000, DATA + 0x20000, DATA + 0x90000, DATA + 0x94000
REGS = [getattr(arm, 'UC_ARM_REG_R' + str(i)) for i in range(4)]
WORK = WORK_ROOT / 'open-assets'
OUT = BUILD_ROOT / 'open-assets'


# Read the unstripped ELF symbol table to locate compiled internal functions/state.
def elf_symbols(data):
    start = struct.unpack_from('<I', data, 32)[0]
    size, count = struct.unpack_from('<HH', data, 46)
    sections = [struct.unpack_from('<10I', data, start + i * size) for i in range(count)]
    result = {}
    for s in sections:
        if s[1] != 2:
            continue
        strings = sections[s[6]]
        names = data[strings[4]:strings[4] + strings[5]]
        for at in range(s[4], s[4] + s[5], s[9]):
            name, value, length, info, other, index = struct.unpack_from('<IIIBBH', data, at)
            if name and index:
                end = names.index(0, name)
                result[names[name:end].decode()] = (value, length)
    return result


def fill_rows(rows):
    # Each native label uses a distinct text buffer. Its last text call is the
    # crisp fill; earlier calls are shadows/halos even when their RGB matches.
    return list({row['text_ptr']: row for row in rows}.values())


# Own one simulated process with native code, relocated plugin imports, and
# recorded SDK/ImGui interactions. Tests vary its thread, memory, and failure state.
class CaptureProbe:
    def __init__(self, image, nro_bytes, gateway_bytes, gateway_names, title=True, allocation=True):
        self.uc = u = Uc(UC_ARCH_ARM, UC_MODE_ARM)
        u.ctl_set_cpu_model(arm.UC_CPU_ARM_MAX)
        u.reg_write(arm.UC_ARM_REG_C1_C0_2, 0xf00000)
        u.reg_write(arm.UC_ARM_REG_FPEXC, 0x40000000)
        self.nro = Nro(nro_bytes)
        module_bytes = (len(nro_bytes) + self.nro.u32(0x38) + 4095) & ~4095
        self.regions = [(MAIN, 0x2000000, 5), (MODULE, module_bytes, 7), (DATA, 0x210000, 7),
                        (0x31000000, 0x10000, 5), (0x32000000, 0x20000, 5)]
        for address, size, perm in self.regions:
            u.mem_map(address, size)
        u.mem_write(MAIN, image)
        u.mem_write(MODULE, nro_bytes)
        u.mem_write(0x31000000, gateway_bytes)
        self.symbols = elf_symbols((WORK / 'plugin.elf').read_bytes())
        self.imports = {}
        for i, (name, entry, *_) in enumerate(self.nro.symbols):
            if name and not entry[5]:
                self.imports[0x32000000 + i * 16] = name.decode()
        self.relocate()
        self.correct_title, self.allocate = title, allocation
        self.thread = 0
        self.tls = {}
        self.allocations = 0
        self.tls_allocations = 0
        self.aliases = []
        self.logs = []
        self.clock = (1 << 33) + 96000000
        self.frequency = 19200000
        self.text_draws = []
        self.lines = []
        self.triangles = []
        self.images = []
        self.widgets = []
        self.clip_depth = 0
        self.font_available = True
        self.font_registrations = 0
        self.font_released = 0
        self.measuring_pixels = 0
        self.varied_digits = False
        self.save_available = False
        self.save_mounts = 0
        self.save_commits = 0
        self.mount_calls = 0
        self.files = {}
        self.handles = {}
        self.fs_calls = []
        self.fs_paths = []
        self.fail_write_path = None
        self.fail_flush = False
        self.fail_commit = False
        self.changes = {}
        self.buttons = set()
        self.headers = []
        self.tree_open = {'Damage numbers##damage-numbers': True}
        self.tree_depth = 0
        self.tree_pops = 0
        self.fs_context = 0x12340000
        self.fs_guard_depth = 0
        abi = self.address('dn_font_abi')
        self.font_abi = struct.unpack('<6I', self.uc.mem_read(abi, 24))
        self.draw_abi = struct.unpack('<4I', self.uc.mem_read(self.address('dn_draw_abi'), 16))
        self.vertex_buffer = 0x35000000
        self.uc.mem_map(self.vertex_buffer, 0x100000)
        self.draw_vector = DATA + 0x1e1000 + self.draw_abi[0]
        self.write(self.draw_vector + self.draw_abi[1], self.vertex_buffer)
        self.write(DATA + 0x1e0000 + self.font_abi[0], DATA + 0x1e3000)
        self.write(DATA + 0x1e0000 + self.font_abi[5], 16)
        self.uc.mem_map(0x34000000, 0x100000)
        self.sprite_abi = struct.unpack('<4I', self.uc.mem_read(self.address('dn_sprite_abi'), 16))
        self.texture_registrations = 0
        self.texture_destroyed = 0
        self.texture_allocations = 0
        self.texture_sizes = {}
        self.quads = []
        self.sprite_decodes = 0
        self.context_hooks = []
        self.texture_unregistrations = 0
        self.codec_address = self.address(
            next(name for name in self.symbols if name.startswith('_ZN14damage_numbers13decode_sprite')))
        self.uc.mem_write(DATA + 0x1e0008, struct.pack('<2f', 1280, 720))
        self.collision_body = gateway_names['dn_probe_collision_body']
        self.write(MONSTER, VTABLE)
        self.write(MONSTER + 0x1428, PARAM)
        self.write(PARAM + 0x388, .75, '<f')
        self.players = [DATA + 0x40000 + 0x5000 * i for i in range(4)]
        session, table = DATA + 0xe0000, DATA + 0xe1000
        self.write(MAIN + 0x188ca48, session)
        self.write(MAIN + 0x1885b58, table)
        for i, player in enumerate(self.players):
            self.write(table + 0x1c + 4 * i, player)
        self.write(session + 0x30, 0)
        self.write(HIT + 0x10, 0, '<B')
        self.write(HIT + 0xc, self.players[0])
        self.write(HIT + 0x38, 100, '<h')
        self.write(HIT + 0x3a, 0, '<B')
        self.write(DATA + 0x93026, 3, '<B')
        self.uc.mem_write(POSITION, struct.pack('<3f', 12.0, 34.0, 56.0))
        self.uc.mem_write(DATA + 0x92000, bytes([45, 30]))
        self.uc.hook_add(UC_HOOK_CODE, self.trace)
        self.uc.hook_add(UC_HOOK_MEM_WRITE, self.mirror)

    # Translate a compiled ELF symbol to its mapped plugin address.
    def address(self, name):
        return MODULE + self.symbols[name][0]
    # Locate a private compiled variable by its C++ mangled name.

    def private(self, name):
        return self.address(f'_ZN14damage_numbersL{len(name)}{name}E')

    def write(self, address, value, fmt='<I'):
        self.uc.mem_write(address, struct.pack(fmt, value))

    def read(self, address, fmt='<I'):
        return struct.unpack(
            fmt, self.uc.mem_read(address, struct.calcsize(fmt)))[0]

    def string(self, address):
        data = bytes(self.uc.mem_read(address, 512))
        return data.split(b'\0', 1)[0].decode()

    # Resolve plugin relocations to mapped module data and named import stub addresses.
    def relocate(self):
        n = self.nro
        tables = []
        if 17 in n.tags:
            tables.append((n.tags[17], n.tags[18]))
        if 23 in n.tags:
            tables.append((n.tags[23], n.tags[2]))
        symbols = {name: address for address, name in self.imports.items()}
        for start, size in tables:
            for at in range(start, start + size, 8):
                target, info = struct.unpack_from('<II', n.data, at)
                kind = info & 255
                index = info >> 8
                addend = self.read(MODULE + target)
                if kind == 23:
                    value = MODULE + addend
                elif kind in (2, 21, 22):
                    name, entry, *_ = n.symbols[index]
                    value = MODULE + entry[1] if entry[5] else symbols[name.decode()]
                    if kind == 2:
                        value += addend
                else:
                    raise ValueError(f'Unsupported fixture relocation {kind}')
                self.write(MODULE + target, value & 0xffffffff)

    # Copy writes between fixture aliases representing the same CodeMemory storage.
    def mirror(self, u, access, address, size, value, _):
        for owner, slave in self.aliases:
            if owner <= address < owner + 4096:
                u.mem_write(slave + address - owner, value.to_bytes(size, 'little'))
                return

    # Return a stubbed 32-bit result through r0 and the native link register.
    def ret(self, value=0):
        self.uc.reg_write(arm.UC_ARM_REG_R0, value)
        self.uc.reg_write(arm.UC_ARM_REG_PC, self.uc.reg_read(arm.UC_ARM_REG_LR))

    # Return the 64-bit value through the ARM32 r0/r1 result pair.
    def ret64(self, value):
        self.uc.reg_write(arm.UC_ARM_REG_R1, value >> 32)
        self.ret(value & 0xffffffff)

    # Dispatch named import/native fixtures and capture their observable calls.
    # Each branch models one dependency contract while compiled plugin code executes.
    def trace(self, u, address, size, _):
        if address == self.codec_address:
            self.sprite_decodes += 1
        if address == RETURN:
            u.emu_stop()
            return
        args = [u.reg_read(r) for r in REGS]
        if address in self.imports:
            name = self.imports[address]
            if name == 'forge_fs_mountSaveData':
                if not self.fs_guard_depth:
                    raise ValueError('Save mount lacks scoped abort guard')
                self.mount_calls += 1
                if self.save_available:
                    self.save_mounts += 1
                self.ret(int(self.save_available))
            elif name == 'forge_fs_unmountSaveData':
                if not self.fs_guard_depth or self.save_mounts != 1 or self.handles:
                    raise ValueError(
                        'Save-mount release lacks guard, balanced mount or closed handles')
                self.save_mounts -= 1
                self.save_commits += 1
                self.ret()
            elif name == 'forge_mem_getMainTextAddr':
                self.ret(MAIN)
            elif name == 'nnosAllocateTlsSlot':
                self.tls_allocations += 1
                self.write(args[0], 17)
                self.ret()
            elif name == 'nnosGetTlsValue':
                if args[0] != 17:
                    raise ValueError('Capture accessed another TLS slot')
                self.ret(self.tls.get(self.thread, 0))
            elif name == 'nnosGetSystemTick':
                self.ret64(self.clock)
            elif name == 'nnosGetSystemTickFrequency':
                self.ret64(self.frequency)
            elif name.startswith('_ZN2nn2fs'):
                self.fs_calls.append(name)
                if name == '_ZN2nn2fs23ScopedAutoAbortDisablerC1Ev':
                    self.write(args[0], self.fs_context)
                    self.fs_context = 0x12341000
                    self.fs_guard_depth += 1
                    self.ret(args[0])
                elif name == '_ZN2nn2fs25SetCurrentThreadFsContextEPKNS0_9FsContextE':
                    self.fs_context = args[0]
                    self.fs_guard_depth -= 1
                    self.ret(DATA + 0x180034)
                elif not self.fs_guard_depth:
                    raise ValueError(
                        f'Settings filesystem access lacks scoped SDK abort suppression: {name}')
                elif not self.save_mounts:
                    raise ValueError('Settings I/O lacks a counted player save mount')
                elif name == '_ZN2nn2fs14CommitSaveDataEPKc':
                    if self.string(args[0]) != 'savedata' or self.handles:
                        raise ValueError('Unexpected commit mount or open handles')
                    self.save_commits += 1
                    self.ret(int(self.fail_commit))
                elif name == '_ZN2nn2fs8OpenFileEPNS0_10FileHandleEPKci':
                    path = self.string(args[1])
                    self.fs_paths.append(path)
                    if path != 'savedata:/damage_numbers.ini':
                        raise ValueError('Settings accessed an unrelated file')
                    if path not in self.files:
                        self.ret(1)
                    else:
                        handle = len(self.handles) + 1
                        self.handles[handle] = (path, args[2])
                        self.write(args[0], handle)
                        self.ret()
                elif name == '_ZN2nn2fs9CloseFileENS0_10FileHandleE':
                    del self.handles[args[0]]
                    self.ret()
                elif name == '_ZN2nn2fs11GetFileSizeEPxNS0_10FileHandleE':
                    self.write(args[0], len(self.files[self.handles[args[1]][0]]), '<q')
                    self.ret()
                elif name == '_ZN2nn2fs8ReadFileENS0_10FileHandleExPvj':
                    sp = u.reg_read(arm.UC_ARM_REG_SP)
                    buffer, length = struct.unpack('<2I', u.mem_read(sp, 8))
                    if args[2] or args[3] or not 0 < length < 256:
                        raise ValueError('Unexpected settings ReadFile ARM32 ABI')
                    data = self.files[self.handles[args[0]][0]]
                    u.mem_write(buffer, data)
                    self.ret()
                elif name == '_ZN2nn2fs9WriteFileENS0_10FileHandleExPKvjRKNS0_11WriteOptionE':
                    sp = u.reg_read(arm.UC_ARM_REG_SP)
                    buffer, length, option = struct.unpack('<3I', u.mem_read(sp, 12))
                    if args[2] or args[3] or not 0 < length < 256 or self.read(option) != 1:
                        raise ValueError('Unexpected settings WriteFile ARM32 ABI')
                    path, mode = self.handles[args[0]]
                    if mode != 2:
                        raise ValueError('Settings write without a write handle')
                    if path == self.fail_write_path:
                        self.files[path] = bytes(u.mem_read(buffer, 13))
                        self.ret(1)
                    else:
                        self.files[path] = bytes(u.mem_read(buffer, length))
                        self.ret()
                elif name == '_ZN2nn2fs11SetFileSizeENS0_10FileHandleEx':
                    if not 0 < args[2] < 256 or args[3]:
                        raise ValueError('Unexpected settings resize ABI')
                    path, mode = self.handles[args[0]]
                    self.files[path] = self.files[path][:args[2]].ljust(args[2], b'\0')
                    self.ret()
                elif name == '_ZN2nn2fs10CreateFileEPKcx':
                    path = self.string(args[0])
                    if not 0 < args[2] < 256 or args[3] or path in self.files:
                        self.ret(1)
                    else:
                        self.files[path] = bytes(args[2])
                        self.ret()
                elif name == '_ZN2nn2fs9FlushFileENS0_10FileHandleE':
                    self.ret(int(self.fail_flush))
                else:
                    raise ValueError(f'Unexpected settings SDK import {name}')
            elif name == '__aeabi_ul2f':
                self.ret(struct.unpack('<I', struct.pack('<f', args[0] + (args[1] << 32)))[0])
            elif name == 'strlen':
                self.ret(len(self.string(args[0])))
            elif name == 'igGetIO_Nil':
                self.ret(DATA + 0x1e0000)
            elif name == 'igGetBackgroundDrawList_Nil':
                self.ret(DATA + 0x1e1000)
            elif name == 'igGetFont':
                self.ret(DATA + 0x1e2000)
            elif name == 'ImFontConfig_ImFontConfig':
                u.mem_write(DATA + 0x1e6000, bytes(256))
                self.ret(DATA + 0x1e6000)
            elif name == 'ImFontConfig_destroy':
                if self.read(args[0] + self.font_abi[1], '<B') != 0:
                    raise ValueError('Atlas must not free embedded font')
                self.font_released += 1
                self.ret()
            elif name == 'ImFontAtlas_AddFontFromMemoryTTF':
                sp = u.reg_read(arm.UC_ARM_REG_SP)
                config = self.read(sp)
                ranges = self.read(sp + 4)
                if args[0] != DATA + 0x1e3000 or args[2] != json.loads((WORK / 'font-manifest.json').read_text())['subset_bytes'] or bytes(u.mem_read(args[1], 4)) != b'\x00\x01\x00\x00':
                    raise ValueError('Invalid embedded font registration ABI')
                if hashlib.sha256(bytes(u.mem_read(args[1], args[2]))).hexdigest() != json.loads((WORK / 'font-manifest.json').read_text())['subset_sha256']:
                    raise ValueError('Registered font payload differs from the selected profile')
                if self.read(config + self.font_abi[1], '<B') or bytes(u.mem_read(ranges, 14)) != struct.pack('<7H', 32, 32, 48, 57, 63, 63, 0):
                    raise ValueError('Font ownership or persistent ranges differ')
                self.font_registrations += 1
                self.ret(DATA + 0x1e7000 if self.font_available else 0)
            elif name == 'ImFont_GetFontBaked':
                self.measuring_pixels = struct.unpack('<f', struct.pack('<I', args[1]))[0]
                self.write(DATA + 0x1e4000 + self.font_abi[2], self.measuring_pixels, '<f')
                self.ret(DATA + 0x1e4000)
            elif name == 'ImFontBaked_FindGlyph':
                pixels = self.measuring_pixels
                glyph = DATA + 0x1e5000
                self.write(glyph, 2)
                width = (.28 if args[1] == 49 else .62) if self.varied_digits else .55
                self.write(glyph + self.font_abi[3], pixels * width, '<f')
                u.mem_write(glyph + self.font_abi[4], struct.pack('<4f',
                            0, pixels * .15, pixels * width, pixels * .85))
                self.ret(glyph)
            elif name == 'ImTextureData_ImTextureData':
                ptr = DATA + 0x1e8000 + self.texture_allocations * 0x100
                self.texture_allocations += 1
                u.mem_write(ptr, bytes(self.sprite_abi[0]))
                self.ret(ptr)
            elif name == 'ImTextureData_Create':
                if args[1:4] not in ([0, 512, 256], [0, 64, 512]):
                    raise ValueError('Sprite texture format/dimensions changed')
                self.texture_sizes[args[0]] = tuple(args[2:4])
                self.write(args[0] + self.sprite_abi[1],
                           0x34000000 if args[2] == 512 else 0x34080000)
                self.ret()
            elif name == 'ImTextureData_GetPixels':
                self.ret(self.read(args[0] + self.sprite_abi[1]))
            elif name == 'ImTextureData_destroy':
                self.texture_destroyed += 1
                self.ret()
            elif name == 'igRegisterUserTexture':
                width, height = self.texture_sizes[args[0]]
                data = bytes(u.mem_read(
                    self.read(args[0] + self.sprite_abi[1]), width * height * 4))
                expected = json.loads((WORK / ('sprite-manifest.json' if width ==
                                      512 else 'rise-manifest.json')).read_text())['rgba_sha256']
                if hashlib.sha256(data).hexdigest() != expected:
                    raise ValueError(
                        'Compiled lossless decoder corrupted profile sprite/effect pixels')
                self.texture_registrations += 1
                self.ret()
            elif name == 'ImTextureData_GetTexRef':
                u.mem_write(args[0], struct.pack('<IIQ', args[1], 0, 0))
                self.ret(args[0])
            elif name == 'igAddContextHook':
                size, typ, callback, user = struct.unpack(
                    '<4I', u.mem_read(self.address('dn_context_hook_abi'), 16))
                self.context_hooks.append(
                    (args[0], bytes(u.mem_read(args[1], size)), self.read(args[1] + callback)))
                if self.read(args[1] + typ) != 6:
                    raise ValueError('Expected shutdown texture hook')
                self.ret(len(self.context_hooks))
            elif name == 'igUnregisterUserTexture':
                self.texture_unregistrations += 1
                self.ret()
            elif name == 'ImDrawList_AddImageQuad':
                sp = u.reg_read(arm.UC_ARM_REG_SP)
                points = [tuple(self.read(sp + 8 + 8 * j + 4 * i, '<f')
                                for i in range(2)) for j in range(4)]
                uv = [tuple(self.read(sp + 40 + 8 * j + 4 * i, '<f')
                            for i in range(2)) for j in range(4)]
                self.quads.append({'points': points, 'uv': uv, 'color': self.read(sp + 72)})
                self.ret()
            elif name == 'ImDrawList_AddImage':
                sp = u.reg_read(arm.UC_ARM_REG_SP)
                self.images.append({'min': tuple(self.read(sp + 8 + 4 * i, '<f') for i in range(2)),
                                    'max': tuple(self.read(sp + 16 + 4 * i, '<f') for i in range(2)),
                                    'uv_min': tuple(self.read(sp + 24 + 4 * i, '<f') for i in range(2)),
                                    'uv_max': tuple(self.read(sp + 32 + 4 * i, '<f') for i in range(2)),
                                    'color': self.read(sp + 40)})
                self.ret()
            elif name == 'ImDrawList_AddTriangleFilled':
                sp = u.reg_read(arm.UC_ARM_REG_SP)
                self.triangles.append({'a': struct.unpack('<2f', struct.pack('<2I', *args[1:3])),
                                       'b': (struct.unpack('<f', struct.pack('<I', args[3]))[0], self.read(sp, '<f')),
                                       'c': (self.read(sp + 4, '<f'), self.read(sp + 8, '<f')), 'color': self.read(sp + 12)})
                self.ret()
            elif name == 'ImFont_CalcTextSizeA':
                # ARM32 aggregate return uses a hidden output pointer in r0.
                sp = u.reg_read(arm.UC_ARM_REG_SP)
                text = self.string(self.read(sp + 4))
                pixels = struct.unpack('<f', struct.pack('<I', args[2]))[0]
                u.mem_write(args[0], struct.pack('<2f', len(text) * pixels * .55, pixels))
                self.ret(args[0])
            elif name == 'ImDrawList_AddText_FontPtr':
                sp = u.reg_read(arm.UC_ARM_REG_SP)
                self.text_draws.append({'text': self.string(self.read(sp + 8)),
                                        'pixels': struct.unpack('<f', struct.pack('<I', args[2]))[0],
                                        'position': (struct.unpack('<f', struct.pack('<I', args[3]))[0], self.read(sp, '<f')),
                                        'color': self.read(sp + 4), 'font': args[1], 'text_ptr': self.read(sp + 8)})
                row = self.text_draws[-1]
                first = self.read(self.draw_vector)
                row['vertex_start'] = first
                row['vertex_count'] = 4 * len(row['text'])
                for j in range(row['vertex_count']):
                    self.uc.mem_write(self.vertex_buffer + (first + j) * self.draw_abi[2],
                                      struct.pack('<4fI', row['position'][0] + j % 2, row['position'][1] + (j // 2) % 2, 0, 0, row['color']))
                self.write(self.draw_vector, first + row['vertex_count'])
                self.ret()
            elif name == 'ImDrawList_AddLine':
                sp = u.reg_read(arm.UC_ARM_REG_SP)
                self.lines.append({'start': struct.unpack('<2f', struct.pack('<2I', *args[1:3])),
                                   'end': (struct.unpack('<f', struct.pack('<I', args[3]))[0], self.read(sp, '<f')),
                                   'color': self.read(sp + 4), 'thickness': self.read(sp + 8, '<f')})
                self.ret()
            elif name == 'ImDrawList_PushClipRect':
                self.clip_depth += 1
                self.ret()
            elif name == 'ImDrawList_PopClipRect':
                self.clip_depth -= 1
                if self.clip_depth < 0:
                    raise ValueError('Unbalanced native draw-list clipping')
                self.ret()
            elif name == 'nnosSetTlsValue':
                if args[0] != 17:
                    raise ValueError('Capture changed another TLS slot')
                self.tls[self.thread] = args[1]
                # Native void setter retains a computed TLS address in r0.
                # Returning fixture zero masked the old Result ABI mistake.
                self.ret(DATA + 0x180000 + 4 * args[0])
            elif name in ('memcpy', 'memmove'):
                u.mem_write(args[0], bytes(u.mem_read(args[1], args[2])))
                self.ret(args[0])
            elif name == 'memcmp':
                a = bytes(u.mem_read(args[0], args[2]))
                b = bytes(u.mem_read(args[1], args[2]))
                self.ret((int(a > b) - int(a < b)) & 0xffffffff)
            elif name == 'memset':
                u.mem_write(args[0], bytes([args[1] & 255]) * args[2])
                self.ret(args[0])
            elif name == 'forge_log':
                fmt = self.string(args[1])
                sp = u.reg_read(arm.UC_ARM_REG_SP)
                values = tuple(args[2:]) + struct.unpack('<10I', u.mem_read(sp, 40)
                                                         ) if fmt.startswith('[damage-numbers] Hit %u:') else ()
                positions = struct.unpack('<3d', u.mem_read(sp + 40, 24)) if values else ()
                self.logs.append((fmt, values, positions))
                self.ret()
            elif name == 'igTreeNode_Str':
                label = self.string(args[0])
                self.headers.append(label)
                opened = self.tree_open.get(label, False)
                if opened:
                    self.tree_depth += 1
                self.ret(int(opened))
            elif name == 'igTreePop':
                self.tree_depth -= 1
                self.tree_pops += 1
                if self.tree_depth < 0:
                    raise ValueError('Unbalanced compact plugin tree stack')
                self.ret()
            elif name == 'igCombo_Str_arr':
                label = self.string(args[0])
                self.widgets.append(label)
                if args[3] != 3 or [self.string(self.read(args[2] + 4 * i)) for i in range(3)] != ['Wilds', 'Classic', 'Rise']:
                    raise ValueError('Unexpected style selector ABI')
                if label in self.changes:
                    self.write(args[1], self.changes.pop(label))
                    self.ret(1)
                else:
                    self.ret()
            elif name in ('igCheckbox', 'igSliderFloat'):
                label = self.string(args[0])
                self.widgets.append(label)
                if label not in self.changes:
                    self.ret()
                else:
                    value = self.changes.pop(label)
                    self.write(args[1], value, '<B' if name == 'igCheckbox' else '<f')
                    self.ret(1)
            elif name == 'igButton':
                label = self.string(args[0])
                pressed = label in self.buttons
                self.buttons.discard(label)
                self.ret(int(pressed))
            elif name in ('igText', 'igTextUnformatted', 'igSeparator', 'igSetCurrentContext', 'igSetAllocatorFunctions'):
                self.ret()
            else:
                raise ValueError(f'Unexpected capture import {name}')
            return
        if address == self.address('mwc_svc_get_info'):
            values = {18: 0x0100770008dd8000 if self.correct_title else 0, 12: 0x20000000, 13: 0x100000,
                      2: 0x28000000, 3: 0x10000, 4: 0x29000000, 5: 0x10000}
            if args[1] not in values:
                raise ValueError('Unexpected kernel info selector')
            self.write(args[0], values[args[1]], '<Q')
            self.ret()
            return
        if address == self.address('mwc_svc_query'):
            a = args[2]
            info = next(((start, length, 3, perm)
                        for start, length, perm in self.regions if start <= a < start + length), None)
            if info is None and 0x20000000 <= a < 0x20100000:
                end = min([start for start, length, perm in self.regions if a <
                          start < 0x20100000] + [0x20100000])
                info = (a, end - a, 0, 0)
            if info is None:
                info = (a, 4096, 0, 0)
            start, length, kind, perm = info
            u.mem_write(args[0], struct.pack('<QQIIIIII', start, length, kind, 0, perm, 0, 0, 0))
            self.write(args[1], 0)
            self.ret()
            return
        if address == self.address('mwc_svc_create'):
            self.allocations += 1
            self.write(args[0], 7 if self.allocate else 0xffffffff)
            self.ret(0 if self.allocate else 1)
            return
        if address == self.address('mwc_svc_control'):
            length, high, perm = struct.unpack('<3I', u.mem_read(u.reg_read(arm.UC_ARM_REG_SP), 12))
            if args[0] != 7 or args[3] or high or length != 4096:
                raise ValueError('Unexpected CodeMemory fixture ABI')
            if args[1] in (0, 1):
                if perm != (3 if args[1] == 0 else 5):
                    raise ValueError('Unexpected CodeMemory permission')
                u.mem_map(args[2], 4096)
                self.regions.append((args[2], 4096, perm))
                if args[1] == 0:
                    self.owner = args[2]
                else:
                    self.aliases.append((self.owner, args[2]))
            elif args[1] in (2, 3):
                self.regions = [r for r in self.regions if r[0] != args[2]]
                u.mem_unmap(args[2], 4096)
            else:
                raise ValueError('Unexpected CodeMemory operation')
            self.ret()
            return
        if address in (self.address('mwc_svc_flush'), self.address('mwc_svc_close')):
            self.ret()
            return
        if address == MAIN + 0x170970:
            u.reg_write(arm.UC_ARM_REG_PC, self.collision_body)
            return
        if address in (MAIN + 0x81c6c, MAIN + 0xa1fc8, MAIN + 0x3a8d68, MAIN + 0xbad1c):
            self.ret()
            return
        if address == MAIN + 0xbab60:
            self.ret(DATA + 0x92000)
            return
        if address == MAIN + 0xdf814:
            if args[:2] != [DATA + 0xa1004, 0]:
                raise ValueError('Unexpected native feedback status ABI')
            self.ret(DATA + 0xa1100)
            return
        if address in (MAIN + 0xdd4c4, MAIN + 0x2864c4, RETURN + 4):
            self.ret()
            return
        if address == MAIN + 0x710f0:
            self.ret(0xffffffff)
            return
        permitted = ((MAIN + 0x170968, MAIN + 0x170970), (MAIN + 0x9d6f8, MAIN + 0x9d8a8),
                     (MAIN + 0x16dc6c, MAIN + 0x16dd88), (MAIN + 0x178660, MAIN + 0x178850),
                     (MAIN + 0x2a87a8, MAIN + 0x2a8a50), (MODULE, MODULE + self.nro.segments[0][1]),
                     (0x31000000, 0x31010000))
        if any(a <= address < b for a, b in permitted) or any(slave <= address < slave + 4096 for _, slave in self.aliases):
            return
        raise ValueError(f'Unexpected compiled capture dependency {address:#x}')

    # Enter one compiled symbol with supplied ARM arguments and a bounded instruction budget.
    def call(self, name, args=(), stack=(0, 0, 0, 0), budget=500000):
        if name == 'forge_onImGuiInit':
            budget = max(budget, 5000000)
        entry = self.address(name) if isinstance(name, str) else name
        u = self.uc
        u.reg_write(arm.UC_ARM_REG_SP, STACK + 0xfff0)
        u.reg_write(arm.UC_ARM_REG_LR, RETURN)
        for r, value in zip(REGS, args):
            u.reg_write(r, value)
        u.mem_write(STACK + 0xfff0, struct.pack('<4I', *stack))
        u.emu_start(entry, RETURN, count=budget)
        if u.reg_read(arm.UC_ARM_REG_PC) != RETURN:
            raise ValueError('Compiled capture instruction budget exceeded')
        return u.reg_read(arm.UC_ARM_REG_R0)

    # Drive the synthetic collision body through the installed detour and native helpers.
    def collision(self, amount=100, owner=0, local=0, kind=0, damage_type=0, zone=45, attack=HIT):
        self.write(DATA + 0xe0030, local)
        self.write(attack + 0x10, kind, '<B')
        if kind == 3:
            self.write(DATA + 0x80000 + 0x13c8, 0)
            self.write(DATA + 0x80000 + 0x3ac4, DATA + 0x81000)
            self.write(DATA + 0x81001, owner, '<B')
            self.write(attack + 0xc, DATA + 0x80000)
        else:
            self.write(attack + 0xc, self.players[owner])
        self.write(attack + 0x38, amount, '<h')
        self.write(attack + 0x3a, damage_type, '<B')
        self.write(DATA + 0x92000, zone, '<B')
        self.write(PARAM + 0x48e, 0, '<h')
        value = self.call(MAIN + 0x170968, (0, attack, MONSTER,
                          POSITION), (0, 0, DATA + 0x93020, 0))
        return struct.unpack('<i', struct.pack('<I', value))[0]

    # Invoke the compiled mailbox consumer on the selected simulated thread.
    def consume(self):
        self.call('_ZN14damage_numbers15capture_consumeEv')

    # Visit the compiled settings panel with the current simulated widget/button state.
    def panel(self):
        self.call('forge_onImGuiRender')
        if self.tree_depth:
            raise ValueError('Plugin panel leaked a TreeNode scope')

    # Clear recorded draw operations before invoking the compiled overlay callback.
    def render(self):
        self.text_draws = []
        self.lines = []
        self.triangles = []
        self.images = []
        self.quads = []
        self.write(self.draw_vector, 0)
        self.call('forge_onImGuiFreeRender', budget=8000000)

    # Drive the viewport/camera gateways with fixture matrices and a declared rectangle.
    def camera_record(self, rect=(0, 0, 1280, 720), view=None, projection=None, camera_delta=0):
        frame, viewport, camera, vm, pm, rectangle = [
            DATA + 0x1f0000 + i * 0x1000 for i in range(6)]
        camera += camera_delta
        identity = (1.0, 0, 0, 0, 0, 1.0, 0, 0, 0, 0, 1.0, 0, 0, 0, 0, 1.0)
        self.uc.mem_write(frame, bytes(328))
        self.write(frame + 324, RETURN)
        self.write(frame + 272, viewport)
        self.write(viewport + 0x24, camera)
        self.write(camera, MAIN + 0x1730d0c)
        self.uc.mem_write(vm, struct.pack('<16f', *(view or identity)))
        self.uc.mem_write(pm, struct.pack('<16f', *(projection or identity)))
        self.uc.mem_write(rectangle, struct.pack('<4i', *rect))
        self.call('dn_gateway_enter', (3, frame))
        self.write(frame + 276, vm)
        self.write(frame + 280, pm)
        self.write(frame + 284, rectangle)
        self.call('dn_gateway_tap', (4, frame))
        self.call('dn_gateway_leave', (frame,))

    # Read the compiled recent-record buffer for fixture assertions.
    def recent(self):
        n = self.read(self.private('recent_count'))
        return [struct.unpack('<10I2i3f32f4i', self.uc.mem_read(self.private('recent') + 248 * i, 204)) for i in range(n)]


# Cover hook lifecycle, thread handoff, rendering, source policy, and save failures.
# Native execution and simulated dependencies remain distinct from gameplay evidence.
def main():
    from damage_numbers_assets import argument, paths
    global WORK, OUT
    WORK, OUT = paths(argument())
    if unicorn.__version__ != '2.1.4':
        raise ValueError('Use pinned Unicorn 2.1.4')
    image = (WORK_ROOT / 'inputs/main-memory.bin').read_bytes()
    if hashlib.sha256(image).hexdigest() != IMAGE_SHA:
        raise ValueError('Native image differs')
    manifest = json.loads((OUT / 'candidate-manifest.json').read_text())
    for name, sha in manifest['sources'].items():
        if hashlib.sha256((ROOT / name).read_bytes()).hexdigest() != sha:
            raise ValueError(f'Source changed; rebuild capture: {name}')
    nro_bytes = (OUT / 'candidate-package/romfs/nativeNX/plugins/damage_numbers_capture.nro').read_bytes()
    if hashlib.sha256(nro_bytes).hexdigest() != manifest['nro_sha256']:
        raise ValueError('Candidate NRO differs')
    gateway = json.loads((WORK_ROOT / 'gateway/manifest.json').read_text())
    gateway_bytes = (WORK_ROOT / 'gateway/gateway.bin').read_bytes()
    if hashlib.sha256(gateway_bytes).hexdigest() != gateway['binary_sha256']:
        raise ValueError('Gateway fixture differs')
    checks = []

    def check(ok, label):
        if not ok:
            raise AssertionError(label)
        checks.append(label)
    symbols = elf_symbols((WORK / 'plugin.elf').read_bytes())
    for name in ['dn_gateway_' + str(i) for i in range(8)] + ['dn_gateway_after']:
        offset, length = symbols[name]
        words = struct.unpack('<' + 'I' * (length // 4), nro_bytes[offset:offset + length])
        check(words.count(0xe128f002) == 1 and 0xe12cf002 not in words,
              'Compiled ' + name + ' restores NZCVQ without unsupported APSR_g')

    def probe(**kwargs):
        p = CaptureProbe(image, nro_bytes, gateway_bytes, gateway['symbols'], **kwargs)
        p.call('forge_onImGuiInit', (DATA + 0x1e9000, RETURN, RETURN))
        return p
    for kwargs in ({'title': False}, {'allocation': False}):
        p = probe(**kwargs)
        p.call('forge_onInit')
        check(not p.read(p.private('active')),
              'Capture stays inactive when program/allocation gate fails')
        check(all(bytes(p.uc.mem_read(MAIN + int(offset, 16), 8)) == image[int(offset, 16):int(offset, 16) + 8]
                  for offset in manifest['hook_offsets']), 'Rejected initialization leaves all eight game sites unchanged')
    recovery = probe()
    recovery.call('forge_onInit')
    check(recovery.read(recovery.private('hook_count')) == 8,
          'Default initialization installs all eight combat and camera hooks without flag APIs')
    check({name for name in recovery.imports.values() if name.startswith('forge_fs_')} ==
          {'forge_fs_mountSaveData', 'forge_fs_unmountSaveData'},
          'Compiled candidate imports only counted save-mount helpers, with no activation-file APIs')
    for i, offset in enumerate(manifest['hook_offsets']):
        a = MAIN + int(offset, 16)
        check(recovery.read(a + 4) == recovery.address('dn_gateway_' + str(i)),
              'Automatic startup installs the correct gateway at every hook site')
    recovery.collision()
    recovery.consume()
    check(len(recovery.recent()) == 1 and recovery.read(recovery.private('pool_drops')) == 0,
          'Native void TLS setter returning a nonzero address permits capture on the first call')
    recovery.write(MAIN + 0x885844, 0)
    recovery.call('forge_onUpdate', (0,))
    check(recovery.read(recovery.private('active')) == 0,
          'Automatic startup monitor disables capture after a camera-site change')
    retained = probe()
    retained.call('forge_onInit')
    retained.collision(amount=100)
    retained.consume()
    saved = retained.recent()
    hit_logs = lambda: [row for row in retained.logs if row[1]]
    check(not hit_logs() and saved[-1][10] == 75, 'Release captures native damage without hit logs')
    startup_logs = len(retained.logs)
    retained.write(MAIN + 0x1885b58, 0)
    retained.consume()
    check(retained.recent() == saved, 'Last damage records survive disappearance of the local player table')
    retained.write(MAIN + 0x1885b58, DATA + 0xe1000)
    retained.write(DATA + 0xe0030, 1)
    retained.consume()
    check(retained.recent() == saved, 'Last damage records survive a replacement local hunter context')
    retained.collision(amount=80)
    retained.write(MAIN + 0x1885b58, 0)
    retained.consume()
    check(retained.recent()[-1][10] == 60,
          'A queued owned hit is retained when its player disappears before consumption')
    retained.write(MAIN + 0x1885b58, DATA + 0xe1000)
    for _ in range(12):
        retained.collision(amount=40)
    retained.consume()
    check(retained.read(retained.private('captured_hits')) == 10,
          'A render callback consumes at most eight new hits')
    retained.consume()
    check(retained.read(retained.private('captured_hits')) == 14 and len(retained.recent())
          == 8, 'Subsequent callback drains remaining hits into a bounded history')
    for _ in range(32):
        for _ in range(8):
            retained.collision(amount=24)
        retained.consume()
    check(retained.read(retained.private('captured_hits')) == 270 and len(retained.logs) == startup_logs,
          '270 hits and transitions emit no routine release logs')
    check(all(row[10] == 18 for row in retained.recent()),
          'Latest safety records keep updating with logging disabled')
    fresh = probe()
    fresh.call('forge_onInit')
    check(not fresh.recent() and fresh.read(fresh.private('captured_hits'))
          == 0, 'Restart initializes an empty diagnostic history')
    for _ in range(12):
        fresh.collision(amount=12)
    fresh.thread = 1
    fresh.collision(amount=100)
    fresh.thread = 0
    fresh.consume()
    check(any(row[10] == 75 for row in fresh.recent()),
          'Round-robin draining reaches another producer despite a busy first mailbox')
    invalid_clock = probe()
    invalid_clock.frequency = 0
    invalid_clock.call('forge_onInit')
    check(not invalid_clock.read(invalid_clock.private('active')) and invalid_clock.tls_allocations == 0,
          'Invalid clock frequency fails closed before TLS and hook allocation')
    recovery.render()
    check(not recovery.text_draws, 'External camera-site change stops floating-number rendering')
    visual = probe()
    visual.call('forge_onInit')
    visual.render()
    check(not visual.text_draws, 'Automatically active renderer waits for a quest-camera snapshot')
    visual.uc.mem_write(POSITION, struct.pack('<3f', .25, 0, .5))
    visual.write(HIT + 0x54, 2, '<B')
    visual.camera_record()
    visual.collision()
    visual.render()
    fills = lambda: fill_rows(visual.text_draws)
    font_scale = json.loads((WORK / 'font-manifest.json').read_text())['height_scale']
    open_motion = manifest.get('asset_profile') == 'open' and manifest['recovery_revision'] >= 13
    start_pixels = 24.0 if open_motion else 25.0
    fade_alpha = 128 if open_motion else 102
    stars = lambda p: [r for r in p.images if r['uv_min'][1] == .5]
    center = lambda r: tuple((a + b) * .5 for a, b in zip(r['min'], r['max']))
    check(len(visual.text_draws) == 9 and len(fills()) == 1 and fills()[0]['text'] == '75',
          'Native callback draws smooth texture shading and one crisp profile-font fill')
    check(fills()[0]['color'] == 0xff5bc4f2 and abs(fills()[0]['pixels'] - start_pixels * font_scale) < .001,
          'Physical HZV 45 keeps the golden digit color and accepted normalized height')
    check(abs(fills()[0]['position'][0] + fills()[0]['pixels'] * .55 - 799.5) < 1 and abs(fills()[0]['position'][1] - 352) < .001,
          'Compiled renderer centers measured profile-font advances at the projected impact')
    check(len(stars(visual)) == 1 and stars(visual)[0]['uv_min'][0] == 0 and not visual.triangles,
          'Positive critical selects profile red sprite with baked glow, without triangle overdraw')
    r = stars(visual)[0]
    scale = start_pixels * \
        manifest['sprites'].get('crit_render_scale', font_scale) / 60 * \
        manifest['wilds_crit_size_multiplier']
    check(abs(r['max'][0] - r['min'][0] - 88 * scale) < .001 and abs(r['max'][1] - r['min'][1] - 48 * scale) < .001,
          'Critical sprite preserves the renderer design aspect ratio')
    check(center(r)[1] < fills()[0]['position'][1] and r['color'] == 0xffffffff,
          'Crit sprite stays above glyphs with independent sprite RGBA colors')
    check(abs(fills()[0]['position'][1] + fills()[0]['pixels'] * .15 - center(r)[1] - start_pixels * manifest['wilds_star_height_font_sizes'] * .5 - 2 * start_pixels / 1.2 / 24) < .001,
          'Larger positive crit preserves the gap above visible digits')
    check(len(visual.images) == 3 and sum(row['color'] & 0xffffff == 0 for row in visual.images) == 2,
          'Gold digits retain centered shadow without a warm halo')
    outline = visual.text_draws[:-1]
    fill = visual.text_draws[-1]
    offsets = [(x, y) for x in (-1, 0, 1) for y in (-1, 0, 1) if x or y]

    def vertex_pos(p, row):
        return struct.unpack('<2f', p.uc.mem_read(p.vertex_buffer + row['vertex_start'] * p.draw_abi[2] + p.draw_abi[3], 8))
    origin = vertex_pos(visual, fill)
    stroke = .5 * start_pixels / 20
    check(len(outline) == 8 and all(row['color'] & 0xffffff == 0x0f1317 and row['color'] >> 24 == 230 for row in outline),
          'Wilds uses eight near-black outline passes at 90 percent opacity')
    check(all(abs(vertex_pos(visual, row)[axis] - origin[axis] - offset[axis] * stroke) < .0001
              for row, offset in zip(outline, offsets) for axis in (0, 1)),
          'Compiled outline translates vertices symmetrically by 0.5 pixels scaled with font size')
    check(visual.clip_depth == 0, 'Native renderer balances draw-list clipping')
    check(visual.read(visual.private('recent') + 240, '<Q') == visual.clock,
          'Compiled producer copies the full 64-bit capture timestamp')
    visual.write(HIT + 0x54, 1, '<B')
    visual.collision(amount=40, zone=44)
    visual.render()
    check(len(fills()) == 2 and {row['text'] for row in fills()} == {'75', '30'},
          'Same-frame native contributions render as separate numbers')
    lower = next(row for row in fills() if row['text'] == '30')
    check(lower['color'] == 0xffecf1f4,
          'Physical HZV 44 retains warm white with blue negative marker')
    check(len(stars(visual)) == 2 and stars(visual)[1]['uv_min'][0] == .1875 and
          center(stars(visual)[1])[1] > lower['position'][1] + lower['pixels'] * .85,
          'Negative critical selects profile blue sprite below visible digits')
    check(abs(center(stars(visual)[1])[1] - start_pixels * manifest['wilds_star_height_font_sizes'] * .5 - lower['position'][1] - lower['pixels'] * .85 - 2 * start_pixels / 1.2 / 24) < .001,
          'Larger negative crit preserves the gap below visible digits')
    first_position = fills()[0]['position']
    visual.render()
    check(first_position == fills()[0]['position'],
          'New overlapping hit does not reshuffle an existing number')
    visual.clock += int(.8 * visual.frequency)
    visual.camera_record()
    visual.render()
    check(len(fills()) == 2 and all(abs((row['color'] >> 24) - fade_alpha) <= 1 and abs(row['pixels'] - 20 * font_scale) < .001 for row in fills()),
          'Compiled renderer settles pop and fades both numbers near native capture-time expiry')
    visual.clock += int(.21 * visual.frequency)
    visual.render()
    check(not visual.text_draws and len(visual.recent()) == 2,
          'Stale camera clears transient display while retaining diagnostic hit history')
    visual.camera_record()
    visual.collision(amount=20)
    visual.render()
    check(len(fills()) == 1 and fills()[0]['text'] == '15',
          'Fresh camera and new hit resume the preview')
    visual.camera_record(camera_delta=0x100)
    visual.render()
    check(not visual.text_draws, 'Camera identity change clears old floating numbers without replaying history')
    visual.camera_record(rect=(10, 0, 1290, 720))
    visual.collision()
    visual.render()
    check(not visual.text_draws, 'Unknown viewport offset is hidden in compiled native renderer')
    visual.camera_record()
    visual.uc.mem_write(DATA + 0x1e0008, struct.pack('<2f', 1024, 768))
    visual.collision()
    visual.render()
    check(not visual.text_draws, 'Mismatched native/display aspect is hidden in compiled native renderer')
    visual.uc.mem_write(DATA + 0x1e0008, struct.pack('<2f', 2560, 1440))
    visual.camera_record()
    visual.collision(zone=80)
    visual.render()
    check(len(fills()) == 1 and abs(fills()[0]['pixels'] - 2 * start_pixels * font_scale) < .001 and abs(fills()[0]['position'][0] + fills()[0]['pixels'] * .55 - 1600) < 1,
          'Native render-scale rectangle maps correctly into a larger Forge swapchain')
    if open_motion:
        timing = probe()
        timing.call('forge_onInit')
        timing.uc.mem_write(POSITION, struct.pack('<3f', 0, 0, .5))
        timing.write(HIT + 0x54, 2, '<B')
        timing.camera_record()
        timing.collision()
        timing.render()
        first = fill_rows(timing.text_draws)[0]
        baseline_y = first['position'][1]
        check(abs(first['pixels'] - 24 * font_scale) < .001,
              'Open Wilds compiled stamp starts at 1.2x')
        start_clock = timing.clock

        def frame(age):
            timing.clock = start_clock + int(age * timing.frequency)
            timing.camera_record()
            timing.render()
            return fill_rows(timing.text_draws)
        row = frame(.1)[0]
        check(abs(row['pixels'] - 21 * font_scale) < .001,
              'Open Wilds compiled stamp eases down during 0.2 seconds')
        row = frame(.2)[0]
        check(abs(row['pixels'] - 20 * font_scale) < .001,
              'Open Wilds compiled stamp settles at 0.2 seconds')
        row = frame(.65)[0]
        check(row['position'][1] == baseline_y and row['color'] >> 24 == 255,
              'Open Wilds compiled hold stays stationary and opaque beyond 0.5 seconds')
        row = frame(.8)[0]
        check(abs(row['position'][1] - (baseline_y - 15)) <= 1 and abs((row['color'] >> 24) - 128) <= 1,
              'Open Wilds compiled exit rises 15 pixels and halves opacity in 0.1 seconds')
        check(abs((stars(timing)[0]['color'] >> 24) - 128) <= 1,
              'Open Wilds compiled crit marker fades with the fast exit')
        check(not frame(.91), 'Open Wilds compiled exit expires at saved total duration')
        timing.changes = {'Style##damage-numbers': 1}
        timing.panel()
        timing.collision()
        timing.render()
        classic = fill_rows(timing.text_draws)[0]
        classic_y = classic['position'][1]
        classic_start = timing.clock
        check(classic['pixels'] == 25, 'Open edition compiled Classic retains 1.25x pop')
        timing.clock = classic_start + int(.1 * timing.frequency)
        timing.camera_record()
        timing.render()
        classic = fill_rows(timing.text_draws)[0]
        check(abs(classic['pixels'] - 20) < .001 and classic['position'][1] < classic_y,
              'Open edition compiled Classic keeps its early continuous drift')
    sized = probe()
    sized.call('forge_onInit')
    sized.uc.mem_write(POSITION, struct.pack('<3f', 0, 0, .5))
    sized.write(HIT + 0x54, 2, '<B')
    sized.camera_record()
    sized.collision()
    sized.clock += int(.25 * sized.frequency)
    sized.camera_record()
    sized.render()
    check(abs(fill_rows(sized.text_draws)[0]['pixels'] - 20 * font_scale) < .001,
          'R15 default 1x settles at 20 pixels at 720p')
    sized.uc.mem_write(DATA + 0x1e0008, struct.pack('<2f', 1920, 1080))
    sized.camera_record()
    sized.render()
    check(abs(fill_rows(sized.text_draws)[0]['pixels'] - 30 * font_scale) < .001,
          'R15 default 1x settles at 30 pixels at 1080p')
    sized.uc.mem_write(DATA + 0x1e0008, struct.pack('<2f', 1280, 720))
    sized.changes = {'Number size': 1.2}
    sized.panel()
    sized.camera_record()
    sized.render()
    check(abs(fill_rows(sized.text_draws)[0]['pixels'] - 24 * font_scale) < .001,
          'R15 size slider retains 24 pixels at 1.2x')
    visual.write(MAIN + 0x1885b58, 0)
    visual.render()
    check(not visual.text_draws and len(visual.recent()) > 0,
          'Player disappearance hides world numbers and preserves diagnostics')
    visual.write(MAIN + 0x1885b58, DATA + 0xe1000)
    visual.camera_record()
    visual.render()
    check(not visual.text_draws, 'Returning player context does not replay retained diagnostic hits')
    visual.uc.mem_write(POSITION, struct.pack('<3f', 2, 0, .5))
    visual.collision()
    visual.render()
    check(not visual.text_draws, 'Compiled renderer hides off-screen impact positions')
    delayed = probe()
    delayed.call('forge_onInit')
    delayed.render()
    delayed.collision()
    delayed.clock += 2 * delayed.frequency
    delayed.camera_record()
    delayed.render()
    check(not delayed.text_draws and delayed.read(delayed.private('preview_rejected')) == 1,
          'Consumer never refreshes the lifetime of a hit delayed in a mailbox')
    check(sum('Preview camera:' in row[0] for row in visual.logs) == 0 and
          sum('Preview projection:' in row[0] for row in visual.logs) == 0,
          'Release render frames emit no camera, matrix or projection diagnostics')
    # R4 logged these local-palico child sources but deliberately omitted them
    # from the preview. Exercise the producer's actual secondary-owner lookup.
    for subtype, zone, expected_color in ((4, 80, 0xff5bc4f2), (8, -1, 0xffecf1f4)):
        projectile = probe()
        projectile.call('forge_onInit')
        projectile.render()
        cat, work = DATA + 0x80000, DATA + 0x81000
        projectile.write(cat + 0x13c8, 0)
        projectile.write(cat + 0x3ac4, work)
        projectile.write(work + 1, 0, '<B')
        projectile.write(HIT + 0x6e, 3, '<B')
        projectile.write(HIT + 0x70, cat)
        projectile.write(HIT + 0x6d, subtype, '<B')
        projectile.uc.mem_write(POSITION, struct.pack('<3f', .25, 0, .5))
        if zone == -1:
            projectile.collision_body = gateway['symbols']['dn_probe_collision_shot_body']
            projectile.write(HIT + 0x30, 0)
        projectile.camera_record()
        amount = projectile.collision(kind=2, zone=max(0, zone))
        projectile.render()
        painted = fill_rows(projectile.text_draws)
        check(len(painted) == 1 and painted[0]['text'] == '75' and painted[0]['color'] == expected_color,
              'Observed local-palico projectile subtype renders native damage with physical/unknown HZV color')
        record = projectile.recent()[-1]
        check(record[2] == 1 and record[11] == zone and amount == 75 and projectile.read(PARAM + 0x48e, '<h') == 75,
              'Secondary palico ownership and native HP contribution remain unchanged')
        projectile.write(work + 1, 1, '<B')
        projectile.camera_record()
        projectile.collision(kind=2)
        projectile.render()
        check(len(projectile.recent()) == 1 and projectile.read(projectile.private('captured_hits')) == 1,
              'Other-player palico projectiles remain excluded by the native producer ownership gate')
        projectile.clock += projectile.frequency
        projectile.write(work + 1, 0, '<B')
        projectile.write(HIT + 0x6d, 9, '<B')
        projectile.camera_record()
        projectile.collision(kind=2)
        projectile.render()
        check(not projectile.text_draws and len(projectile.recent()) == 2,
              'Unclassified local-palico child object remains diagnostic without a popup')
        projectile.write(HIT + 0x6d, subtype, '<B')
        projectile.camera_record()
        projectile.collision(kind=2, damage_type=0x1d)
        projectile.render()
        check(not projectile.text_draws and len(projectile.recent()) == 2,
              'Palico projectile exception cannot bypass healing rejection')
    compact = probe()
    compact.call('forge_onInit')
    check('igTreeNode_Str' in compact.imports.values() and 'igTreePop' in compact.imports.values() and
          not any('CollapsingHeader' in name for name in compact.imports.values()),
          'Plugin uses the same compact TreeNode APIs as Better Input without framed headers')
    compact.tree_open = {}
    compact.changes = {'Number size': 2.0}
    compact.buttons = {'Save settings'}
    calls = compact.mount_calls
    compact.panel()
    check(compact.headers == ['Damage numbers##damage-numbers'] and compact.tree_pops == 0 and
          compact.mount_calls == calls and 'Number size' in compact.changes,
          'Collapsed plugin row omits controls, diagnostics and save actions without a TreePop')
    compact.uc.mem_write(POSITION, struct.pack('<3f', .25, 0, .5))
    compact.camera_record()
    compact.collision()
    compact.render()
    check(len(compact.text_draws) == 9, 'Collapsed plugin row leaves floating damage rendering active')
    compact.changes = {}
    compact.buttons = set()
    compact.tree_open = {'Damage numbers##damage-numbers': True}
    compact.headers = []
    compact.panel()
    check(compact.headers == ['Damage numbers##damage-numbers'] and compact.tree_pops == 1,
          'Open compact settings row balances one scope when diagnostics remain closed')
    compact.tree_open['Diagnostics##damage-numbers'] = True
    compact.panel()
    check(compact.tree_pops == 2, 'Developer tree requests cannot add a diagnostic panel')
    compact.write(compact.private('active'), 0)
    compact.panel()
    check(compact.tree_pops == 3, 'Inactive capture early return still balances the outer tree scope')
    other = probe()
    other.call('forge_onInit')
    other.render()
    other.thread = 1
    other.panel()
    check(other.tree_pops == 1, 'Different render-consumer early return balances the compact plugin tree')
    # Menu/source gates and r8 player-save INI persistence. Files are
    # explicit fixtures, never the host/emulator save or SD directories.
    options = probe()
    options.call('forge_onInit')
    options.panel()
    check(options.headers == ['Damage numbers##damage-numbers'],
          'Compact plugin tree opens with diagnostics collapsed independently')
    check(not options.files and not options.handles,
          'Unavailable player save mount does not open or write settings')
    check(options.fs_context == 0x12340000 and options.fs_guard_depth == 0,
          'Unavailable save mount restores original thread filesystem error policy')
    options.uc.mem_write(POSITION, struct.pack('<3f', .25, 0, .5))
    options.write(HIT + 0x6d, 2, '<B')
    options.camera_record()
    options.collision(kind=1)
    options.render()
    check(len(options.text_draws) == 9 and options.recent()
          [-1][2] == 4, 'Recognized monster damage always displays')
    options.clock += options.frequency
    options.collision_body = gateway['symbols']['dn_probe_collision_shot_body']
    options.write(HIT + 0x30, 0)
    options.write(HIT + 0x6d, 8, '<B')
    options.write(HIT + 0x6e, 0, '<B')
    options.write(HIT + 0x70, options.players[1])
    options.camera_record()
    options.collision(kind=2, owner=1)
    options.render()
    check(len(options.text_draws) == 9 and options.text_draws[-1]['color'] == 0xffecf1f4,
          'Recognized remote-owned generic object always displays with unknown-HZV white')
    options.clock += options.frequency
    captured = options.read(options.private('captured_hits'))
    options.write(HIT + 0x6d, 1, '<B')
    options.camera_record()
    options.collision(kind=2, owner=1)
    options.render()
    check(not options.text_draws and options.read(options.private('captured_hits'))
          == captured, 'Always-on objects never admit remote hunter weapon attacks')
    cat, work = DATA + 0x80000, DATA + 0x81000
    options.write(cat + 0x13c8, 0)
    options.write(cat + 0x3ac4, work)
    options.write(work + 1, 1, '<B')
    options.write(HIT + 0x6e, 3, '<B')
    options.write(HIT + 0x70, cat)
    options.write(HIT + 0x6d, 8, '<B')
    options.camera_record()
    options.collision(kind=2)
    options.render()
    check(not options.text_draws and options.read(options.private('captured_hits')) ==
          captured, "Always-on objects never admit another hunter's Palico weapon attacks")
    check(not set(options.widgets) & {'Gray bounced hits', 'Monster-to-monster damage',
          'Placed bombs and objects'}, 'Obsolete controls are absent from compiled UI')
    primary = 'savedata:/damage_numbers.ini'
    backup = 'savedata:/damage_numbers_backup.ini'
    save = probe()
    save.save_available = True
    unrelated = {'savedata:/better_inputs.ini': b'unchanged Better Input',
                 'savedata:/system': b'unchanged game payload',
                 'dnum:/forge/mhgu-damage-numbers-a.cfg': b'old SD settings ignored',
                 'dnum:/forge/mhgu-damage-numbers-b.cfg': b'old SD backup ignored'}
    save.files = unrelated.copy()
    save.call('forge_onInit')
    check(save.files == unrelated, 'Boot ignores old SD preferences and never creates config files')
    check(not save.save_mounts and not save.handles,
          'Initial preference read releases its counted save mount')
    save.changes = {'Show damage numbers': False,
                    'Style##damage-numbers': 1, 'Number size': 1.5, 'Duration': 1.2}
    save.buttons = {'Save settings'}
    save.panel()
    first_files = save.files.copy()
    check(primary in save.files and backup not in save.files and not save.handles and not save.save_mounts,
          'Save button writes primary INI beside Better Input and releases all handles/mounts')
    text = save.files[primary].decode()
    check('[DamageNumbers]' in text and 'version=' not in text and 'enabled=0' in text and 'style=classic' in text and
          'scale=1.500000' in text and not any(
              key in text for key in ('objects', 'monsters', 'bounce_gray')),
          'Versionless INI stores only visibility, style, size and lifetime')
    check(all(save.files[p] == value for p, value in unrelated.items()),
          'Saving preserves Better Input, game files and old SD configs')
    restarted = probe()
    restarted.save_available = True
    restarted.files = first_files.copy()
    restarted.call('forge_onInit')
    restarted.uc.mem_write(POSITION, struct.pack('<3f', .25, 0, .5))
    restarted.camera_record()
    restarted.collision()
    restarted.render()
    check(not restarted.text_draws,
          'Restart restores the saved disabled preference from the player save container')
    restarted.changes = {'Show damage numbers': True}
    restarted.panel()
    restarted.camera_record()
    restarted.collision()
    restarted.render()
    check(len(restarted.text_draws) == 9 and restarted.text_draws[-1]['pixels'] == 37.5,
          'Loaded size setting remains effective when display is re-enabled')
    legacy = probe()
    legacy.save_available = True
    legacy.uc.mem_write(POSITION, struct.pack('<3f', .25, 0, .5))
    legacy.files = {
        primary: b'[DamageNumbers]\nversion=2\nenabled=0\nstyle=classic\nscale=1.5\nlifetime=1.2\n'}
    legacy.call('forge_onInit')
    legacy.camera_record()
    legacy.collision()
    legacy.render()
    check(not legacy.text_draws, 'Existing version-2 file preserves disabled preference in compiled startup')
    legacy.changes = {'Show damage numbers': True}
    legacy.panel()
    legacy.camera_record()
    legacy.collision()
    legacy.render()
    check(len(legacy.text_draws) == 9 and legacy.text_draws[-1]['pixels'] == 37.5,
          'Existing version-2 file preserves Classic style and size in compiled rendering')
    legacy.buttons = {'Save settings'}
    legacy.panel()
    migrated = legacy.files[primary]
    check(b'version=' not in migrated and b'style=classic' in migrated and b'scale=1.500000' in migrated and b'lifetime=1.200000' in migrated,
          'Compiled explicit save removes obsolete version marker and preserves remaining preferences')
    reread = probe()
    reread.save_available = True
    reread.files = legacy.files.copy()
    reread.call('forge_onInit')
    reread.uc.mem_write(POSITION, struct.pack('<3f', .25, 0, .5))
    reread.camera_record()
    reread.collision()
    reread.render()
    check(len(reread.text_draws) == 9 and reread.text_draws[-1]['pixels'] == 37.5 and not reread.handles and not reread.save_mounts,
          'Compiled restart reads migrated versionless preferences with balanced storage cleanup')
    save.files[backup] = b'legacy backup untouched'
    save.buttons = {'Restore defaults', 'Save settings'}
    save.panel()
    check(save.files[backup] == b'legacy backup untouched' and save.files[primary] != first_files[primary],
          'Repeated save changes only primary and leaves legacy backup unused')
    save.fail_write_path = primary
    save.buttons = {'Save settings'}
    save.panel()
    damaged = save.files.copy()
    check(damaged[backup] == b'legacy backup untouched' and not save.handles and not save.save_mounts,
          'Partial primary write leaves legacy backup untouched and closes all handles')
    recovered = probe()
    recovered.save_available = True
    recovered.files = damaged
    recovered.call('forge_onInit')
    recovered.uc.mem_write(POSITION, struct.pack('<3f', .25, 0, .5))
    recovered.camera_record()
    recovered.collision()
    recovered.render()
    check(len(recovered.text_draws) == 9 and abs(recovered.text_draws[-1]['pixels'] - start_pixels * font_scale) < .001,
          'Restart rejects partial primary and uses defaults without backup recovery')
    recovered.changes = {'Show damage numbers': False}
    recovered.panel()
    recovered.render()
    check(not recovered.text_draws, 'Disabling display immediately clears already visible numbers')
    save.fail_write_path = None
    save.fail_commit = True
    save.buttons = {'Save settings'}
    save.panel()
    check('save failed' in save.string(save.read(save.private('storage_status'))) and not save.save_mounts,
          'Explicit save commit failure reports failure and releases counted mount')
    check(save.save_commits > 0 and set(save.fs_paths) == {primary},
          'Config save commits player save mount and accesses only primary INI')
    check(save.fs_context == 0x12340000 and save.fs_guard_depth == 0,
          'All settings paths restore original SDK thread context')
    late = probe()
    late.files = first_files.copy()
    late.call('forge_onInit')
    late.uc.mem_write(POSITION, struct.pack('<3f', .25, 0, .5))
    check(late.read(late.private('pending'), '<B') == 1,
          'Unavailable startup mount leaves preference load pending')
    late.render()
    calls = late.mount_calls
    late.render()
    check(late.mount_calls == calls, 'Unavailable preference load retries at most once per second')
    late.save_available = True
    late.clock += late.frequency
    late.render()
    check(late.read(late.private('pending'), '<B') == 0 and not late.text_draws,
          'Deferred load succeeds once the game creates its player save storage')
    late.changes = {'Show damage numbers': True}
    late.panel()
    late.camera_record()
    late.collision()
    late.render()
    check(late.text_draws[-1]['pixels'] == 37.5,
          'Deferred load restores the existing INI size preference')
    changed = probe()
    changed.files = first_files.copy()
    changed.call('forge_onInit')
    changed.changes = {'Number size': 2.0}
    changed.panel()
    changed.save_available = True
    changed.render()
    changed.uc.mem_write(POSITION, struct.pack('<3f', .25, 0, .5))
    changed.camera_record()
    changed.collision()
    changed.render()
    check(abs(changed.text_draws[-1]['pixels'] - 2 * start_pixels * font_scale)
          < .001, 'Manual edits before storage availability cancel deferred loading')
    # Execute the original native feedback getter against an actual embedded
    # hunter attack. Skill/status calls remain the explicit existing fixtures.
    for second_active, flags, delay, expected_gray in ((False, 1 << 18, 0, True),
                                                       (True, 1 << 18, 0, False), (False, (1 << 18) | (1 << 27), 0, False), (False, 1 << 18, .11, False)):
        bounce = probe()
        bounce.call('forge_onInit')
        bounce.render()
        player = bounce.players[0]
        attack = player + 0x3020
        bounce.write(player, DATA + 0xa0000)
        bounce.write(player + 0x13c4, DATA + 0xa1000)
        for slot in (0x1b8, 0x208, 0x218):
            bounce.write(DATA + 0xa0000 + slot, RETURN + 4)
        bounce.write(attack + 5, 1, '<B')
        bounce.write(attack + 6, 0, '<B')
        bounce.write(attack + 0x30, flags)
        bounce.write(attack + 0x6d, 1, '<B')
        bounce.write(player + 0x30dd, int(second_active), '<B')
        bounce.write(player + 0x30de, 0 if second_active else 255, '<B')
        bounce.write(player + 0x2522, 12, '<H')
        bounce.uc.mem_write(POSITION, struct.pack('<3f', .25, 0, .5))
        bounce.camera_record()
        bounce.collision(zone=20, attack=attack)
        bounce.render()
        bounce.clock += int(delay * bounce.frequency)
        bounce.call(MAIN + 0x2a87a8, (player,))
        bounce.camera_record()
        bounce.render()
        painted = fill_rows(bounce.text_draws)
        check(len(painted) == 1 and painted[0]['color'] == (0xff9a9a9a if expected_gray else 0xffecf1f4),
              'Only uniquely matching, fresh native bounce feedback colors its exact hit gray')
        if expected_gray:
            bounce.changes = {'Style##damage-numbers': 1}
            bounce.panel()
            bounce.render()
            check(len(bounce.text_draws) ==
                  9 and bounce.text_draws[-1]['color'] == 0xff9a9a9a, 'Confirmed bounce stays gray in Classic style')
            bounce.changes = {'Style##damage-numbers': 0}
            bounce.panel()
            bounce.render()
            check(len(bounce.text_draws) ==
                  9 and bounce.text_draws[-1]['color'] == 0xff9a9a9a, 'Confirmed bounce stays gray in Wilds style')
            bounce.clock += bounce.frequency
            bounce.camera_record()
            bounce.collision(amount=80, zone=20, attack=attack)
            bounce.collision(amount=40, zone=20, attack=attack)
            bounce.call(MAIN + 0x2a87a8, (player,))
            bounce.render()
            painted = fill_rows(bounce.text_draws)
            check({r['text']: r['color'] for r in painted} == {'60': 0xffecf1f4, '30': 0xff9a9a9a},
                  'Native feedback resolves only latest impact of a reused attack without coloring earlier multi-hits')
            matched = bounce.read(bounce.private('bounce_matches'))
            bounce.call(MAIN + 0x2a87a8, (player,))
            bounce.render()
            check(bounce.read(bounce.private('bounce_matches')) == matched,
                  'Repeated native feedback cannot reassign an earlier hit')
            bounce.clock += bounce.frequency
            bounce.camera_record()
            bounce.collision(zone=20, attack=attack)
            bounce.thread = 1
            bounce.call(MAIN + 0x2a87a8, (player,))
            bounce.thread = 0
            bounce.render()
            check(bounce.text_draws[-1]['color'] == 0xffecf1f4,
                  'Feedback on a different native thread cannot claim a hit')
    rise = probe()
    rise.call('forge_onInit')
    rise.panel()
    rise.changes = {'Style##damage-numbers': 2}
    rise.panel()
    rise.uc.mem_write(POSITION, struct.pack('<3f', .25, 0, .5))
    rise.write(HIT + 0x54, 2, '<B')
    rise.camera_record()
    rise.collision(zone=45)
    rise.render()
    check(len(rise.quads) == 1 and len(rise.text_draws) == 9 and not rise.lines,
          'Rise ARM draws one separate tilted slash and thick outlined digits without stars')
    quad = rise.quads[0]
    points = quad['points']
    center_x = (points[0][0] + points[1][0]) * .5 * .5 + (points[2][0] + points[3][0]) * .5 * .5
    center_y = (points[0][1] + points[2][1]) * .5
    check(abs(center_x - 800) < .002 and abs(center_y - 360) < .002,
          'Rise ARM slash originates at registered projection without digit anchor correction')
    check(quad['uv'] == [(0, 0), (1, 0), (1, .5), (0, .5)] and rise.text_draws[-1]['color'] == 0xff1eb4f0,
          'Rise ARM positive slash selects red artwork independently of shared orange tint')
    check(rise.sprite_decodes == rise.texture_registrations == 2,
          'Rise ARM lazily decodes and registers its independent open beam once per context')
    rise.clock += int(.06 * rise.frequency)
    rise.camera_record()
    rise.render()
    check(rise.sprite_decodes == 2 and len(rise.quads) == 1 and abs(rise.quads[0]['points'][2][1] - rise.quads[0]['points'][0][1] - 540) < .002,
          'Rise ARM peak selected three-quarter-height slash uses elapsed-time expansion without repeat decoding')
    slope = (points[0][0] + points[1][0] - points[2][0] - \
             points[3][0]) * .5 / (points[2][1] - points[0][1])
    later = rise.quads[0]['points']
    slope2 = (later[0][0] + later[1][0] - later[2][0] - \
              later[3][0]) * .5 / (later[2][1] - later[0][1])
    check(abs(slope - slope2) < .0001 and abs(slope) <= .25,
          'Rise ARM tilt remains stable throughout beam expansion')
    rise.clock += int(.24 * rise.frequency)
    rise.camera_record()
    rise.render()
    check(not rise.quads and len(rise.text_draws) == 9 and rise.text_draws[-1]['color'] == 0xff1eb4f0,
          'Rise ARM slash expires while outlined orange digits remain opaque and held')
    check(abs(rise.text_draws[-1]['pixels'] - 35 * font_scale) < .001,
          'Rise ARM orange settles at 35 design units')
    rise.save_available = True
    rise.buttons = {'Save settings'}
    rise.panel()
    check(b'style=rise' in rise.files[primary] and b'version=' not in rise.files[primary],
          'Rise ARM settings save style=rise without a version marker')
    restarted = probe()
    restarted.files = rise.files.copy()
    restarted.save_available = True
    restarted.call('forge_onInit')
    restarted.uc.mem_write(POSITION, struct.pack('<3f', 0, 0, .5))
    restarted.write(HIT + 0x54, 1, '<B')
    restarted.camera_record()
    restarted.collision(zone=44)
    restarted.render()
    check(len(restarted.quads) == 1 and restarted.quads[0]['uv'][0] == (0, .5) and restarted.text_draws[-1]['color'] == 0xffffffff,
          'Rise ARM saved-style restart restores white digits and independent violet negative slash')
    check(rise.clip_depth == restarted.clip_depth == 0,
          'Rise ARM effect and gameplay clip scopes balance')
    check(len(rise.context_hooks) == 1, 'Rise ARM registers one context shutdown cleanup with lazy texture')
    context, hook_bytes, callback = rise.context_hooks[0]
    hook_ptr = DATA + 0x1d8000
    rise.uc.mem_write(hook_ptr, hook_bytes)
    rise.call(callback, (context, hook_ptr))
    check(rise.texture_unregistrations == rise.texture_destroyed == 1 and rise.read(rise.private('rise_texture')) == 0 and rise.read(rise.private('font_context')) == 0,
          'Rise ARM shutdown unregisters and frees its CPU texture resource')
    broken_rise = probe()
    broken_rise.call('forge_onInit')
    broken_rise.panel()
    broken_rise.uc.mem_write(broken_rise.address('_ZL14rise_sprite_lz'), b'\x80\x00\x00')
    broken_rise.changes = {'Style##damage-numbers': 2}
    broken_rise.panel()
    broken_rise.uc.mem_write(POSITION, struct.pack('<3f', 0, 0, .5))
    broken_rise.write(HIT + 0x54, 2, '<B')
    broken_rise.camera_record()
    broken_rise.collision()
    broken_rise.render()
    broken_rise.render()
    check(not broken_rise.quads and len(broken_rise.text_draws) == 9 and broken_rise.texture_destroyed == 1,
          'Rise ARM malformed beam falls back to outlined digits and frees unregistered texture')
    check(broken_rise.sprite_decodes == 2 and sum('Rise slash texture unavailable' in x[0] for x in broken_rise.logs) == 1,
          'Rise ARM failed beam is attempted once per context and logs once across repeated frames')
    font = probe()
    font.call('forge_onInit')
    font.call('forge_onImGuiInit', (DATA + 0x1e9000, RETURN, RETURN))
    check(font.font_registrations == font.font_released == 1,
          'Repeated ImGui init registers embedded font once and disables atlas ownership')
    check(font.texture_registrations == font.sprite_decodes == 1,
          'Lossless sprite decode and texture registration run once per ImGui context')
    codec = next(name for name in font.symbols if name.startswith(
        '_ZN14damage_numbers13decode_sprite'))
    codec_source, codec_output = DATA + 0x1d0000, DATA + 0x1d1000

    def codec_case(payload, size):
        font.uc.mem_write(codec_source, payload or b'\0')
        font.uc.mem_write(codec_output - 16, b'\xab' * (size + 32))
        result = font.call(codec, (codec_source, len(payload), codec_output, size))
        intact = (bytes(font.uc.mem_read(codec_output - 16, 16)) == b'\xab' * 16 and
                  bytes(font.uc.mem_read(codec_output + size, 16)) == b'\xab' * 16)
        return result, intact
    for payload, size, label in [(b'\x00', 3, 'truncated literal'),
                                 (b'\x80\x00\x00', 3, 'zero distance'), (b'\x80\x01\x00',
                                                                         3, 'invalid backward copy'),
                                 (b'\xff\x00', 3, 'truncated extended length'), (b'\x80\x01',
                                                                                 3, 'truncated copy distance'),
                                 (b'\x00\x2a\xff\xff\xff\x01\x00', 3, 'output overrun'),
                                 (b'\x00\x2a', 3, 'incomplete output')]:
        result, intact = codec_case(payload, size)
        check(not result and intact, 'Compiled DN-LZ1 rejects ' + \
              label + ' without out-of-bounds writes')
    result, intact = codec_case(b'\x00\x2a\x80\x01\x00', 4)
    check(result and intact and bytes(font.uc.mem_read(codec_output, 4)) == b'\x2a' * 4,
          'Compiled DN-LZ1 handles overlapping copy exactly')
    result, intact = codec_case(b'\x00\x2a\xff\x00\x00\x01\x00', 131)
    check(result and intact and bytes(font.uc.mem_read(codec_output, 131)) == b'\x2a' * 131,
          'Compiled DN-LZ1 handles extended copy exactly')
    broken = CaptureProbe(image, nro_bytes, gateway_bytes, gateway['symbols'])
    broken.uc.mem_write(broken.address('_ZL15wilds_sprite_lz'), b'\x80\x00\x00')
    broken.call('forge_onImGuiInit', (DATA + 0x1e9000, RETURN, RETURN))
    check(broken.texture_registrations == 0 and broken.texture_destroyed == 1 and
          broken.read(broken.private('sprite_texture')) == 0,
          'Invalid embedded stream destroys the unregistered texture and retains crisp-digit fallback')
    check(any('texture unavailable' in entry[0] for entry in broken.logs),
          'Invalid sprite data reports fallback through Forge logging')
    decodes_before_render = font.sprite_decodes
    font.uc.mem_write(POSITION, struct.pack('<3f', 0, 0, .5))
    font.camera_record()
    font.write(HIT + 0x54, 2, '<B')
    font.collision(zone=44)
    font.render()
    check(font.sprite_decodes == decodes_before_render,
          'Damage capture and rendering do not call the sprite decoder')
    check(font.text_draws[-1]['font'] == DATA + 0x1e7000 and font.text_draws[-1]['color'] == 0xffecf1f4 and stars(font)[0]['uv_min'][0] == 0,
          'Nonweak critical uses embedded font, white digits and red star')
    font.clock += int(.8 * font.frequency)
    font.camera_record()
    font.render()
    check(abs((stars(font)[0]['color'] >> 24) - fade_alpha) <= 1 and all(r['color'] >> 24 <=
          fade_alpha for r in font.images), 'Solid and glowing stars fade with their number')
    font.changes = {'Style##damage-numbers': 1}
    font.panel()
    font.render()
    check(len(font.text_draws) == 9 and font.text_draws[-1]['font'] == DATA + 0x1e2000 and len(font.lines) == 2 and not font.triangles,
          'Classic uses Forge font, outlined digits and straight critical lines')
    failure = CaptureProbe(image, nro_bytes, gateway_bytes, gateway['symbols'])
    failure.font_available = False
    failure.call('forge_onImGuiInit', (DATA + 0x1e9000, RETURN, RETURN))
    failure.call('forge_onImGuiInit', (DATA + 0x1e9100, RETURN, RETURN))
    failure.call('forge_onInit')
    check(failure.sprite_decodes == failure.texture_registrations == 2,
          'New ImGui context decodes and registers the texture once again')
    failure.uc.mem_write(POSITION, struct.pack('<3f', 0, 0, .5))
    failure.write(HIT + 0x54, 2, '<B')
    failure.camera_record()
    failure.collision()
    failure.render()
    check(failure.text_draws[-1]['font'] == DATA + 0x1e2000 and len(stars(failure)) == 1 and sum('digit font unavailable' in row[0] for row in failure.logs) == 1,
          'Registration failure falls back to Forge font, retaining Wilds color and stars with one warning')
    dense = probe()
    dense.call('forge_onInit')
    dense.varied_digits = True
    dense.uc.mem_write(POSITION, struct.pack('<3f', 0, 0, .5))
    for batch in range(17):
        for i in range(8):
            dense.write(HIT + 0x54, (batch + i) % 3, '<B')
            dense.collision(amount=148 if i % 2 else 1184)
        dense.camera_record()
        dense.render()
    painted = fill_rows(dense.text_draws)
    check(len(painted) == 128, 'Dense compiled renderer keeps the newest 128 impacts without summing')

    def offsets(p):
        data = bytes(p.uc.mem_read(p.private('placement'), 128 * 28))
        return [(struct.unpack_from('<2f', data, i * 28 + 16), data[i * 28 + 24]) for i in range(128)]
    saved_offsets = offsets(dense)
    check(all(not live or abs(x) <= 32.0001 and abs(y) <= 20.0001 for ((x, y), live)
          in saved_offsets), 'Compiled dense placement stays bounded to 1.6 by 1 font sizes')
    widths = {r['text']: r['position'][0] for r in painted}
    check(set(widths) == {'111', '888'},
          'Nonuniform actual glyph advances are measured for different digit widths')
    dense.clock += int(.05 * dense.frequency)
    dense.camera_record()
    dense.render()
    check(offsets(dense) == saved_offsets, 'Compiled offsets stay stable throughout pop and drift')
    dense.changes = {'Style##damage-numbers': 1}
    dense.panel()
    dense.render()
    check(len(dense.text_draws) == 128 * 9 and not dense.triangles,
          'Classic shares compact placement and the 128-number limit')
    options.changes = {'Show damage numbers': False}
    options.panel()
    options.write(HIT + 0x6d, 2, '<B')
    options.camera_record()
    options.collision(kind=1)
    options.render()
    check(not options.text_draws, 'Master disable suppresses always-on monster impacts')
    options.write(HIT + 0x6d, 8, '<B')
    options.write(HIT + 0x6e, 0, '<B')
    options.write(HIT + 0x70, options.players[1])
    options.collision(kind=2, owner=1)
    options.render()
    check(not options.text_draws, 'Master disable suppresses always-on object impacts')
    aligned = probe()
    aligned.call('forge_onInit')
    aligned.uc.mem_write(POSITION, struct.pack('<3f', .2517, .013, .5))
    aligned.write(HIT + 0x54, 2, '<B')
    aligned.camera_record()
    aligned.collision(zone=44)
    aligned.render()
    fill = fill_rows(aligned.text_draws)[0]
    check(all(value == int(value)
          for value in fill['position']), 'Wilds glyph origin follows ImGui pixel truncation')
    star = stars(aligned)[0]
    center = star['min'][0] + (star['max'][0] - star['min'][0]) * 44.5 / 88
    check(abs(center - (fill['position'][0] + fill['pixels'] * .55)) < .001,
          'Positive star centers on actual rendered glyph bounds at fractional impact coordinates')
    check(len(aligned.text_draws) == 9 and all(row['color'] & 0xffffff == 0 for row in aligned.images if row not in stars(
        aligned)), 'White digits retain centered shadow and eight outline passes without a gold halo')
    aligned.clock += aligned.frequency
    aligned.write(HIT + 0x54, 1, '<B')
    aligned.camera_record()
    aligned.collision(zone=80)
    aligned.render()
    fill = fill_rows(aligned.text_draws)[0]
    check(stars(aligned)[0]['uv_min'][0] == .1875 and abs(stars(aligned)[0]['min'][0] + (stars(aligned)[0]['max'][0] - stars(aligned)[0]['min'][0]) * 44.5 / 88 - (fill['position'][0] + fill['pixels'] * .55)) < .001,
          'Negative star centers below outlined gold glyphs')
    aligned.clock += int(.895 * aligned.frequency)
    aligned.camera_record()
    aligned.render()
    check(len(fill_rows(aligned.text_draws)) == 1 and 0 < aligned.text_draws[-1]['color'] >> 24 < 10,
          'Very faint final fill remains identifiable separately from outline passes')
    old = probe()
    old.save_available = True
    old.files = {
        primary: b'[DamageNumbers]\nversion=1\nenabled=0\nmonsters=1\nobjects=1\nbounce_gray=0\nscale=2\nlifetime=1.8\n'}
    old.files[backup] = old.files[primary]
    old_files = old.files.copy()
    old.call('forge_onInit')
    old.uc.mem_write(POSITION, struct.pack('<3f', 0, 0, .5))
    old.camera_record()
    old.collision()
    old.render()
    check(old.files == old_files and len(old.text_draws) == 9 and abs(
        old.text_draws[-1]['pixels'] - start_pixels * font_scale) < .001, 'Version-1 primary and backup are ignored without migration, deletion or automatic writes')
    old.buttons = {'Save settings'}
    old.panel()
    check(b'version=' not in old.files[primary] and b'style=wilds' in old.files[primary] and old.files[backup] ==
          old_files[backup], 'Explicit Save replaces invalid primary with versionless defaults and leaves old backup untouched')
    p = probe()
    p.call('forge_onInit')
    check(p.read(p.private('active')) == 1 and p.allocations == p.tls_allocations ==
          1, 'Compiled adapter installs one arena and one native TLS slot')
    for i, offset in enumerate(manifest['hook_offsets']):
        check(p.read(MAIN + int(offset, 16)) == 0xe51ff004 and
              p.read(MAIN + int(offset, 16) + 4) == p.address('dn_gateway_' + str(i)), 'Exact private gateway publication')
    for kind in (0, 3):
        for local in range(4):
            for owner in range(4):
                before = p.read(p.private('consumed'))
                p.write(p.private('recent_count'), 0)
                amount = p.collision(owner=owner, local=local, kind=kind)
                p.consume()
                records = p.recent()
                check(amount == 75 and p.read(PARAM + 0x48e, '<h') == 75,
                      'Compiled capture leaves original native damage and accumulation unchanged')
                check(len(records) == (1 if local == owner else 0),
                      'Compiled hunter/palico filtering across all local-session slots')
                if records:
                    r = records[0]
                    check(r[10:15] == (75, 45, 12.0, 34.0, 56.0),
                          'Compiled capture retains damage, unweighted HZV and actual impact vector')
                    check(r[2] == (0 if kind == 0 else 1),
                          'Compiled record identifies local hunter/Prowler versus local palico')
    for damage_type, amount in ((0x1d, 100), (0, -50), (0, 0), (0, 1)):
        p.write(p.private('recent_count'), 0)
        result = p.collision(amount=amount, damage_type=damage_type)
        p.consume()
        check(result == (max(1, int(f32(amount * .75))) if amount > 0 else amount),
              'Healing/zero/low-damage capture leaves native outcome intact')
        check(len(p.recent()) == (1 if damage_type != 0x1d and amount > 0 else 0),
              'Compiled adapter rejects healing and nonpositive contributions')
    p.write(p.private('recent_count'), 0)
    for zone in (44, 45, 46):
        p.collision(amount=17, zone=zone)
    p.consume()
    r = p.recent()
    check([x[11] for x in r] == [44, 45, 46], 'Compiled adapter preserves physical HZV color boundary')
    check(len({x[1] for x in r}) == 3 and all(x[10] == 12 for x in r),
          'Same-frame impacts retain distinct serials and individual native contributions')
    p.collision_body = gateway['symbols']['dn_probe_collision_shot_body']
    for zone, flags, expected in ((44, 1, 44), (45, 1, 45), (80, 1, 80), (45, 0, -1)):
        p.write(p.private('recent_count'), 0)
        p.write(HIT + 0x30, flags)
        p.write(DATA + 0x92002, zone, '<B')
        amount = p.collision(amount=100)
        p.consume()
        check(amount == 75 and p.recent(
        )[-1][11] == expected, 'Compiled shot adapter captures physical HZV only when native shot HZV applies')
    p.collision_body = gateway['symbols']['dn_probe_collision_body']
    player = p.players[0]
    p.write(DATA + 0xe0030, 0)
    p.write(player, DATA + 0xa0000)
    p.write(player + 0x13c4, DATA + 0xa1000)
    for slot in (0x1b8, 0x208, 0x218):
        p.write(DATA + 0xa0000 + slot, RETURN + 4)
    p.write(player + 0x3025, 1, '<B')
    p.write(player + 0x3026, 0, '<B')
    p.write(player + 0x3050, 1 << 18)
    p.write(player + 0x3074, 2, '<B')
    p.write(player + 0x30dd, 0, '<B')
    p.write(player + 0x30de, 255, '<B')
    p.write(player + 0x3108, 1 << 27)
    p.write(player + 0x2522, 12, '<H')
    check(p.call(MAIN + 0x2a87a8, (player,)) == 0,
          'Compiled feedback gateway preserves original resolved bounce outcome')
    # Inspect the published mailbox payload before consuming it. Optimized
    # settings-only builds eliminate the unused last_feedback debug copy.
    expected = struct.pack('<IIBBBBIIBBBBI', player + 0x3020, 1 << 18, 1, 0,
                           2, 0, player + 0x30d8, 1 << 27, 0, 255, 0, 0, 12)
    context_symbol = '_ZN14damage_numbersL8contextsE'
    published = bytes(p.uc.mem_read(p.private('contexts'), p.symbols[context_symbol][1]))
    check(expected in published,
          'Compiled feedback record retains both native attack identities, flags, outcomes and counter for correlation')
    p.consume()
    for _ in range(70):
        p.call(MAIN + 0x2a87a8, (player,))
    before = p.read(p.private('consumed'))
    p.consume()
    check(p.read(p.private('consumed')) - before == 64,
          'Render callback bounds total feedback/diagnostic drain to 64 records')
    p.consume()
    check(p.read(p.private('consumed')) - before == 70,
          'Feedback records beyond the callback budget remain queued for the next callback')
    # Camera copy is a compiled scope/tap fixture. The real viewport and GPU
    # uploader are not executed here, so active-camera/render-scale acceptance
    # remains a live gate.
    frame, viewport, camera, vm, pm, rect = DATA + 0xf0000, DATA + \
        0xf1000, DATA + 0xf2000, DATA + 0xf3000, DATA + 0xf3100, DATA + 0xf3200
    p.uc.mem_write(frame, bytes(328))
    p.write(frame + 324, RETURN)
    p.write(frame + 272, viewport)
    p.write(viewport + 0x24, camera)
    p.write(camera, MAIN + 0x1730d0c)
    view = tuple(float(i) for i in range(16))
    projection = tuple(float(i + 100) for i in range(16))
    p.uc.mem_write(vm, struct.pack('<16f', *view))
    p.uc.mem_write(pm, struct.pack('<16f', *projection))
    p.uc.mem_write(rect, struct.pack('<4i', 10, 20, 1290, 740))
    p.call('dn_gateway_enter', (3, frame))
    p.write(frame + 276, vm)
    p.write(frame + 280, pm)
    p.write(frame + 284, rect)
    p.call('dn_gateway_tap', (4, frame))
    p.call('dn_gateway_leave', (frame,))
    p.consume()
    raw = p.private('last_camera')
    check(tuple(struct.unpack('<16f', p.uc.mem_read(raw + 60, 64))) == view and
          tuple(struct.unpack('<16f', p.uc.mem_read(raw + 124, 64))) == projection and
          tuple(struct.unpack('<4i', p.uc.mem_read(raw + 188, 16))) == (10, 20, 1290, 740),
          'Compiled scoped upload copies final view/projection/rectangle fields')
    p.write(camera, 0)
    p.write(frame + 272, viewport)
    p.call('dn_gateway_enter', (3, frame))
    p.call('dn_gateway_tap', (4, frame))
    p.call('dn_gateway_leave', (frame,))
    p.consume()
    check(p.read(raw + 12) == 0 and p.read(raw + 8) == camera,
          'Non-quest camera cannot replace the scoped camera candidate')
    # Exercise real compiled scope-capacity quarantine without inventing a return.
    frames = [DATA + 0x100000 + i * 336 for i in range(65)]
    for f in frames:
        p.uc.mem_write(f, bytes(328))
        p.write(f + 324, RETURN)
        p.call('dn_gateway_enter', (6, f))
    check(p.read(p.private('scope_drops')) >= 1, 'Compiled scope overflow is counted')
    check(p.read(frames[-1] + 324) == RETURN, 'Overflow retains the original caller lr')
    for f in reversed(frames[:-1]):
        check(p.call('dn_gateway_leave', (f,)) == RETURN,
              'Compiled scopes unwind to their original caller')
    check(p.read(p.private('contexts')) == 0, 'Compiled scope quarantine clears after outermost return')
    p.collision(amount=100)
    p.consume()
    check(p.recent()[-1][10] == 75, 'Capture resumes after bounded scope quarantine')
    for thread in range(1, 17):
        p.thread = thread
        p.collision(amount=100)
    check(p.read(p.private('pool_drops')) ==
          1 and p.tls[16] == 1, 'Thread-pool exhaustion installs a per-thread bypass sentinel')
    allocations = p.read(p.private('claimed'))
    p.collision(amount=100)
    check(p.read(p.private('claimed')) == allocations,
          'Exhausted thread does not retry allocation on every hit')
    p.thread = 0
    p.call('forge_onUpdate', (0,))
    check(p.read(p.private('active')) == 1, 'Compiled monitor accepts unchanged installed sites')
    p.write(MAIN + 0x170968 + 12, 0)
    p.call('forge_onUpdate', (0,))
    check(p.read(p.private('active')) == 0,
          'External code modification disables capture without hot unpatching')
    report = {'kind': 'Compiled capture NRO in Unicorn with explicit SDK/kernel and collision-body fixtures; not Eden',
              'nro_sha256': manifest['nro_sha256'], 'image_sha256': IMAGE_SHA, 'checks': len(checks), 'passed': checks,
              'stubs': ['SDK TLS/clock, Forge counted save mount/release, SDK save filesystem and libc imports', 'ImGui IO/font/draw calls record ARM32 arguments; no GPU rendering',
                        'Kernel query/info/CodeMemory/cache bridges',
                        'Synthetic collision body routes to copied native HZV/normalizer', 'Native monster-state services return explicit fixture values'],
              'limits': ['No original live collision traversal or SDK execution', 'No native element/raw assembly acceptance',
                         'No live thread scheduling, active camera selection, per-hit bounce or Eden acceptance']}
    (OUT / 'compiled-capture-probe.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f'Passed {len(checks)} compiled capture NRO checks; explicit fixtures only, no Eden trial')


if __name__ == '__main__':
    main()
