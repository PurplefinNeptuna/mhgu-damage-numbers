#!/usr/bin/env python3
"""Bake an independently authored red/violet tapered beam; no game assets."""
import hashlib, json, math
from PIL import Image
from paths import MOD
from damage_numbers_assets import paths
from damage_numbers_sprite_codec import encode, decode

def prepare():
    work,_=paths('open');work.mkdir(parents=True,exist_ok=True)
    recipe=json.loads((MOD/'fonts/rise-open.json').read_text())
    width,height=64,256
    data=bytearray()
    for color in ((255,35,30),(165,55,255)):
        for y in range(height):
            dy=(y+.5)/height-.5
            along=max(0,1-abs(dy)/.5)
            taper=.12+.88*along
            for x in range(width):
                across=abs((x+.5)/width-.5)*2/taper
                glow=math.exp(-across*across*4.5)*along**.65
                core=math.exp(-across*across*100)*along**.35
                alpha=round(255*min(1,glow*.72+core*.85))
                pale=(255,245,238) if color[0]==255 else (248,238,255)
                rgb=tuple(round(c*(1-core)+p*core) for c,p in zip(color,pale))
                data.extend((*rgb,alpha) if alpha else (0,0,0,0))
    data=bytes(data);packed=encode(data);assert decode(packed,len(data))==data
    (work/'rise-slash.rgba').write_bytes(data)
    Image.frombytes('RGBA',(width,height*2),data).save(work/'rise-slash.png')
    rows=','.join(map(str,packed))
    (work/'rise_sprite_data.h').write_text('#pragma once\nstatic const unsigned char rise_sprite_lz[] __attribute__((used))={'+rows+'};\nstatic constexpr unsigned rise_sprite_width=64,rise_sprite_height=512;\n')
    report={'source_kind':'Independent procedural artwork; no sampled or traced game art',
            'recipe':recipe,'width':width,'height':height*2,'decoded_bytes':len(data),
            'compressed_bytes':len(packed),'rgba_sha256':hashlib.sha256(data).hexdigest(),
            'compressed_sha256':hashlib.sha256(packed).hexdigest(),'codec':'DN-LZ1',
            'polarity_rows':['red','violet'],'origin_v':.5,'texture_context_lifetime':True}
    (work/'rise-manifest.json').write_text(json.dumps(report,indent=2)+'\n');return report
if __name__=='__main__':print(json.dumps(prepare(),indent=2))
