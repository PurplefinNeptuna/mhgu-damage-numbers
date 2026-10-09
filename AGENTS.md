# Standalone mod development

Read README.md, DEVELOPMENT.md, DISTRIBUTION.json and dependencies.lock.json. This folder owns its source, tests, recipes, notices and snapshots. Do not import sibling mod folders, installed emulator packages, personal paths or parent Git history. Configure supplied inputs in ignored build.local.json. Use build.sh; downloads require explicit prepare --fetch. Preserve inputs and immutable accepted releases. Panels contain settings only, with concise lifetime-deduplicated Forge logs outside hot hooks. Preserve background callbacks, ARM32 ABI, NZCVQ and all live GE state; unsupported APSR_g writes are forbidden. Static fixtures do not establish gameplay acceptance. Commit validated changes locally; never install, activate, publish or push without a user request.

## Explanatory code comments

Include enough inline explanation for the user to follow the implementation.
Describe substantial functions and complex blocks beside their code; the code
walkthrough supplements this explanation. Explain how hooks publish records,
records become events, and events become draw commands. Cover kind-dependent
fields, register/frame offsets, native field meanings, filtering, mailbox
publication, projection, placement and animation phases.
Explain units, bit masks, constants and formulas using their verified meanings.

Explain Rust wrappers, derives, closures and scope guards where they carry
behavior. Describe TLS borrow boundaries, nested returns, mailbox ownership,
resource lifetimes and rollback. State each unsafe boundary's caller requirements.
Use plain, specific comments and small flow examples; avoid repeating obvious
statements or aiming for a comment-count quota. Update comments with the code.

For comments-only work, preserve executable source and verify equivalence.
Keep pinned assembly, vendored dependencies and existing candidate/release bytes
unchanged. Validate source exports and the full distribution check before a
local commit.

## Forge-rs configuration exception

The user explicitly authorized stock forge-rs config::load, automatic default creation, ScopedMount and Writer for this mod. The older primary-only custom saving rules are superseded here. Keep validated per-field defaults and Save/Reset controls. Do not add duplicate-mod detection or coexistence handling; the C++ package is disabled during Rust use.

## Release ZIP layout

Ready and published ZIPs include `README.md` at the archive root beside the named
mod folder. Keep only runtime files inside the mod folder. The archived README
must match the adjacent release README. Use the distribution packager and full
check; support refreshes preserve runtime bytes and back up the previous ZIP.

Notices/licenses stay outside the ZIP. Supply combined `NOTICES.txt` and a
notices-only `NOTICES.html` alternative beside the ZIP, formatted as a paste-ready
GameBanana fragment. Preserve license wording.

Release notices cover only the package's current code, assets and applicable
credits, including Forge where used. Preserve required copyright and license
text. Exclude unrelated mods, development tools, local source paths, Git/history
references and previous-license commentary. Keep development provenance in
`THIRD_PARTY.md`; use dedicated `DISTRIBUTION_NOTICES.md` for release credits and
component notices, alongside current license files. Notices must be usable
without a source checkout.

For every current and future mod, supply `README.html` beside the ZIP as a
paste-ready GameBanana HTML fragment generated from the release README. Preserve
its installation/use content, headings, lists, controls/settings tables, public
links and code examples. Keep HTML alternatives outside the ZIP; archive only
`README.md` beside the runtime folder. Keep README content user-facing and
notices limited to current package contents and applicable credits.

For GameBanana, generate separate `GAMEBANANA_README.html` from the install
README plus the explicit `gamebanana` record in `DISTRIBUTION.json`. Follow
https://gamebanana.com/wikis/2175: prominently disclose actual AI code assistance,
name/link its software and describe its scope. Credit applicable original code
and asset authors/sources. AI-assisted code needs no training-dataset inventory;
model-generated artwork/audio still needs known software, input and dataset
credits. Never infer AI provenance or claim human review that has not occurred.
Keep this platform description outside the ZIP; retain the install guides and
notices separately. Apply this to every current and future mod and recheck the
live policy before publication.
