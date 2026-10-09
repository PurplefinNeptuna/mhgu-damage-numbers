#!/usr/bin/env python3
"""Bake owned fonts and procedural sprites, then supply Rust constants without changing tracked source."""
import json,re,hashlib
from paths import WORK_ROOT
from prepare_damage_numbers_font import prepare as font
from prepare_damage_numbers_sprites import prepare as wilds
from prepare_damage_numbers_rise import prepare as rise

def prepare():
 f=font();w=wilds();r=rise();folder=WORK_ROOT/'open-assets'
 header=(folder/'rise_sprite_data.h').read_text()
 data=bytes(map(int,re.search(r'rise_sprite_lz\[\].*?=\{([^}]+)\}',header,re.S).group(1).split(',')))
 (folder/'rise-slash.dnlz').write_bytes(data)
 source='// Generated from pinned open assets; never edit by hand.\n'
 source+=f'const FONT_SCALE:f32={f["height_scale"]:.9f};\n'
 source+=f'const RECTS:[[u16;8];22]={json.dumps(w["records"])};\n'
 for name,file in [('WILDS','wilds-effects.dnlz'),('RISE','rise-slash.dnlz')]:source+=f'const {name}:&[u8]=include_bytes!("{folder/file}");\n'
 (folder/'assets.rs').write_text(source)
 (folder/'assets.json').write_text(json.dumps({'font':f,'wilds':w,'rise':r},indent=2)+'\n')
 print('Prepared digit TTF and both procedural atlases')
if __name__=='__main__':prepare()
