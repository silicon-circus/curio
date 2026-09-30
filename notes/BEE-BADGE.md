# Bee badge

Embroidered iron-on patches, die-cut to the bee's own silhouette — the gold overlocked satin-stitch
border follows the contour of the body and wings rather than enclosing it in a circle or shield.

| file | view | notes |
|---|---|---|
| `bee-badge.top.png`  | top-down, symmetric | cleaner read at small sizes; the classic scout-patch look |
| `bee-badge.side.png` | profile, head left, wings raised | matches the flying-bee orientation of the sprites |

Both 1024² RGBA, corner alpha 0, filling ~52% of their bounding box — i.e. genuinely die-cut, not a
rectangle with art on it. Both stay readable down to about 40px; the top-down holds slightly better.

## If you regenerate these

**Do not mention a shirt, a uniform, or fabric.** The first pass said "the kind sewn onto a scout
uniform" and Nano rendered the patch lying on magenta *cloth*, complete with weave texture and a
cast shadow — nothing keyed, and the bbox was the whole frame. Naming the garment invites the
garment.

The top-down patch was salvaged rather than re-rolled: its art was already right, so it was fed back
in as its own reference with the instruction to keep the patch exactly and replace only the
background with flat magenta. That is much cheaper than re-rolling good art to fix a bad backdrop,
and it holds registration — worth reaching for whenever the subject is right and only the ground is
wrong.

## Disposition

Verified by content hash, 2026-09-30 — see `README.md` in this directory.

| delivered as | went to |
|---|---|
| `bee-badge.top.png`  | curio `bee-badge.top.png` |
| `bee-badge.side.png` | curio `bee-badge.side.png` |

Both kept. The originals are still in `intake/bee-badge/`, byte-identical — routed by hand rather than
through the console, so nothing deleted the source.
