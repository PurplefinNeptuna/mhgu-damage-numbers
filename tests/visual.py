#!/usr/bin/env python3
"""Compare production Rust and r25 C++ draw lists, atlas pixels and composited 1080p scenes."""
import json,math,subprocess,sys,hashlib,shutil
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'scripts'))
from paths import MOD,BUILD_ROOT,WORK_ROOT,CARGO_TARGET
from build import environment
from compositor import render
env,rust=environment();assets=WORK_ROOT/'open-assets';out=WORK_ROOT/'visual';out.mkdir(parents=True,exist_ok=True)
font=out/'app:/nativeNX/plugins/damage-numbers-rs/wilds-digits.ttf';font.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(assets/'wilds-digits.ttf',font)
subprocess.run([rust/'cargo','build','--release','--config',MOD/'.cargo/vendor.toml','--manifest-path',MOD/'tests/render-harness/Cargo.toml','--target','x86_64-unknown-linux-gnu'],cwd=MOD.parent,env=env,check=True)
cimgui=MOD/'vendor/cimgui';stub=out/'forge/log.h';stub.parent.mkdir(exist_ok=True);stub.write_text('extern "C" void forge_log_info(const char*,...);\n#define forge_log_warn forge_log_info\n')
sources=[MOD/'tests/visual.cpp',MOD/'tests/oracle/cosmetics.cpp',cimgui/'cimgui.cpp',*[cimgui/'imgui'/name for name in ['imgui.cpp','imgui_draw.cpp','imgui_tables.cpp','imgui_widgets.cpp','imgui_demo.cpp']]]
library=CARGO_TARGET/'x86_64-unknown-linux-gnu/release/libdamage_numbers_render_harness.a'
subprocess.run(['c++','-std=c++17','-O2','-DDAMAGE_NUMBERS_HOST_VISUAL',*['-I'+str(p)for p in [cimgui,out,assets,MOD/'tests/oracle']],*map(str,sources),str(library),'-ldl','-lpthread','-lm','-o',str(out/'visual')],check=True)
subprocess.run([out/'visual',MOD/'vendor/fonts/Roboto-Medium.ttf',out],cwd=out,check=True)
scenes=[]
for style in range(3):
 for age in [0.05,0.2,0.75]:
  left=out/f'cpp-{style}-{age:.6f}';right=out/f'rust-{style}-{age:.6f}';a=json.loads((left/'draw.json').read_text());b=json.loads((right/'draw.json').read_text())
  assert a['textures']==b['textures'] and a['indices']==b['indices'],(style,age,'texture/index mismatch')
  assert len(a['vertices'])==len(b['vertices']) and len(a['commands'])==len(b['commands']),(style,age,'geometry size mismatch')
  for key in ['vertices','commands']:
   for i,(x,y)in enumerate(zip(a[key],b[key])):
    assert len(x)==len(y)
    for j,(v,w)in enumerate(zip(x,y)):
     exact=(key=='vertices'and j==4)or(key=='commands'and j<4)
     assert (v==w if exact else math.isclose(v,w,abs_tol=0.0003,rel_tol=2e-6)),(style,age,key,i,j,v,w)
  for ident,_,_ in a['textures']:
   assert (left/f'texture-{ident}.rgba').read_bytes()==(right/f'texture-{ident}.rgba').read_bytes(),(style,age,'atlas pixels')
  lp,rp=render(left),render(right)
  from PIL import Image
  import numpy as np
  pixels_a=np.asarray(Image.open(lp)).astype(int);pixels_b=np.asarray(Image.open(rp)).astype(int);difference=np.abs(pixels_a-pixels_b)
  assert difference.max()<=1,(style,age,int(difference.max()))
  scenes.append({'style':['Wilds','Classic','Rise'][style],'age':age,'maximum_channel_difference':int(difference.max()),'rust_preview_sha256':hashlib.sha256(rp.read_bytes()).hexdigest(),'vertices':len(b['vertices'])})
report={'status':'PASSED','renderer':'Production Rust renderer and r25 oracle with pinned real ImGui 1.92.8','resolution':[1920,1080],'scenes':scenes,'runtime_test':'NOT RUN'}
(BUILD_ROOT/'visual.json').write_text(json.dumps(report,indent=2)+'\n');print('PASS: nine real-renderer 1080p scenes match r25')
