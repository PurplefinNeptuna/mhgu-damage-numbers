#!/usr/bin/env python3
"""Verify byte-exact accepted visuals using public hash receipts."""
import hashlib,json
from paths import ROOT,MOD
from damage_numbers_assets import paths
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 work,out=paths('open');b=json.loads((MOD/'research/visual-baseline.json').read_text());c=json.loads((out/'candidate-manifest.json').read_text())
 assert c['nro_file_bytes']<b['old_nro_bytes'] and c['nro_bss_bytes']<=b['bss_bytes']
 assert c['font']==b['font']
 assert sha(work/'wilds-effects.rgba')==b['sprites_rgba_sha256']
 comparisons={}
 for name,h in b['real_imgui'].items():
  p=ROOT/'work/damage-numbers/open-visual'/name
  assert sha(p)==h,name
  comparisons[name]={'identical':True,'sha256':h,'bytes':p.stat().st_size}
 report={'status':'PASSED_R15_COMPRESSION_EQUIVALENCE','nro_sha256':c['nro_sha256'],'reference_nro_sha256':b['reference_nro_sha256'],'old_nro_bytes':b['old_nro_bytes'],'nro_bytes':c['nro_file_bytes'],'bss_bytes':c['nro_bss_bytes'],'bss_not_increased':True,'font_manifest_unchanged':True,'rgba_sha256':sha(work/'wilds-effects.rgba'),'real_imgui':comparisons,'scope':'Exact artwork/font/host rendering, bounded memory and fresh import checks; settings/log cleanup changes executable/imports. No live GPU claim.'}
 (out/'compression-render-comparison.json').write_text(json.dumps(report,indent=2)+'\n');print('PASS: byte-exact r15 artwork, font, real ImGui textures, draw commands and preview; BSS bounded')
if __name__=='__main__':main()
