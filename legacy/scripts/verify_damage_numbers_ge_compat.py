#!/usr/bin/env python3
# Audit emitted gateway restore instructions against the pinned assembly.
# The eight-site GE liveness justification lives in GE_PRESERVATION_AUDIT.md.

"""Audit all emitted gateway restores without a historical binary or emulator repo."""
import hashlib, json, re, struct
from paths import MOD, ROOT
from damage_numbers_assets import paths
from mhgu_binary import Nro


# Require the unchanged audited gateway source and inspect all nine emitted restores.
# The after gateway adds the ninth restore to the eight installed entry sites.
def main():
    work, out = paths('open')
    raw = (out / 'candidate-package/romfs/nativeNX/plugins/damage_numbers_capture.nro').read_bytes()
    n = Nro(raw)
    baseline = json.loads((MOD / 'research/gateway-baseline.json').read_text())
    assert hashlib.sha256((MOD / 'src/gateway.S').read_bytes()).hexdigest(
    ) == baseline['source_sha256'], 'Gateway source changed; repeat liveness audit'
    names = {name: int(a, 16) for a, name in re.findall(
        r'^([0-9a-f]+) [tT] (dn_gateway_(?:[0-7]|after))$', (work / 'symbols.txt').read_text(), re.M)}
    assert len(names) == 9
    sites = []
    for name, at in sorted(names.items()):
        length = 76 if name.endswith('after') else 80
        words = [(p, struct.unpack_from('<I', raw, p)[0]) for p in range(at, at + length, 4)]
        restores = [p for p, w in words if w == 0xe128f002]
        assert len(restores) == 1 and not any(w == 0xe12cf002 for p, w in words)
        sites.append({'gateway': name, 'offset': hex(restores[0]), 'new': 'e128f002'})
    assert not re.search(r'\bmsr\b.*(?:APSR_g|APSR_nzcvqg)',
                         (work / 'disassembly.txt').read_text(), re.I)
    report = {'status': 'PASSED_NZCVQ_ONLY_GATEWAYS', 'nro_sha256': hashlib.sha256(raw).hexdigest(), 'restore_sites': sites, 'assembly_source_unchanged_from_r17': True, 'gateway_contract':
              'NZCVQ preserved; GE liveness audited for every site and continuation in GE_PRESERVATION_AUDIT.md', 'scope': 'Static emitted instructions and unchanged audited gateway assembly; no compiled gateway fixtures or gameplay run by this audit.'}
    (out / 'ge-compatibility.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS: nine NZCVQ-only gateways, unchanged r17 assembly and no unsupported GE writes')


if __name__ == '__main__':
    main()
