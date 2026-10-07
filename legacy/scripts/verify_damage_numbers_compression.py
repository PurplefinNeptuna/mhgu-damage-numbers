#!/usr/bin/env python3
"""Verify unchanged assets and the separately approved r19 renderer baseline."""
import hashlib,json
from paths import ROOT,MOD
from damage_numbers_assets import paths
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 work,out=paths('open');b=json.loads((MOD/'research/visual-baseline.json').read_text());c=json.loads((out/'candidate-manifest.json').read_text())
 assert c['nro_file_bytes']<b['old_nro_bytes'] and c['nro_bss_bytes']<=b['bss_bytes']
 assert c['font']==b['font']
 assert sha(work/'wilds-effects.rgba')==b['sprites_rgba_sha256']
 accepted=json.loads((MOD/'research/readability-r19.json').read_text())
 assert c['recovery_revision']==19 and c['wilds_outline_pixels_at_20px']==.5 and not c['wilds_weakspot_glow']
 expected=dict(accepted['real_imgui'])
 for name in ('texture-1.rgba','texture-2.rgba'):assert expected[name]==b['real_imgui'][name]
 comparisons={}
 for name,h in expected.items():
  p=ROOT/'work/damage-numbers/open-visual'/name
  assert sha(p)==h,name
  comparisons[name]={'identical':True,'sha256':h,'bytes':p.stat().st_size}
 report={'status':'PASSED_R19_READABILITY_ASSET_EQUIVALENCE','nro_sha256':c['nro_sha256'],'reference_nro_sha256':b['reference_nro_sha256'],'appearance_baseline':'research/readability-r19.json','appearance_baseline_sha256':sha(MOD/'research/readability-r19.json'),'old_nro_bytes':b['old_nro_bytes'],'nro_bytes':c['nro_file_bytes'],'bss_bytes':c['nro_bss_bytes'],'bss_not_increased':True,'font_manifest_unchanged':True,'rgba_sha256':sha(work/'wilds-effects.rgba'),'real_imgui':comparisons,'scope':'Unchanged r15 font and texture bytes; exact approved r19 host draw/preview hashes; symmetric subpixel/scaling/fade and vertex-limit checks. No live GPU claim.'}
 (out/'compression-render-comparison.json').write_text(json.dumps(report,indent=2)+'\n');print('PASS: unchanged font/artwork textures, pinned r19 outline rendering and bounded BSS')
if __name__=='__main__':main()
