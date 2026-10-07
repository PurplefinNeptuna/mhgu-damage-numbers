from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
#!/usr/bin/env python3
"""Run the cosmetic renderer with pinned real ImGui, then composite its draw list."""
from pathlib import Path
import subprocess
from prepare_damage_numbers_sprites import prepare
from render_damage_numbers_visual import render

from paths import ROOT, MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path

def main():
    from damage_numbers_assets import argument, paths
    profile=argument();work,_=paths(profile)
    prepare(profile)
    output = ROOT / 'work/damage-numbers/open-visual'
    output.mkdir(parents=True, exist_ok=True)
    stub = output / 'forge/log.h'
    stub.parent.mkdir(exist_ok=True)
    stub.write_text('extern "C" void forge_log_info(const char*,...);\n')
    cimgui = DEPS / 'libs/cimgui'
    sources = [MOD / 'tests/visual.cpp',
               MOD / 'src/cosmetics.cpp', cimgui / 'cimgui.cpp']
    sources += [cimgui / 'imgui' / name for name in
                ('imgui.cpp', 'imgui_draw.cpp', 'imgui_tables.cpp',
                 'imgui_widgets.cpp', 'imgui_demo.cpp')]
    includes = [output, MOD / 'src',
                work, cimgui]
    subprocess.run(['g++', '-std=c++17', '-O2', '-DDAMAGE_NUMBERS_HOST_VISUAL',
                    *['-I' + str(p) for p in includes], *map(str, sources),
                    '-o', str(output / 'render')], check=True, cwd=ROOT)
    subprocess.run([str(output / 'render'),
                    str(FORGE / 'data/fonts/Roboto-Medium.ttf'),
                    str(output)], check=True, cwd=ROOT)
    render(output)

if __name__ == '__main__':
    main()
