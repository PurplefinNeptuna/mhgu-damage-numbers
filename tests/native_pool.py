#!/usr/bin/env python3
"""Compile pinned Forge hook.c with host doubles for platform operations only."""
import json
import re
import os
import subprocess
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from paths import MOD, WORK_ROOT, BUILD_ROOT


def main():
    folder = WORK_ROOT / 'native-pool'
    folder.mkdir(parents=True, exist_ok=True)
    source = (MOD / 'research/native-hook.c').read_text()
    # The host cannot execute ARM DSB/ISB. Keep allocator, hook writes, cache
    # calls and error paths verbatim; the ARM gateway suite runs separately.
    source, barriers = re.subn(r'__asm__ volatile\("(?:dsb sy|isb)" ::: "memory"\);', '', source)
    assert barriers == 2
    (folder / 'native-hook-host.c').write_text(source)
    (folder / 'mock.h').write_text('''#pragma once
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <stdlib.h>
typedef uint8_t u8;
typedef uint32_t u32;
typedef int TlsSlot;
typedef uint32_t Result;
typedef struct { void* rw_addr; void* rx_addr; size_t size; int type; } Jit;
#define PAGE_SIZE 4096
#define JitType_CodeMemory 1
#define FORGE_NNOS_MUTEX_SIZE 0x14
#define R_FAILED(r) ((r)!=0)
#define KERNELRESULT(value) 1
#define Module_Libnx 0
#define LibnxError_BadInput 1
#define MAKERESULT(module,error) 1
''')
    for name in ['switch.h', 'forge/log.h', 'forge/types.h', 'forge/wrapper.h',
                 'forge/nn/os.h', 'switch/arm/cache.h', 'switch/kernel/svc.h']:
        header = folder / name
        header.parent.mkdir(parents=True, exist_ok=True)
        header.write_text('#include "mock.h"\n')
    (folder / 'forge/hook.h').write_text('#include "native-hook.h"\n')
    executable = folder / 'native-pool'
    subprocess.run(['cc', '-std=c11', '-fsanitize=address,undefined', '-fno-omit-frame-pointer',
                    '-g', '-I' + str(folder), '-I' + str(MOD / 'research'),
                    str(MOD / 'tests/native-pool.c'), '-o', str(executable)], check=True)
    # LeakSanitizer cannot inspect threads under the sandbox's tracer. ASan and
    # UBSan remain enabled; the fixture accounts for every mock JIT allocation.
    env = dict(os.environ)
    env['ASAN_OPTIONS'] = env.get('ASAN_OPTIONS', '') + ':detect_leaks=0'
    subprocess.run([executable], check=True, env=env)
    BUILD_ROOT.mkdir(parents=True, exist_ok=True)
    (BUILD_ROOT / 'native-pool.json').write_text(json.dumps({
        'status': 'PASSED', 'source': 'research/native-hook.c',
        'ordinary_hooks': 11, 'slot_bytes': 176, 'chunks_for_eleven_hooks': 1,
        'chunk_bytes': 16384, 'max_chunks': 2, 'capacity_hooks': 2048,
        'allocation_failure_preserves_target': True,
        'fallback_rejected': True, 'full_pool_preserves_target': True,
        'sanitizers': ['ASan', 'UBSan'], 'leaks': 'mock JIT allocations counted; LSan disabled under tracer',
        'runtime_test': 'NOT RUN',
        'scope': 'pinned native C allocator with host platform doubles and omitted ARM barriers',
    }, indent=2) + '\n')


if __name__ == '__main__':
    main()
