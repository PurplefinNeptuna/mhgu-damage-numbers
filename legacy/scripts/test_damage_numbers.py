#!/usr/bin/env python3
"""Host policy and presentation checks. No Eden or in-game validation."""
from pathlib import Path
import os
import subprocess
import argparse

from paths import ROOT, MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path, WORK_ROOT

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--thread-sanitizer',action='store_true',help='Also exercise mailbox concurrency with ThreadSanitizer')
    args=parser.parse_args()
    out = WORK_ROOT/'tests'
    out.mkdir(parents=True, exist_ok=True)
    src = MOD
    exe = out / 'core'
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', '-O1', '-g',
                    '-fsanitize=address,undefined', '-fno-omit-frame-pointer',
                    '-I'+str(src/'src'), str(src/'tests/core.cpp'), '-o', str(exe)], check=True)
    # LeakSanitizer cannot enumerate threads inside this ptraced sandbox.
    # Core has no heap allocation; retain address/undefined-behavior checks.
    env = os.environ.copy()
    env['ASAN_OPTIONS'] = env.get('ASAN_OPTIONS', '') + ':detect_leaks=0'
    subprocess.run([str(exe)], check=True, env=env)
    mailbox = out/'mailbox'
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', '-O1', '-g',
                    '-fsanitize=address,undefined', '-fno-omit-frame-pointer', '-pthread',
                    '-I'+str(src/'src'), str(src/'tests/mailbox.cpp'), '-o', str(mailbox)], check=True)
    subprocess.run([str(mailbox)], check=True, env=env, timeout=60)
    transaction=out/'transaction'
    subprocess.run(['c++','-std=c++17','-Wall','-Wextra','-Werror','-O1','-g',
                    '-fsanitize=address,undefined','-fno-omit-frame-pointer',
                    '-I'+str(src/'src'),str(src/'tests/transaction.cpp'),'-o',str(transaction)],check=True)
    subprocess.run([str(transaction)],check=True,env=env)
    mock=out/'mock/forge';mock.mkdir(parents=True,exist_ok=True)
    (mock/'fs.h').write_text('#pragma once\nbool forge_fs_mountSaveData();\nvoid forge_fs_unmountSaveData();\n')
    (mock/'log.h').write_text('#pragma once\nvoid forge_log_warn(const char*,...);\nvoid forge_log_info(const char*,...);\n')
    settings=out/'settings'
    subprocess.run(['c++','-std=c++17','-Wall','-Wextra','-Werror','-O1','-g',
                    '-fsanitize=address,undefined','-fno-omit-frame-pointer','-fno-pie','-no-pie',
                    '-I'+str(mock.parent),str(src/'tests/settings.cpp'),'-o',str(settings)],check=True)
    subprocess.run([str(settings)],check=True,env=env)
    if args.thread_sanitizer:
        tsan=out/'mailbox-tsan'
        subprocess.run(['c++','-std=c++17','-Wall','-Wextra','-Werror','-O1','-g',
                        '-fsanitize=thread','-pthread','-I'+str(src/'src'),
                        str(src/'tests/mailbox.cpp'),'-o',str(tsan)],check=True)
        subprocess.run([str(tsan)],check=True,timeout=60)

if __name__ == '__main__':
    main()
