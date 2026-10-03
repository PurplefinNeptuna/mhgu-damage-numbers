# Damage-number binding research

All addresses below are offsets relative to `main`, after subtracting the
Ghidra image base `0x009c9000`. The original NSO is authoritative; the imported
database has one altered text byte and conflicting types. Selected exports
below report zero instruction-byte differences from the original.

Original MHGU 1.4.0 `main` SHA-256:
`b3935415499549adaf4c0e89822831daa175cfd456a7990791c79be3997645ef`.
Uncompressed memory-image SHA-256:
`24c208c395c65727b21b5fa91af8381396e70cf421b0fcfee0345d8743298110`.
Preserved source identity and segment hashes are in
`work/damage-numbers/inputs/provenance.json`.

## Damage and hit metadata

| Native offset | Evidence and use |
| --- | --- |
| `0x168960` | Attack initialization; player arrays use a `0xb8` attack stride. The imported 182-byte layout is unsuitable. |
| `0x16915c` | Stores affinity and marks it enabled; this is not the critical outcome. |
| `0x16c0c0` | Monster collision traversal supplies the actual collision vector and hurtbox record. |
| `0x170968` | Routes an impact to source-specific monster damage processing. Its imported body combines shared branches; verify ABI from instructions. |
| `0x9d6f8` | Applies defense/sleep/rage normalization and returns the signed contribution added to the per-part accumulator. Original-byte fixture checks passed for ordinary parts. |
| `0x97ddc` | Consumes per-part contributions. The sum is subtracted from current HP (`monster parameter +0x370`) before clamps and phase-specific HP floors. |
| `0x17885c` | Resolves actual affinity outcomes, including Weakness Exploit, Critical Boost and Bitter Affinity. Attack byte `+0x54` is 0/1/2 for none/negative/positive native outcomes. |
| `0xbab60` | Returns a monster's physical/elemental hitzone record; ordinary indices are below 8. |
| `0x16dc6c` | Selects weighted cut/blunt damage and applies monster-state HZV adjustments. Uses VFP float arguments/results. |
| `0x1748a8`, `0x178660` | Melee sharpness/bounce-effect and shot/fixed-damage paths. Effect codes alone do not prove a hunter bounce. |

Attack fields checked in native accesses include primary owner pointer `+0xc`
and kind `+0x10`, secondary owner kind `+0x6e` and pointer `+0x70`, impact vector
`+0x20..+0x28`, damage type `+0x3a`, effect code `+6`, critical outcome `+0x54`
and hurtbox pointer `+0xac`. Hurtbox byte `+6` selects the HZV record; byte `+8`
selects the accumulation part. Damage type `0x1d` takes a healing branch and
must be rejected explicitly, in addition to rejecting nonpositive contributions.

`0x9d6f8` adds its signed return to `monster parameter +0x488+2*part`.
The consumer reads each eligible part into the HP sum before applying distinct
part-break modifiers. The HP-subtraction store is at `0x98b04`; nearby clamps
would make an HP-difference-only display lose lethal-hit values and merge hits.
The callback at enemy vtable `+0x2f8` is an on-hit notification whose base
implementation is a no-op. `0x9dd40` tracks attacker threat; neither is an HP
application hook.

Ordinary collision paths combine accompanying elemental damage with physical
damage before native integer conversion and normalization. Capture must use
that completed contribution and preserve the call's impact identity. The native
normalizer's extended-part path calls vtable `+0x310` and returns zero; it needs
separate investigation before claiming coverage of every monster part.

## Ownership

The native local getter `0x276278` obtains the session index through the
singleton cell `main+0x188ca48` and resolves the player from the table at
`main+0x1885b58`, offset `+0x1c+4*index`. Valid ordinary player indices are 0–3.
`ownership.h` agrees with original native execution across all four local slots.

The palico master getter `0x285c6c`, for its palico branch, reads the current
work pointer at `object+0x3ac4`, then its master byte at `work+1`, and calls
`0x274ce4` to look up the player. The native constructor supplies inline work;
the saved DTI reports `obw->mMasterId` at `+0x3ad1`. Use the current pointer,
which can differ from inline storage. Byte `work+0x34d` is a separate serialized
field and is not the master ID.

The ownership fixture redirects work away from inline storage and compares
local/remote palico classification with the native master getter. Invalid
indices, missing work and absent master players are hidden. Constructed attack
snapshots cover direct and secondary-owned projectiles; live source attribution
still needs solo/Prowler and multiplayer trials.

## Camera

Native look-at `0x7c5e4c`, perspective `0x7c617c` and matrix product `0x29d48`
use row-vector matrices: `world * view * projection`. The perspective matrix
uses negative camera-space Z and a zero-to-one depth interval. `camera.h`
transposes the native view/projection product for the presentation helper.
The same C++ helper agrees with copied native matrices across rotated and
translated cameras, FOVs and aspect ratios. The fixture's tangent import is an
explicit float32 stub; it does not select a live camera.

The viewport render path `0xb8909c` reads a camera pointer at `viewport+0x24`,
rectangle edges at `+0x38..+0x44`, view/projection matrices at `+0xb0/+0xf0`
and alternate matrices at `+0x130/+0x170`. It can interpolate matrices and
adjust projection for viewport/render scale. The eventual adapter must capture
the matrices actually sent to rendering, identify the gameplay viewport and
map its rectangle to Forge's display coordinates. Reading only camera FOV or
an unblended matrix is insufficient.

## Open gates

- Actual collision-to-normalization correlation, distinct hit identity,
  lifecycle cleanup and extended monster-part behavior.
- HZV for the resolved physical type and current part state, including mixed
  damage-type ties and attacks without a physical HZV.
- Actual hunter bounce versus visual impact-effect codes, Mind's Eye and
  non-bouncing attacks.
- Placed/environmental bombs versus owned support bomb moves; monster attacks,
  recurring ticks and all-owner optional-source exceptions.
- Critical marker behavior for special affinity outcomes, particularly the
  positive native outcome produced by Bitter Affinity.
- Active gameplay camera, blended projection, viewport and render-scale mapping.
- ABI-preserving hooks, private arena transaction/rollback, thread ownership,
  imports, ARM32 build and runtime capture trial.

No hook site is approved for runtime use yet. The Phase 1 runtime gate remains
open, and the host core is a foundation for subsequent integration.

## Reproduce research

```bash
python3 scripts/damage_numbers_research.py inspect
python3 scripts/damage_numbers_research.py export 0x170968 0x9d6f8 0x97ddc
python3 scripts/damage_numbers_research.py scan 0x3ac4 0x3ad1
```

Exports preserve original bytes beside imported instruction bytes and caller
references, and decompile selected functions without reanalysis. They serialize
headless access to the preserved project. Generated inputs, database exports
and logs stay outside Git; no runtime dumper is rerun.
