from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
#!/usr/bin/env python3
"""Verify the production r17 executable changes against the audited r16 NRO."""
import hashlib
import json
import re
import struct
import subprocess
from damage_numbers_assets import ROOT, paths
from mhgu_binary import Nro


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    work,out=paths('open')
    reference=ROOT/'backups/damage-numbers/r16-before-r17-ge-20261006/build/damage-numbers/releases/r16/damage-number/romfs/nativeNX/plugins/damage_numbers_capture.nro'
    before=reference.read_bytes()
    assert sha(before)=='5f59506f88d5156ed8db7cffd5747651717b3bab7ecdfd398800a0402fe4f33e'
    after=(out/'candidate-package/romfs/nativeNX/plugins/damage_numbers_capture.nro').read_bytes()
    old,new=Nro(before),Nro(after)
    assert new.segments==old.segments and len(after)==len(before)
    assert new.u32(0x38)==old.u32(0x38)
    start,size=old.segments[0]
    # NRO build ID is content-dependent; compare executable bytes after its
    # 128-byte header, not that header's fingerprint.
    positions=[at for at in range(128,start+size,4) if struct.unpack_from('<I',before,at)[0]==0xe12cf002]
    assert len(positions)==9
    expected=bytearray(before)
    for at in positions:struct.pack_into('<I',expected,at,0xe128f002)
    assert after[128:start+size]==expected[128:start+size], 'Unexpected executable change outside nine restore instructions'
    names={name:int(address,16) for address,name in re.findall(
        r'^([0-9a-f]+) [tT] (dn_gateway_(?:[0-7]|after))$',(work/'symbols.txt').read_text(),re.M)}
    assert len(names)==9
    sites=[]
    for name,at in sorted(names.items()):
        length=76 if name.endswith('after') else 80
        inside=[p for p in positions if at<=p<at+length]
        assert len(inside)==1
        sites.append({'gateway':name,'offset':hex(inside[0]),'old':'e12cf002','new':'e128f002'})
    data_at,data_size=new.segments[2]
    assert after[data_at:data_at+data_size]==before[data_at:data_at+data_size]
    imports=lambda n:sorted(row[0].decode() for row in n.symbols if row[0] and not row[1][5])
    assert imports(old)==imports(new)
    reference_repo=ROOT/'references/ryujinx-nextendo'
    revision=subprocess.check_output(['git','-C',str(reference_repo),'rev-parse','HEAD'],text=True).strip()
    assert revision=='05577ab543f4322cb7534c1f5caca0e0079791bf'
    translator=reference_repo/'src/ARMeilleure/Instructions/InstEmitSystem32.cs'
    source=translator.read_text()
    assert 'if ((op.Mask & 8) != 0)' in source and 'if ((op.Mask & 4) != 0)' in source
    assert 'throw new NotImplementedException("APSR_g")' in source
    assert (0xe128f002>>16)&15==8 and not (0xe128f002&(1<<22))
    report={'status':'PASSED_NZCVQ_ONLY_GATEWAYS','nro_sha256':sha(after),
            'reference_nro_sha256':sha(before),'restore_sites':sites,
            'executable_changes':'Exactly nine e12cf002 to e128f002 words; all other executable bytes identical to audited r16',
            'data_segment_unchanged':True,'bss_bytes_unchanged':new.u32(0x38),'imports_unchanged':True,
            'translator_reference_revision':revision,'translator_source_sha256':sha(translator.read_bytes()),
            'gateway_contract':'NZCVQ preserved; GE is left as set by callbacks, based on the eight-site liveness audit',
            'scope':'Static production NRO/translator validation; not Ryujinx or Eden gameplay acceptance'}
    (out/'ge-compatibility.json').write_text(json.dumps(report,indent=2)+'\n')
    print('Verified exactly nine production restore changes, identical remaining executable/data, BSS/imports and supported mask-8 translator path')


if __name__=='__main__':main()
