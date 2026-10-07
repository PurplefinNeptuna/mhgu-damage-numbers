from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
#!/usr/bin/env python3
"""Compare r16 to the preserved r15 binary, pixels and real-ImGui rendering."""
import hashlib
import json
from damage_numbers_assets import ROOT, paths


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    work, out=paths('open')
    backup=ROOT/'backups/damage-numbers/r15-before-compression-20261005'
    records=json.loads((backup/'manifest.json').read_text())
    for name,record in records.items():
        if not record.get('read_only'):
            assert sha(backup/name)==record['sha256'],name
    baseline=json.loads((backup/'build/damage-numbers/open-assets/candidate-manifest.json').read_text())
    current=json.loads((out/'candidate-manifest.json').read_text())
    assert baseline['nro_sha256']=='7c112c6bf497207f8bdf475687bab3b0056c0726ac4db1707db8f8adc01e07da'
    assert current['nro_file_bytes']<baseline['nro_file_bytes']
    assert current['nro_bss_bytes']==baseline['nro_bss_bytes']
    assert current['imports']==baseline['imports']
    assert current['font']==baseline['font']
    comparisons={}
    visual=ROOT/'work/damage-numbers/open-visual'
    for name in ('draw.json','preview.png','texture-1.rgba','texture-2.rgba'):
        old=backup/'work/damage-numbers/open-visual'/name
        new=visual/name
        assert old.read_bytes()==new.read_bytes(),name
        comparisons[name]={'identical':True,'sha256':sha(new),'bytes':new.stat().st_size}
    assert (backup/'work/damage-numbers/open-assets/wilds-effects.rgba').read_bytes()==(work/'wilds-effects.rgba').read_bytes()
    report={'status':'PASSED_R15_COMPRESSION_EQUIVALENCE',
            'nro_sha256':current['nro_sha256'],'reference_nro_sha256':baseline['nro_sha256'],
            'old_nro_bytes':baseline['nro_file_bytes'],'nro_bytes':current['nro_file_bytes'],
            'bss_bytes_unchanged':current['nro_bss_bytes'],'imports_unchanged':True,
            'font_manifest_unchanged':True,'rgba_sha256':sha(work/'wilds-effects.rgba'),
            'real_imgui':comparisons,
            'scope':'Exact host rendering/payload equivalence and static imports/memory; no live GPU performance claim'}
    (out/'compression-render-comparison.json').write_text(json.dumps(report,indent=2)+'\n')
    print('Passed r15 byte-exact artwork, font, both ImGui textures, draw commands and composited preview; BSS/imports unchanged')


if __name__=='__main__':main()
