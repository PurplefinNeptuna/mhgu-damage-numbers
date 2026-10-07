from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path, BUILD_ROOT, WORK_ROOT
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
    from prepare_damage_numbers_rise import prepare as prepare_rise
    prepare_rise()
    output = WORK_ROOT/'open-visual'
    output.mkdir(parents=True, exist_ok=True)
    stub = output / 'forge/log.h'
    stub.parent.mkdir(exist_ok=True)
    stub.write_text('extern "C" void forge_log_info(const char*,...);\n#define forge_log_warn forge_log_info\n')
    cimgui = DEPS / 'libs/cimgui'
    sources = [MOD / 'tests/visual.cpp', MOD / 'tests/rise_visual.cpp',
               MOD / 'src/cosmetics.cpp', cimgui / 'cimgui.cpp']
    sources += [cimgui / 'imgui' / name for name in
                ('imgui.cpp', 'imgui_draw.cpp', 'imgui_tables.cpp',
                 'imgui_widgets.cpp', 'imgui_demo.cpp')]
    includes = [output, MOD / 'src',
                work, cimgui]
    subprocess.run(['g++', '-std=c++17', '-O2', '-DDAMAGE_NUMBERS_HOST_VISUAL',
                    *['-I' + str(p) for p in includes], *map(str, sources),
                    '-o', str(output / 'render')], check=True, cwd=ROOT)
    for bg in ('dark','bright'):
        for height in ('half','three-quarter'):
            for frame in (0,1,2,4,6,9,21,23,26,27):
                (output/f'rise-{bg}-{height}-{frame}').mkdir(exist_ok=True)
    subprocess.run([str(output / 'render'),
                    str(FORGE / 'data/fonts/Roboto-Medium.ttf'),
                    str(output)], check=True, cwd=ROOT)
    import runpy
    runpy.run_path(str(MOD/"tests/compositor.py"))["run"]()
    render(output)
    from PIL import Image, ImageDraw
    import hashlib,json
    scenes={}
    for bg in ('dark','bright'):
        for height in ('half','three-quarter'):
            for frame in (0,1,2,4,6,9,21,23,26,27):
                scene=output/f'rise-{bg}-{height}-{frame}'
                preview=render(scene)
                scenes[scene.name]={'draw_sha256':hashlib.sha256((scene/'draw.json').read_bytes()).hexdigest(),
                    'preview_sha256':hashlib.sha256(preview.read_bytes()).hexdigest(),'age_seconds':frame/30}
        sheet=Image.new('RGB',(2560,760),'#20282a');d=ImageDraw.Draw(sheet)
        for x,height,label in ((0,'half','1/2 viewport height'),(1280,'three-quarter','3/4 viewport height')):
            sheet.paste(Image.open(output/f'rise-{bg}-{height}-2/preview.png'),(x,40));d.text((x+20,12),label,fill='white')
        sheet.save(output/f'rise-height-comparison-{bg}.png')
        # All frames are real renderer output; labels live outside the viewport.
        film=Image.new('RGB',(1280*5,760*2),'#20282a');d=ImageDraw.Draw(film)
        for j,frame in enumerate((0,1,2,4,6,9,21,23,26,27)):
            x=(j%5)*1280;y=(j//5)*760
            film.paste(Image.open(output/f'rise-{bg}-three-quarter-{frame}/preview.png'),(x,y+40))
            d.text((x+20,y+12),f'{frame/30:.3f} seconds',fill='white')
        film.save(output/f'rise-timeline-{bg}.png')
    (output/'rise-preview-manifest.json').write_text(json.dumps({'status':'PASSED','renderer':'Pinned real ImGui draw lists; triangle UV/RGBA interpolation and command clipping','scenes':scenes,'runtime_test':'NOT RUN'},indent=2)+'\n')

if __name__ == '__main__':
    main()
