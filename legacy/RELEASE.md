# Damage Numbers r22-config1

READY_TO_DISTRIBUTE, locally packaged and unpublished. The release preserves the
tested candidate NRO SHA-256
`c9e1f0740e12bb2806926ef985f575221aa2785384be6a064fd8181b47567f70`.
ZIP: `build/damage-numbers/releases/r22-config1/damage-numbers-r22-config1.zip`.

Includes Wilds, Classic and Rise, retaining the r21 Rise visual work and r20
Wilds rendering. Configs use only the primary INI with explicit saves; old
backups are unused. Host and ARM32 persistence failures, 361 compiled capture
checks, GE/import/asset audits and standalone byte parity passed.

The user reported the tested plugins working on Eden on 2026-10-08 and requested
promotion. Installed bytes match this payload. [Runtime scope](RUNTIME.md) and
[release evidence](docs/CONFIG_RELEASE_RUNTIME.json) distinguish that report
from exhaustive testing or new Ryujinx validation. Forge render1 stays LOCAL_ONLY
and is an external dependency. Distribution preparation rebuilds no runtime and
performs no installation, activation, publication or push.

The previous r20 delivery is preserved byte-for-byte under
`backups/release-promotion-20261008/`, with verified hashes.
