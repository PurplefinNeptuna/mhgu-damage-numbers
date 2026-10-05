# R12 brief visual check

The reference is the default Wilds UI. Existing combat and persistence coverage
stays accepted; this update changes digit shapes, shading and crit artwork.
Installation remains separate from the staged build.

- [ ] Compare a white positive crit and a gold positive crit with the reference:
  digits should have the same shape, crisp centers and soft shading; the red
  star should have a pale core, soft bloom and centered placement.
- [ ] Check one negative crit: the original blue sprite should sit below its
  number and fade with it. Gold should have a subtle soft halo, without a hard
  black stroke or repeated sharp outline.
- [ ] Fire a couple of familiar LBG bursts: confirm the accepted size/animation
  and compact spacing still feel right. Switch Classic and back once.

If anything differs, one close screenshot and whether it concerns digit shape,
shadow/halo or the star is sufficient. No full damage or save test repeat is
needed. Host preview: `work/damage-numbers/host-visual/preview.png`.

The original digit outlines and marker pixels are used, but Forge's font
rasterization and the Gaussian conversion of RE Engine's blur are different
rendering paths. The preview does not prove a one-to-one Eden match.
