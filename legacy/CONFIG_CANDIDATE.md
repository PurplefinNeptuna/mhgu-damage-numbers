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
