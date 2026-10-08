# r22-config1 candidate

Retains all r21-rise1 visuals and changes only config persistence. Uses only
`damage_numbers.ini`; legacy backups remain untouched and unused. Missing or
invalid primary configs use defaults. Explicit saves validate, flush, verify and
commit the primary; failures retain session settings without promising prior
disk contents. Accepted r20 and r21-rise1 artifacts remain preserved.

Outputs use `build/damage-numbers/candidates/r22-config1/`. Host and ARM32
fixtures cover persistence; real save restart and Forge GPU acceptance are
separate manual checks. This candidate is not installed or accepted.

## Release promotion, 2026-10-08

r22-config1 is now READY_TO_DISTRIBUTE following the user's tested-Eden
confirmation. The preceding candidate-stage notes describe preparation before
that report. The candidate NRO is preserved unchanged; the immutable release
lives under `build/damage-numbers/releases/r22-config1/`. Forge remains LOCAL_ONLY.

## Version marker removed, 2026-10-08

r23-config2 writes only actual preference fields. Existing `version=2` configs
remain readable; the next explicit save drops the marker. Primary-only storage
and backup non-access remain in place. The previous delivery is preserved under
`backups/config-no-version-20261008/`. Validation is recorded in
[CONFIG_WITHOUT_VERSION.json](docs/CONFIG_WITHOUT_VERSION.json); no fresh game run
is claimed for this revision.

## Standalone save hints, 2026-10-08

r24-savehint replaces the Better Input location reference in save feedback
with this plugin's INI filename and MHGU's save folder. Only the success
string and public instructions change; storage and config behavior are retained.
[Validation](docs/SAVE_HINT_VALIDATION.json) records the checks and runtime scope.
