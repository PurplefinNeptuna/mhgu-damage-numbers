# Damage Numbers r23-config2

READY_TO_DISTRIBUTE, locally packaged and unpublished. The INI requires only
`enabled`, `style`, `scale` and `lifetime` under `[DamageNumbers]`. No version
marker is required or written. Existing `version=2` files still load their
preferences; the next explicit save omits the marker. Primary-only storage,
validation, explicit saves and ignored backups remain in place.

NRO SHA-256: `31646800b1bbfb5aa80d0d968b60d1c171b4aebaa1e5662b473eaa2b530312b1`.
ZIP: `build/damage-numbers/releases/r23-config2/damage-numbers-r23-config2.zip`.

Host validation passed 3,315 core/presentation checks and production storage
load/save/restart, old-file migration and failure paths. All 365 compiled capture
checks, 9,287 gateway checks, SDK/GE/import audits and real-ImGui/asset checks
passed. Font and sprite hashes, rendering and native hooks are unchanged.
[Config validation](docs/CONFIG_WITHOUT_VERSION.json) records the exact payload
and reports. The r22-config1 Eden confirmation remains in [RUNTIME.md](RUNTIME.md);
no fresh game run is claimed for this config revision. Forge stays LOCAL_ONLY.
Nothing was installed, activated or published.

The r22-config1 delivery is preserved with hashes under
`backups/config-no-version-20261008/`.
