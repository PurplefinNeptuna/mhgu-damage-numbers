from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
#!/usr/bin/env python3
"""Reproduce the empty-flag/font collision in a pinned Eden layout model.

Uses file metadata from the copied dump and listed packages, not activation
proof. Executes the host standard-library sort and the relevant pinned layout
and concatenation decisions. It does not run the full emulator or guest SDK.
"""
from pathlib import Path
import hashlib
import json
import os
import subprocess

from paths import ROOT, MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
WORK=ROOT/'work/damage-numbers/romfs-flag-test'
EDEN_REV='d16735f5b6'
SOURCES={
    'fsmitm_romfsbuild.cpp':'c0863dab013176cbd9ea8f4f7e20bb48a968d70f10544d6b3c450710c5c6959f',
    'vfs/vfs_concat.cpp':'90cb0107348f2a5678e2538df6865c00a9daf4097135fe26b976fc27cdba9d1b',
}


def main():
    files={}
    dump=Path('~/.local/share/eden/dump/0100770008DD8000/romfs')
    load=ROOT/'work/inputs/packages'
    for root in [dump]+sorted(p/'romfs' for p in load.iterdir() if (p/'romfs').is_dir()):
        for parent,dirs,names in os.walk(root):
            for name in names:
                p=Path(parent)/name;files[p.relative_to(root).as_posix()]=p.stat().st_size
    flag='nativeNX/forge/damage-numbers-capture.enabled'
    files[flag]=0 # Reproduce even after the installed flag has been corrected.
    WORK.mkdir(parents=True,exist_ok=True)
    table=WORK/'file-layout.tsv'
    table.write_text(''.join(f'{size}\t{path}\n' for path,size in sorted(files.items())))
    source=MOD / 'tests/romfs_flag.cpp'
    exe=WORK/'romfs-flag-test'
    subprocess.run(['c++','-std=c++17','-Wall','-Wextra','-Werror','-O2',str(source),'-o',str(exe)],check=True)
    font=load/'forge-patched/romfs/nativeNX/forge/fonts/Roboto-Medium.ttf'
    stdout=subprocess.check_output([str(exe),str(table),str(font)],text=True)
    cases=[json.loads(line) for line in stdout.splitlines()]
    report={'kind':'Pinned Eden layout/concatenation model; not a live emulator test',
            'eden_revision':EDEN_REV,'listed_overlay_file_count':len(files),'cases':cases,
            'layout_sha256':hashlib.sha256(table.read_bytes()).hexdigest(),
            'font_sha256':hashlib.sha256(font.read_bytes()).hexdigest(),
            'upstream_sources':{f'https://github.com/eden-emulator/mirror/blob/{EDEN_REV}/src/core/file_sys/{name}':sha
                                for name,sha in SOURCES.items()},
            'limits':['Package presence is not activation proof','Full RomFS/guest filesystem and Eden not executed']}
    out=ROOT/'build/damage-numbers/open-assets/romfs-flag-probe.json'
    out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(report,indent=2)+'\n')
    print('Reproduced empty flag shadowing font with padding; absent, 1-, 2- and 8-byte flags preserve the font')


if __name__=='__main__': main()
