# Silver plaques (blank)

Four small satin-silver plaques, the kind screwed to a wooden frame. Every one is **deliberately
blank** — no engraving, no inscription — so the text can be drawn procedurally over the central field.

| file | outline | fills its bounding box |
|---|---|---|
| `plaque-scroll.png` | nameplate with scrolled ogee ends and an ornate border | 88.5% |
| `plaque-oval.png`   | broad oval with a beaded rim                            | 88.0% |
| `plaque-notch.png`  | rectangle with cut corners and a moulded border         | 97.7% |
| `plaque-shield.png` | cartouche: bowed top and bottom, pinched waist, corner scrolls | 86.9% |

Box fill is a shape check — a plain rectangle sits near 99%.

All 1376×768 RGBA, corner alpha 0, finished satin rather than mirror so nothing mirrors the magenta
key back into the metal.

## Text field

The central field is 100% solid on all four. Safe rectangles, in the 1376×768 canvas:

| file | x | y |
|---|---|---|
| `plaque-scroll` | 266–1109 | 271–497 |
| `plaque-oval`   | 266–1109 | 265–501 |
| `plaque-notch`  | 269–1106 | 226–542 |
| `plaque-shield` | 287–1087 | 225–542 |

## If you regenerate these

`plaque-shield` needed a **massing template** — a grey cartouche silhouette on flat magenta, at
`props/plaque-shield/template.png`. Two attempts to describe the outline in words came back as a
plain rectangle both times, including one that said "definitely NOT a plain rectangle". Shape is
controlled by the template, not the prompt.

Nano also drew a cast shadow onto the backdrop on three of the four; it keys in because dark magenta
is far from the key colour. Silver separates from it cleanly by luminance, so it is stripped in post
(kill dark pixels lying strictly below the metal's lowest row in each column) rather than re-rolled.

## Disposition

Verified by content hash, 2026-09-30 — see `README.md` in this directory.

**None of these went to curio.** All four were routed to `bloom-static`, which vendors its own art
because it ships as a store build and cannot fetch by name at runtime:

| delivered as | went to |
|---|---|
| `plaque-scroll.png` | `bloom-static/www/img/plaque-scroll.png` |
| `plaque-oval.png`   | `bloom-static/www/img/plaque-oval.png` |
| `plaque-notch.png`  | `bloom-static/www/img/plaque-notch.png` |
| `plaque-shield.png` | `bloom-static/www/img/plaque-shield.png` |

All four byte-identical to the originals, which are still in `intake/plaque/`.

Curio's `wcfranks.arcade.inner-sanctum-plaque.{png,webp}` is **not** one of these — different bytes,
different plaque, unrelated to this delivery.
