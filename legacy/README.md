# Damage Numbers

Displays individual damage to monsters with Wilds, Rise and Classic styles,
source filtering, physical-HZV colors and critical markers.

Current local release: **r25**, READY_TO_DISTRIBUTE and unpublished.
[Installation and settings](DISTRIBUTION_README.md); [status and payload hashes](DISTRIBUTION.json).

## Development and evidence

Use [DEVELOPMENT.md](DEVELOPMENT.md) for pinned inputs, build/test/package/export
commands. [Runtime evidence](RUNTIME.md) retains the version and limits of user
reports. [THIRD_PARTY.md](THIRD_PARTY.md) records development provenance;
[DISTRIBUTION_NOTICES.md](DISTRIBUTION_NOTICES.md) supplies release credits.


User-approved base sizes: Wilds/Classic 24 px and Rise 20 px at 720p; Rise
weak points retain a 1.4× boost. R25 has build/import/asset/gateway/package
checks. Behavior suites and gameplay were skipped at the user's request.
Exact evidence: [R25_SIZE_CHANGE.json](docs/R25_SIZE_CHANGE.json).

Technical references: [native bindings and presentation](RESEARCH.md),
[GE preservation audit](GE_PRESERVATION_AUDIT.md),
[artwork compression](COMPRESSION.md), [Rise research](RISE_RESEARCH.md) and
[font/texture registration](fonts/README.md).
