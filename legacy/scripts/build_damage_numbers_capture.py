from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path, BUILD_ROOT, WORK_ROOT
#!/usr/bin/env python3
"""Build and inspect the automatically active damage-number NRO; never install it."""
from pathlib import Path
import hashlib
import json
import os
import re
import shutil
import subprocess
import mhgu_binary as baseline
from prepare_damage_numbers_font import prepare
from prepare_damage_numbers_sprites import prepare as prepare_sprites
from native_support import nso_exports
from mhgu_binary import Nro

from paths import ROOT, MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
RECIPE = MOD
NATIVE = WORK_ROOT/'open-assets'
OUT = BUILD_ROOT/'open-assets'
SDK_SHA = '9a9b7035266637ee693a32da6b0fde54bda00e897bef05b1c8d8cda30f8ac8b0'
OFFSETS = (0x170968,0x9d6f8,0x2a87a8,0xb8909c,0x885844,0x16dd7c,0x16dc6c,0x178660,
           0x172e52c,0x276278,0x285c6c)
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    from damage_numbers_assets import argument, paths
    global NATIVE, OUT
    profile=argument()
    NATIVE, OUT=paths(profile)
    from paths import check_dependencies
    check_dependencies()
    revision=baseline.REVISION
    forge=FORGE
    main_path = WORK_ROOT/'inputs/main'
    image,_ = baseline.main_image(main_path.read_bytes())
    memory_path=WORK_ROOT/'inputs/main-memory.bin'
    if memory_path.exists() and memory_path.read_bytes()!=image:
        raise ValueError('Preserving differing normalized main input')
    memory_path.write_bytes(image)
    rows = re.findall(r'\{(0x[0-9a-f]+), \{([^}]+)\}\}', (RECIPE/'src/signatures.h').read_text())
    if tuple(int(offset,16) for offset,_ in rows) != OFFSETS: raise ValueError('Unexpected signature recipe')
    for offset, values in rows:
        expected = bytes(int(x,16) for x in values.split(',')); a = int(offset,16)
        if len(expected)!=64 or image[a:a+64]!=expected: raise ValueError(f'Original signature differs at {offset}')
    sdk = input_path('sdk')
    live_forge=input_path('forge')
    if sha(sdk)!=SDK_SHA: raise ValueError('SDK fingerprint changed')
    font_manifest=prepare(profile)
    sprite_manifest=prepare_sprites(profile)
    from prepare_damage_numbers_rise import prepare as prepare_rise
    rise_manifest=prepare_rise()
    NATIVE.mkdir(parents=True,exist_ok=True)
    command = """set -eu
export PATH=/opt/devkitpro/devkitARM/bin:$PATH
arm-none-eabi-g++ --version > /output/compiler.txt
flags='-DMHGU_DIAGNOSTICS={diagnostics} -Wall -Wextra -Werror -O2 -fPIC -marm -march=armv8-a+crc+crypto -mfloat-abi=softfp -mfpu=crypto-neon-fp-armv8 -fno-exceptions -fno-rtti -fno-unwind-tables -fno-asynchronous-unwind-tables -fno-threadsafe-statics'
for unit in plugin capture settings cosmetics; do
 arm-none-eabi-g++ -std=c++17 $flags -I/output -include /mod/vendor/forge/include/compat.h -I/mod/vendor/forge/include -I/mod/vendor/forge/libs/nnsdk/include -isystem /mod/vendor/sdk/libs/libnx/nx/include -isystem /mod/vendor/sdk/libs/cimgui -c /mod/src/$unit.cpp -o /output/$unit.o
done
arm-none-eabi-g++ -std=c++17 $flags -isystem /mod/vendor/sdk/libs/libnx/nx/include -c /mod/vendor/modern-weapon-controls/src/native_platform.cpp -o /output/native_platform.o
for unit in start gateway; do
 arm-none-eabi-gcc -fPIC -marm -march=armv8-a+crc+crypto -mfpu=crypto-neon-fp-armv8 -c /mod/src/$unit.S -o /output/$unit.o
done
arm-none-eabi-gcc -fPIC -marm -c /mod/vendor/modern-weapon-controls/src/svc.S -o /output/svc.o
arm-none-eabi-g++ -shared -nostdlib -Wl,--hash-style=sysv,-z,max-page-size=4096,-z,noexecstack,-init=init,-fini=fini -Wl,--version-script=/mod/src/exports.txt -Wl,-T,/mod/src/plugin.ld /output/start.o /output/plugin.o /output/capture.o /output/settings.o /output/cosmetics.o /output/native_platform.o /output/gateway.o /output/svc.o -o /output/plugin.elf
arm-none-eabi-readelf -aW /output/plugin.elf > /output/readelf.txt
arm-none-eabi-nm -n /output/plugin.elf > /output/symbols.txt
arm-none-eabi-objdump -d /output/plugin.elf > /output/disassembly.txt
"""
    args = ['podman','--root',CONTAINER_STORAGE,'--runroot',
            f'/tmp/mhgu-forge-container-{os.getuid()}','--storage-driver','vfs','run','--pull=never','--rm',
            '--network=none','-v',f'{MOD}:/mod:ro','-v',f'{ROOT}:/workspace:ro','-v',f'{NATIVE}:/output:rw',
            baseline.IMAGE,'bash','-lc',command.replace('{diagnostics}', os.environ.get('MHGU_DIAGNOSTICS', '0'))]
    result=subprocess.run(list(map(str,args)),capture_output=True,text=True)
    (NATIVE/'build.log').write_text(result.stdout+result.stderr)
    if result.returncode: raise RuntimeError(f'Capture build failed; inspect {NATIVE/"build.log"}')
    if '_GLOBAL__sub_I_' in (NATIVE/'symbols.txt').read_text() or '.init_array' in (NATIVE/'readelf.txt').read_text():
        raise ValueError('Unexpected dynamic C++ initializers')
    nro_bytes=baseline.elf_to_nro((NATIVE/'plugin.elf').read_bytes())
    nro=Nro(nro_bytes)
    imports={s[0].decode('ascii') for s in nro.symbols if s[0] and s[1][5]==0}
    exports={s[0].decode('ascii') for s in nro.symbols if s[0] and s[1][5]!=0}
    callbacks={'forge_onLoad','forge_onInit','forge_onUpdate','forge_onImGuiInit','forge_onImGuiRender','forge_onImGuiFreeRender'}
    if not callbacks<=exports: raise ValueError('Missing Forge callbacks')
    available=nso_exports(live_forge)|nso_exports(main_path)|nso_exports(sdk)
    if imports-available: raise ValueError(f'Unresolved imports: {sorted(imports-available)}')
    settings_save_imports={'forge_fs_mountSaveData','forge_fs_unmountSaveData','_ZN2nn2fs14CommitSaveDataEPKc'}
    if not settings_save_imports<=imports: raise ValueError('Missing counted save-mount/config commit APIs')
    if any(n.startswith(('forge_hook_','jit','svc','forge_fs_fileExists','forge_fs_getRomFsMountPoint')) or
           ('SaveData' in n and n not in settings_save_imports) or 'MountSdCard' in n for n in imports):
        raise ValueError('Capture unexpectedly imports shared hooks, activation/SD APIs or unrelated save APIs')
    OUT.mkdir(parents=True,exist_ok=True)
    target=OUT/'candidate-package/romfs/nativeNX/plugins/damage_numbers_capture.nro'
    target.parent.mkdir(parents=True,exist_ok=True)
    if target.exists() and target.read_bytes()!=nro_bytes:
        old=OUT/'previous-candidates'/sha(target)
        old.mkdir(parents=True,exist_ok=True); shutil.copy2(target,old/target.name)
    target.write_bytes(nro_bytes)
    shutil.copy2(RECIPE/'fonts/OFL.txt',OUT/'candidate-package/FONT_LICENSE.txt')
    (OUT/'candidate-package/OPEN_ASSETS.txt').write_text(
        'Supported build: OFL Open Sans digits and independently authored crit sprites.\n'
        'No extracted Wilds or Rise font or texture is embedded.\n')
    for name in ('Roboto-OFL.txt','RobotoCondensed-OFL.txt'):
        old_license=OUT/'candidate-package'/name
        if old_license.exists():
            if sha(old_license) not in {'061402327a96aadb0bfb694a960ed289ecd38d383e396243831ab81feb109c41','0e4cc6ece88573545be2ed25835363662a6182ba4a4c1b5c8feda52add30e8a6'}:
                raise ValueError('Unexpected previous generated font license')
            old_license.unlink()
    # Retire only the generated flag templates from earlier test builds.
    for name in ('damage-numbers-capture.enabled','damage-numbers-camera.enabled'):
        legacy=OUT/'activation'/name
        if legacy.exists():
            if legacy.read_bytes()!=b'1\n':
                raise ValueError(f'Unexpected contents in legacy generated flag: {legacy}')
            legacy.unlink()
    # Empty RomFS files can overlap the following font in the tested Eden build.
    for p in (OUT/'candidate-package').rglob('*'):
        if p.is_file() and not p.stat().st_size:
            raise ValueError(f'Unsafe empty RomFS package file: {p}')
    for name in ('damage-numbers-capture.enabled','damage-numbers-camera.enabled'):
        if (OUT/'candidate-package/romfs/nativeNX/forge'/name).exists():
            raise ValueError('Automatic-start candidate must not contain obsolete activation files')
    sources=sorted(p for p in RECIPE.rglob('*') if p.is_file() and not any(x in p.relative_to(RECIPE).parts for x in ('build','work','tools','.venv','__pycache__','history')) and p.name!='build.local.json')+[Path(__file__),SCRIPTS / 'prepare_damage_numbers_font.py',SCRIPTS / 'prepare_damage_numbers_sprites.py',SCRIPTS / 'render_damage_numbers_visual.py',SCRIPTS / 'test_damage_numbers_visual.py',SCRIPTS / 'damage_numbers_capture_probe.py',
        SCRIPTS / 'test_damage_numbers.py',SCRIPTS / 'damage_numbers_assets.py',SCRIPTS / 'damage_numbers_sprite_codec.py',SCRIPTS / 'test_damage_numbers_sprite_codec.py',SCRIPTS / 'verify_damage_numbers_compression.py',SCRIPTS / 'verify_damage_numbers_ge_compat.py',SCRIPTS / 'build_damage_numbers_gateway.py',SCRIPTS / 'damage_numbers_gateway_probe.py',SCRIPTS / 'audit_damage_numbers_open_assets.py',SCRIPTS / 'package_damage_numbers.py',SCRIPTS / 'damage_numbers_package_helpers.py',MOD / 'vendor' / 'modern-weapon-controls' / 'src/native_platform.cpp',
        MOD / 'vendor' / 'modern-weapon-controls' / 'src/native_platform.h',MOD / 'vendor' / 'modern-weapon-controls' / 'src/svc.S']
    manifest={'asset_profile':profile,'status':'RISE_STYLE_CANDIDATE','nro_sha256':sha(target),
        'package':str(target.parent.parent.parent.parent),'main_sha256':baseline.MAIN_SHA,'sdk_sha256':SDK_SHA,
        'forge_sha256':sha(live_forge),'forge_revision':revision,
        'hook_offsets':[hex(o) for o in OFFSETS[:8]],
        'default_hook_offsets':[hex(o) for o in OFFSETS[:8]],
        'recovery_revision':21,'base_font_pixels_720p':24.0,'rise_base_font_pixels_720p':20.0,'rise_weak_font_pixels_720p':28.0,
        'apsr_restore':'NZCVQ only; GE remains as left by helper',
        'ge_liveness_audit':'mods/damage-numbers/GE_PRESERVATION_AUDIT.md',
        'wilds_animation':{'peak_scale':1.2,'settle_seconds':0.2,'rise_fade_seconds':0.2,'rise_pixels_720p':30,'duration_is_total':True},'camera_capture_default':True,'activation_flags_required':False,
        'diagnostic_history_records':8,'history_retained_across_player_changes':True,
        'hit_log_limit_per_process':256 if os.environ.get('MHGU_DIAGNOSTICS','0')=='1' else 0,'hit_budget_per_render_consume':8,'record_budget_per_render_consume':64,
        'camera_max_age_seconds':0.20,'preview_direct_owned_sources_only':False,
        'preview_local_palico_child_tags':['0x030402','0x030802'],
        'recognized_monster_dispatch_subtype':2,'recognized_object_hunter_child_tag':'0x000802',
        'recognized_monster_and_object_sources_always_on':True, 'bounce_gray_always_on':True,'unknown_sources_displayed':False,
        'bounce_match_max_age_seconds':0.10,'bounce_requires_unique_active_attack':True,
        'bounce_requires_same_native_thread_and_counter':True,
        'settings_save_paths':['savedata:/damage_numbers.ini'],
        'settings_format':'INI without version marker; four required fields; existing version=2 accepted; bounded 255 bytes; verified primary only; legacy backups ignored',
        'appearance_default':'Wilds','appearance_styles':['Wilds','Classic','Rise'],'rise':rise_manifest,'font':font_manifest,'sprites':sprite_manifest,
        'spacing_candidates':17,'spacing_max_font_sizes':[1.6,1.0],'spacing_offsets_cached':True,
        'plugin_panel_style':'compact unframed TreeNode; closed by default; settings only',
        'settings_load_retry_seconds':1,'settings_sd_migration':False,'settings_save_requires_button':True,
        'settings_fs_error_policy':'SDK scoped thread-local abort disabler; previous context restored',
        'extra_code_memory_objects':1,'arena_bytes':4096,
        'tls_slots':1,'thread_context_capacity':16,'scope_capacity_per_thread':64,'records_per_thread':256,
        'record_bytes':248,'timestamp_api':'nnosGetSystemTick C u64; validated original SDK wrapper',
        'empty_romfs_files_allowed':False,
        'game_save_payload_writes':False,'save_container_config_writes':True,'sd_settings_writes':False,'floating_number_renderer':True,'installation_performed':False,'imports':sorted(imports),
        'container_image':baseline.IMAGE,'sources':{str(p.relative_to(ROOT)):sha(p) for p in sources if p.is_file()},
        'nro_file_bytes':len(nro_bytes),'nro_bss_bytes':nro.u32(0x38),
        'wilds_weakspot_glow':False,'wilds_outline_pixels_at_20px':0.5*20/24,'wilds_outline_pixels_at_base':0.5,'wilds_outline_rgb':'17130f','wilds_outline_opacity':0.9,'wilds_outline_subpixel_translation':True,'wilds_star_height_font_sizes':0.31612110*1.5,'wilds_crit_size_multiplier':1.5,'wilds_glyph_origin_aligned':True,
        'limits':['OFL Open Sans and independently authored procedural art only; fixtures do not establish new runtime acceptance',
                  'Gray bounce only for uniquely matching latest hunter hit; ambiguous feedback retains HZV color',
                  'Recognized objects only; unsupported environment families and recurring ticks remain hidden',
                  'Only zero-origin full-screen camera rectangles with matching display aspect are accepted']}
    (OUT/'candidate-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(f'Built and inspected {target}; automatic startup; no activation flags; installed files unchanged')

if __name__=='__main__': main()
