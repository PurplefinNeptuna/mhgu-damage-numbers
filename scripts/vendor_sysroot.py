#!/usr/bin/env python3
"""Pin registry metadata needed by this nightly's build-std resolver, even for unbuilt target libraries.

Only prepare --fetch calls this downloader. Archives are verified against the toolchain's Cargo.lock.
"""
import hashlib,io,json,tarfile,tomllib,urllib.request
from pathlib import Path
from paths import MOD

def fetch(rust):
 lock=rust.parent/'lib/rustlib/src/rust/library/Cargo.lock'
 (MOD/'research/sysroot-Cargo.lock').write_bytes(lock.read_bytes())
 for package in tomllib.loads(lock.read_text())['package']:
  if 'source'not in package:continue
  name,version=package['name'],package['version'];directory=f'{name}-{version}';dest=MOD/'vendor/crates'/directory
  if (dest/'.cargo-checksum.json').exists():continue
  url=f'https://static.crates.io/crates/{name}/{directory}.crate'
  with urllib.request.urlopen(url,timeout=60)as response:data=response.read()
  if hashlib.sha256(data).hexdigest()!=package['checksum']:raise ValueError('Crate checksum differs: '+directory)
  with tarfile.open(fileobj=io.BytesIO(data),mode='r:gz')as archive:
   for entry in archive.getmembers():
    path=Path(entry.name)
    if not entry.isfile()or Path(entry.name).suffix=='.o':continue
    if path.parts[0]!=directory or '..'in path.parts or path.is_absolute():raise ValueError('Unsafe crate path')
    target=dest/Path(*path.parts[1:]);target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(archive.extractfile(entry).read())
  files={str(f.relative_to(dest)):hashlib.sha256(f.read_bytes()).hexdigest()for f in dest.rglob('*')if f.is_file()}
  (dest/'.cargo-checksum.json').write_text(json.dumps({'files':files,'package':package['checksum']}))
  print('Vendored sysroot resolver input:',directory,flush=True)
