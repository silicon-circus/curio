# Radio mast

Old-school galvanised steel lattice broadcast masts, drawn as flat side elevations with no
vanishing point, matching the boardwalk panorama convention.

| file | h/w | lattice | notes |
|---|---|---|---|
| `radio-mast.tapering.day.png` | 4.53 | 68% see-through | full mast, whip aerial and red beacon, top clear of the frame |
| `radio-mast.straight.day.png` | 1.90 | 52% see-through | close view of a mast section with a dipole array; **top and sides run off the frame** |

Both 768×1376 RGBA, base flush to the bottom edge so they can be sunk to any depth.

The camera is level with the mast on both — no looking up, no foreshortening, nothing seen from
above.

## Worm's-eye pair

Two dramatic low-angle shots, deliberately breaking the flat-elevation rule — these are hero images,
not panorama props.

| file | notes |
|---|---|
| `radio-mast.wormseye.corner-on.png` | transparent, 59% opaque — **matches the elevation's 45° orientation**: one leg rushes up the centre, faces falling away either side. Its top carries modern panel antenna arrays rather than the plain beacon-and-whip |
| `radio-mast.wormseye.face-on.png`   | transparent, 45% opaque — square to one face. Cleaner, more old-school top, but a different orientation from the elevation |
| `radio-mast.wormseye.sky.png`   | finished scene with blue sky, cumulus and sun flare — shot from *directly beneath*, a more extreme view |

**Keying gotcha worth remembering:** the keyed one first came back apparently un-keyed at 99.7%
opaque. The render was fine — flat magenta — but the mast's base fills the frame's *bottom corners*,
and `auto_background: true` sampled one of those occupied corners and keyed black instead of magenta.
Setting `auto_background: false` with an explicit `background: [255,0,255]` fixed it immediately.
Any composition whose subject reaches a corner will hit this.

## Orientation

The flat elevation is **corner-on (45°)** — one leg up the centre with X-bracing either side; a
horizontal scanline finds 5 members. A face-on view of the same tower finds 10, because you see
through to the far face. That count is a quick way to tell them apart if you generate more.

## The straight one is a different kind of asset

Two attempts to bring its top inside the frame both failed; it kept filling the picture. Rather than
keep re-rolling, it is delivered as what it actually is — a close-up mast *section*, useful as a
foreground element where the top being off-screen reads as "taller than the frame". If you need a
complete straight-sided mast with its tip visible, that needs another approach.

## Keying

A lattice is the hard case: magenta shows through every triangle, so the cut is mostly edge. It came
out clean — 68% and 52% of each bounding box is transparent, with only 26 and 32 pixels of key crumbs
to drop. `despeckle` stays at 0 for these; the real detail is small and scattered, and anything that
removes crumbs aggressively will eat bracing.

## Disposition

Verified by content hash, 2026-09-30 — see `README.md` in this directory.

| delivered as | went to |
|---|---|
| `radio-mast.wormseye.corner-on.png` | curio `radio-mast.corner-on.day.wormseye.png` |
| `radio-mast.wormseye.face-on.png`   | curio `radio-mast.face-on.day.wormseye.png` |
| `radio-mast.wormseye.sky.png`       | **not kept** — still in `intake/radio-mast/`, nowhere else |
| `radio-mast.tapering.day.png`       | curio, renamed — see below |
| `radio-mast.straight.day.png`       | curio, renamed — see below |

Curio holds exactly two non-worm's-eye masts, `radio-mast.corner-on.day.png` and
`radio-mast.face-on.day.png`, which must be these two. **Which is which cannot be recovered from the
bytes** — the originals left `intake/` when they were filed, so there is nothing left to hash against,
and the delivered names describe the mast's shape (tapering vs straight) while the filed names describe
the camera (corner-on vs face-on). Those are different axes, so it is not even certain the pairing is
one-to-one.

A fifth delivery, `radio-mast.wormseye.corner-on-panels.png`, is in `intake/` and appears in no note —
not kept.

**Pairing (tapering/straight → corner-on/face-on):**

**Why sky and corner-on-panels were dropped:**
