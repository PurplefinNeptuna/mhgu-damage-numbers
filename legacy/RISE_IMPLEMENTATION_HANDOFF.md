# Rise style implementation handoff

Prepared 2026-10-08. This handoff is documentation only: Rise is not implemented,
built, installed or runtime-tested. The next session should implement a Rise
style candidate in the existing Damage Numbers plugin and produce visual
comparisons before settling the remaining appearance choices.

Implementation follow-up: [r21-rise1 candidate](RISE_CANDIDATE.md). During
implementation on 2026-10-08, the user corrected the slash to equal upward and
downward reach from the registered hit center, with no ground cutoff. This
supersedes the short lower-tail direction below. The comparison still uses
one-half and three-quarter total viewport heights. The user then selected
three-quarter height only for Rise; r20 remains the accepted distribution.

## Start here

1. Read the applicable workspace/mod `AGENTS.md`, `README.md`, `DEVELOPMENT.md`,
   `DISTRIBUTION.json`, `dependencies.lock.json`, `docs/FORGE_PLUGIN_GUIDELINES.md`
   and `GE_PRESERVATION_AUDIT.md`. In the coordinating workspace also follow
   `docs/RESEARCH_TOOLING.md` and inspect saved Ghidra/DTI before native changes.
2. Inspect Git status and preserve unrelated edits. Read this handoff for the
   agreed behavior; native asset clip names do not override these design rules.
3. Read [Rise research](RISE_RESEARCH.md),
   [native asset inspection](research/rise-ui-inspection.json) and
   [screenshot inspection](research/rise-screenshot-inspection.json).
4. Preserve the accepted r20 runtime and releases. Work on a new candidate;
   installation, activation, emulator/save/settings changes and publication
   are outside this handoff. Do not launch either game for further extraction.

The last research commit before this handoff is `713a31e` on `main`. Prior
research checkpoints are `3ac0333`, `ad18b58`, `2ca00ca` and `2a9eee0`.
Current release is **r20**, open assets only, with accepted NRO SHA-256:

```text
318ee366acfd905451ddeec2c39d42b8587d24372f222ded3026907f8f9cc2c3
```

Its coordinating-workspace package is
`build/damage-numbers/releases/r20/damage-number/romfs/nativeNX/plugins/damage_numbers_capture.nro`.
`READY_TO_DISTRIBUTE` applies to r20 only. R20 has host/ARM/static evidence,
without a fresh gameplay run; earlier r17 gameplay reports retain their scope.
Recheck current records/hashes instead of assuming this snapshot is current.

## Agreed behavior

- Add **Rise** alongside Wilds and Classic. Preserve their accepted appearance,
  settings compatibility, defaults, damage capture, ownership and finalized
  individual-hit totals. No additional native hooks are expected.
- All styles use the existing shared `tint()` classification. Confirmed bounce
  overrides HZV and selects gray; otherwise HZV >=45 selects orange/yellow and
  lower/unknown HZV selects white. Rise borrows colors, not Rise's categories.
- Rise palette: orange/yellow `#F0B41E`, white `#FFFFFF`, bounce gray `#808080`
  from OTOMO. Companion ownership must not select OTOMO gray. Affinity affects
  the slash only, independently of digit color. Wilds/Classic keep their palettes.
- Use broad rendered digits with thick black outlines and a lightly softened
  outer edge. The old unoutlined font preview does not justify a narrow face.
  A screenshot scan found approximately 4-5 px dark edge beside 30 px tall
  orange fill; use full outlined glyph previews to calibrate, not a literal
  stroke radius copied across resolutions. Retain the previously agreed size
  relationships: native orange settles at 35 versus normal/gray 25 design units.
- Entrance grows/shrinks/settles, then holds. Exit starts with a brief downward
  kick, resembling one sampled 30 fps frame, then rises while fading. Native
  exit keys are y=+4 to -50 over 10 authored frames. Use elapsed time; runtime
  clip tick rate and exact full lifetime were not measured. Share the motion
  behavior across digit colors, with no new damage categories.
- Omit the shared blue-white hit flash in the first two screenshot samples.
  Preserve the digit entrance; MHGU already supplies its hit effects.
- Positive crit gets a tapered red glow slash with a bright pale core; negative
  gets purple/violet. Noncrit gets no slash. The slash briefly expands/brightens,
  thins/fades and expires while the number is still held. No Wilds stars in Rise.
- Crit origin is the **registered world hit position**, saved in `Event.position`
  and projected with the current camera. Apply no digit anchor adjustment,
  collision offset or digit motion to it. Do not follow the monster's later
  position. Damage numbers keep their usual independent overlap avoidance,
  cached per-hit placement and bounce/rise animation.
- Pick one tilt per hit, independent of affinity, and retain it throughout the
  slash. Proposed slope `s` in `[-1/4,+1/4]` gives approximately +/-14.04 degrees,
  satisfying the user's approximate +/-15 degree request. Endpoints can use
  `(x+s*dy, y-dy)`; normalize once if needed. Avoid per-frame trigonometry.
- Adapt Rise's world effect to our UI renderer. Start with total vertical extent
  **1/2 gameplay viewport height**, compare **3/4**; final height is undecided.
  Draw behind digits, keep the lower tail short and clip to the viewport. Keep
  slash bounds out of the digit collision solver. The existing overlay has no
  terrain intersection/depth test, so ground stops and scene occlusion remain
  outside this UI approximation.

## Evidence and unresolved choices

Local source screenshots are in coordinating-workspace
`work/damage-numbers/rise-research/hit_screenshots/`, 17 PNGs at 1919x1080,
reported at 30 fps: normal orange 38, positive orange 48, negative orange 58.
The nearby older 78 in the negative sequence is a separate number. Hashes,
ROIs, exit comparisons and tilt fits are in the tracked screenshot record.

Original Rise installation:
`/mnt/storage2/SteamLibrary/steamapps/common/MonsterHunterRise/`.
Ignored local extraction scripts/assets/font previews and crop sheets are under
`work/damage-numbers/rise-research/`. These are optional research inputs, not
runtime or independently exported build dependencies. Preserve originals.

The extracted digit font is GP-UDKakugoC80 Pro DB. Keep it and all extracted
Rise GUI/effect material out of the open-asset runtime/distribution. Use an open
font recipe or independently authored digits and procedural slash artwork.
Local Rise size-mod activation was not established, so screenshot pixel sizes
do not establish stock settings. Only orange was measured; shared behavior for
other colors is the user's direction. The supplied samples do not establish
the full hold/exit duration or a native random tilt distribution.

Still choose through previews: open font/glyph width, outline softness, final
slash height (1/2 versus 3/4), beam width/bloom/tail proportions, short effect
timing, and mapping authored motion to the existing duration control. These
choices do not reopen agreed color classification, origin or tilt range. Avoid
adding unrequested settings; keep Wilds as the existing default.

## Implementation map

| Location | Required work |
| --- | --- |
| `src/model.h` | Append Rise without renumbering Wilds/Classic; extend style validation and palette; preserve shared `tint()` |
| `src/settings.h`, `src/capture.cpp` | Round-trip `style=rise`, preserve version-2 Wilds/Classic settings, extend the existing Style selector |
| `src/presentation.h` | Rise size/motion/bounds; separate raw hit anchor and effect age/opacity/geometry from moving digits; retain overlap avoidance |
| `src/cosmetics.cpp` | Open font/outlined glyphs, separate red/purple slash draw before text, independent short alpha lifetime; balanced context-owned resources |
| Font/sprite recipes and provenance scripts | Add licensed/open assets, bounds and compression/provenance records without importing game assets |
| `tests/core.cpp`, `tests/visual.cpp`, relevant ARM probes | Meaningful behavior checks and real-ImGui visual comparisons, preserving existing styles |
| `scripts/render_damage_numbers_visual.py` | Handle tilted geometry accurately before using its output to judge the slash |

The current host compositor assumes axis-aligned, same-color quads and uses
position/UV min/max. A rotated image quad would render incorrectly there.
Extend it for actual triangle/UV interpolation, or use a verified equivalent
preview path. Pinned cimgui declares `ImDrawList_AddImageQuad` and filled
triangle/polygon APIs, but header presence does not prove native import
availability; verify actual Forge exports/imports before selecting the API.

Store deterministic per-hit tilt or derive it from the stable `HitKey`; do not
reroll every frame. Do not scale the crit effect with the digit's entrance pop
or inherit its placement/exit alpha. The raw hit projection can move with the
camera while the number is separately displaced. Native `project()` currently
rejects off-screen/behind-camera hits; retain safe bounds behavior.

Use Forge lifecycle/render/font/texture/preferences facilities and its counted
save mount. Keep the compact settings-only tree row closed by default, with
balanced TreePop; rendering continues when the panel closes. Preserve the
eight-site gateway ABI/NZCVQ/GE audit and forbid APSR_g/APSR_nzcvqg dependencies.
Do not change the tested Forge baseline to solve this presentation task.

## Candidate validation and delivery

From `mods/damage-numbers/`, use the mod-local entry point and configured inputs:

```sh
./build.sh doctor
./build.sh prepare
./build.sh build
./build.sh test --host
./build.sh test
./build.sh verify
```

No downloads without the explicit `prepare --fetch` workflow. Inspect recipes
and output/version selection before building so immutable r20 files stay intact.
Do not use release packaging with the r20 identity for changed runtime bytes.

Verify shared tint priority and affinity independence for all styles; settings
round trips; entrance/exit timing at 30/60 fps; multiple same-frame hits; separate
hit/slash origin under digit collision displacement and camera motion; stable
per-hit tilt; independent slash expiry; viewport clipping; font/context lifetime;
and unchanged Wilds/Classic presentation. Show real-renderer comparisons of
the two slash heights, both polarities, three digit colors, thick outlined
multi-digit values and crowded hits on bright/dark backgrounds.

Reproduce research measurements with an explicit screenshot copy:

```sh
python3 scripts/inspect_rise_hit_screenshots.py /path/to/hit_screenshots /tmp/rise-screenshot-inspection.json
```

For a deliverable, choose a new runtime version, update mod-local provenance/
exports/status records, use the prescribed package entry point and pass the
full coordinating-workspace `python3 scripts/mod_distribution.py check`.
Keep candidate/static evidence separate from gameplay acceptance. Fresh manual
Eden/Ryujinx checks should cover camera motion, bright backgrounds, dense bursts,
bounce/crit combinations, transitions and saved-style restart when the user
requests runtime testing/installation. The known Forge menu/overlay corruption
remains disclosed. Commit coherent validated changes locally; no automatic push.

## Suggested prompt for the new session

> Read `mods/damage-numbers/RISE_IMPLEMENTATION_HANDOFF.md` and the applicable
> AGENTS.md files. Implement the Rise damage-number style candidate using the
> agreed shared HZV/bounce colors, thick outlined digits, exit bounce and separate
> crit slash originating at the registered hit. Produce faithful renderer
> previews comparing half-height and three-quarter-height UI slashes, run the
> relevant host/ARM/static checks and commit locally. Preserve accepted r20 and
> unrelated edits; do not install, activate, launch games, publish or push.
