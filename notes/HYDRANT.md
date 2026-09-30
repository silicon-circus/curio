# Fire hydrant

| file | |
|---|---|
| `hydrant.red.day.png`   | bright daylight from the upper left |
| `hydrant.red.night.png` | boardwalk lamps lit — warm amber from the left, cool moonlight on the other side |

Both 768×1376 RGBA, corner alpha 0, drawn as flat side elevations with no vanishing point to match
the boardwalk panorama convention. The flange base sits level at the bottom of the figure, so the
hydrant stands on the deck rather than being sunk into it.

The pair is **pixel-registered** — identical bounding boxes at (119,173)–(649,1236) — so they swap
in place with no repositioning.

## Keyed on green, not magenta

Red against magenta is the closest subject/key pairing in this set; chroma green gives maximum
separation from red. The result has a body green cast of −55 (strongly red, no spill) and needed no
`rim_bleed` at all.

The night version relit cleanly at the first attempt with exact registration — unlike the slim
piling, where the model would not preserve the subject's proportions through a relight. The
difference seems to be that a hydrant's shape is distinctive and unambiguous, whereas "a slender
post" fights a strong prior about how thick a post should be.

## Disposition

Verified by content hash, 2026-09-30 — see `README.md` in this directory.

| delivered as | went to |
|---|---|
| `hydrant.red.day.png`   | curio, same name |
| `hydrant.red.night.png` | curio, same name |

Both kept under the delivered names. `intake/hydrant/` is empty — group finished.
