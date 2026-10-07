from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
#!/usr/bin/env python3
"""Bake open-font effects and independently authored procedural crit sprites."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont, ImageFilter
import hashlib, json, math
from paths import ROOT, MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
OUT=ROOT/'work/damage-numbers/open-assets'
def original_marker(color):
    # Independently authored generic four-point sparkle, not traced or sampled.
    zoom=4;center=(44.5,24);rx,ry,wx,wy=24,10.5,3.5,1.5
    cx,cy=center
    points=[(cx-rx,cy),(cx-wx,cy-wy),(cx,cy-ry),(cx+wx,cy-wy),
            (cx+rx,cy),(cx+wx,cy+wy),(cx,cy+ry),(cx-wx,cy+wy)]
    mask=Image.new('L',(88*zoom,48*zoom));ImageDraw.Draw(mask).polygon(
        [(round(x*zoom),round(y*zoom)) for x,y in points],fill=255)
    art=Image.new('RGBA',mask.size)
    # A broad colored halo and a horizontal bloom streak stay outside the
    # unchanged solid polygon. Bake them once instead of adding draw calls.
    for sx,sy,strength in ((18,5,180),(20,2,120)):
        halo=Image.frombytes('L',mask.size,bytes(
            round(strength*math.exp(-.5*(((x/zoom-cx)/sx)**2+((y/zoom-cy)/sy)**2)))
            for y in range(mask.height) for x in range(mask.width)))
        layer=Image.new('RGBA',art.size,color+(0,));layer.putalpha(halo)
        art=Image.alpha_composite(art,layer)
    for radius,strength,alpha in ((3.2,1.0,100),(1.5,1.3,140),(.55,1,175),(0,1,230)):
        a=mask.filter(ImageFilter.GaussianBlur(radius*zoom)) if radius else mask
        a=a.point(lambda value:min(255,round(value*strength*alpha/255)))
        layer=Image.new('RGBA',art.size,color+(0,));layer.putalpha(a)
        art=Image.alpha_composite(art,layer)
    highlight=mask.filter(ImageFilter.GaussianBlur(.22*zoom)).point(lambda a:round(a*.7))
    layer=Image.new('RGBA',art.size,(255,245,250,0));layer.putalpha(highlight)
    return Image.alpha_composite(art,layer).resize((88,48),Image.Resampling.LANCZOS)

def prepare(profile='open'):
    from damage_numbers_assets import paths
    global OUT
    OUT, _ = paths(profile)
    from prepare_damage_numbers_font import prepare as font_prepare
    font=font_prepare(profile)
    atlas=Image.new('RGBA',(512,256));records=[]
    face=ImageFont.truetype(str(OUT/'wilds-digits.ttf'),round(45*font['height_scale']))
    for row in range(2):
        for digit in range(10):
            mask=Image.new('L',(48,64));draw=ImageDraw.Draw(mask)
            box=face.getbbox(str(digit));w,h=box[2]-box[0],box[3]-box[1]
            x,y=24-w//2,32-h//2
            draw.text((x-box[0],y-box[1]),str(digit),font=face,fill=255)
            radius=(4.5 if row==0 else 3.0)*font['height_scale']
            # Native blur sizes are 6x6 for shadow and 4x4 for foreground glow.
            # The Gaussian conversion is a renderer approximation, not a shader port.
            blur=mask.filter(ImageFilter.GaussianBlur(radius))
            blur=blur.point(lambda a:min(255,round(a*(5 if row==0 else 2))))
            img=Image.new('RGBA',mask.size,(255,255,255,0));img.putalpha(blur)
            atlas.paste(img,(48*digit,64*row))
            records.append([48*digit,64*row,48,64,x,y,w,h])
    for i,(x0,y0) in enumerate([(0,148),(92,148)]):
        sprite=original_marker(
            (241,104,155) if i==0 else (104,185,255))
        atlas.paste(sprite,(i*96,128))
        records.append([i*96,128,88,48,0,0,88,48])
    data=atlas.tobytes()
    # Preserve the accepted RGBA pixels, including canonical transparent padding.
    pixels=[bytes((0,0,0,0)) if data[i+3]==0 else data[i:i+4] for i in range(0,len(data),4)]
    canonical=b''.join(pixels)
    from damage_numbers_sprite_codec import encode, decode
    packed=encode(canonical)
    if decode(packed,len(canonical))!=canonical:raise ValueError('Sprite codec round trip failed')
    def array(name,b):return 'static const unsigned char '+name+'[]={\n'+',\n'.join(','.join(str(v) for v in b[i:i+30]) for i in range(0,len(b),30))+'\n};\n'
    header='#pragma once\n'+array('wilds_sprite_lz',packed)
    header+='static constexpr unsigned wilds_sprite_width=512,wilds_sprite_height=256;\n'
    header+='static const unsigned short wilds_sprite_rects[22][8]={\n'+',\n'.join('{'+','.join(map(str,r))+'}' for r in records)+'\n};\n'
    # The accepted open star size is independent of the selected font's metrics.
    crit_scale=1.0
    header+=f'static constexpr float wilds_sprite_font_scale={crit_scale:.9f}f;\n'
    (OUT/'wilds_sprite_data.h').write_text(header);atlas.save(OUT/'wilds-effects.png')
    (OUT/'wilds-effects.rgba').write_bytes(canonical)
    (OUT/'wilds-effects.dnlz').write_bytes(packed)
    report=dict(asset_profile=profile,source_sha256=None,rgba_sha256=hashlib.sha256(canonical).hexdigest(),codec='DN-LZ1',compressed_sha256=hashlib.sha256(packed).hexdigest(),compressed_bytes=len(packed),decoded_bytes=len(canonical),width=512,height=256,records=records,crit_source='Independently authored procedural four-point stars and Gaussian bloom; no sampled game art',effect_limit='Gaussian bloom with a crisp font fill')
    report['crit_render_scale']=crit_scale
    report['crit_glow']='Broad colored halo and horizontal bloom, unchanged solid polygon'
    (OUT/'sprite-manifest.json').write_text(json.dumps(report,indent=2)+'\n');return report
if __name__=='__main__':
    from damage_numbers_assets import argument
    print(json.dumps(prepare(argument()),indent=2))
