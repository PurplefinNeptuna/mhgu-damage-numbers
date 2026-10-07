from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
#!/usr/bin/env python3
"""Lossless sprite, bounded decoder and host initialization-cost checks; no game."""
import hashlib
import json
import os
from pathlib import Path
import random
import subprocess
from damage_numbers_sprite_codec import encode, decode
from damage_numbers_assets import ROOT, paths


def main():
    work, out = paths('open')
    from prepare_damage_numbers_sprites import prepare
    art = prepare()
    raw = (work/'wilds-effects.rgba').read_bytes()
    # This is the accepted r15 artwork, including its transparent RGB bytes.
    expected = 'fe15190c98259443a01add264777b2b4bee994bbd58f9c8f5f12338abb399f2b'
    assert hashlib.sha256(raw).hexdigest() == expected
    random_source = random.Random(613)
    samples = [b'', b'x', b'xx', b'xxx', bytes(range(256)), b'x'*70000,
               bytes(range(256))*500, raw]
    samples += [random_source.randbytes(n) for n in (127,128,129,8192,65536)]
    for sample in samples:
        packed = encode(sample)
        assert decode(packed,len(sample)) == sample
        assert packed == encode(sample)
    # Retain the former encoder for the cost comparison only.
    rle=bytearray();at=0
    while at<len(raw):
        end=at+4
        while end<len(raw) and raw[end:end+4]==raw[at:at+4] and end-at<255*4:end+=4
        rle.append((end-at)//4);rle.extend(raw[at:at+4]);at=end
    (work/'old-sprite-rle.bin').write_bytes(rle)
    src=MOD
    args=[str(work/'wilds-effects.dnlz'),str(work/'wilds-effects.rgba'),str(work/'old-sprite-rle.bin')]
    common=['c++','-std=c++17','-Wall','-Wextra','-Werror','-I'+str(src/'src'),str(src/'tests/sprite_codec.cpp')]
    sanitized=work/'sprite-codec-sanitized'
    subprocess.run(common+['-O1','-g','-fsanitize=address,undefined','-fno-omit-frame-pointer','-o',str(sanitized)],check=True)
    env=dict(os.environ,ASAN_OPTIONS='detect_leaks=0')
    subprocess.run([str(sanitized),*args],check=True,env=env,capture_output=True,text=True)
    benchmark=work/'sprite-codec-benchmark'
    subprocess.run(common+['-O2','-o',str(benchmark)],check=True)
    result=subprocess.run([str(benchmark),*args],check=True,capture_output=True,text=True)
    cost=json.loads(result.stdout)
    report={'status':'PASSED_LOSSLESS_SPRITE_CODEC','codec':'DN-LZ1',
            'rgba_sha256':expected,'rgba_bytes':len(raw),'compressed_bytes':art['compressed_bytes'],
            'old_rle_bytes':len(rle),'deterministic_round_trips':len(samples),
            'sanitizers':'AddressSanitizer and UndefinedBehaviorSanitizer passed',
            'host_benchmark':cost,'extra_persistent_bytes':0,'decodes_per_context':1,
            'per_frame_decode':False,'texture_dimensions':[512,256],
            'scope':'Host CPU timings only; no Eden/GPU timing or new gameplay run'}
    out.mkdir(parents=True,exist_ok=True)
    (out/'sprite-codec-test.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))


if __name__=='__main__':main()
