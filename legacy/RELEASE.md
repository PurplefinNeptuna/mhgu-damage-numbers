# Damage Numbers r24-savehint

READY_TO_DISTRIBUTE, locally packaged and unpublished. Save feedback now says
"Saved damage_numbers.ini in MHGU's save folder". The public guide uses the
plugin's own filename and MHGU's player save folder as its location.

NRO SHA-256: `2c0e89e721ea50e0d0b4f9962d2d694c28f9d8cbec1069f8de37caadd8e53e5b`.
ZIP: `build/damage-numbers/releases/r24-savehint/damage-numbers-r24-savehint.zip`.

The only native source change is the success string. Seven other native objects
are byte-identical to r23-config2; settings instructions and relocation targets
also match. Fresh host checks cover production save/load, failure cleanup and
3,315 core/presentation cases. NRO imports, GE restoration and the asset audit
passed. The preceding 365 compiled capture checks are recorded for r23-config2;
they were not rerun for this text correction. Rendering, config parsing and
persistence operations are unchanged. [Validation](docs/SAVE_HINT_VALIDATION.json)
records this scope. No fresh game run, installation or publication occurred.

The previous delivery is archived with verified hashes under
`backups/config-save-hints-20261008/`. Forge remains LOCAL_ONLY.
