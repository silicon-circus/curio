# Advertising blimp

Non-rigid airships with a blank rectangular panel on the flank for an ad. Both flight directions
supplied, since one circling the park needs to fly each way.

All four are 1376×768 RGBA, corner alpha 0, delivered **uncropped and unresized** so the panel
coordinates below are exact.

| file | ad panel (x0,y0)–(x1,y1) | panel size |
|---|---|---|
| `blimp.silver.nose-left.png`      | (456,245)–(987,359)  | 532×115 |
| `blimp.silver.nose-right.png`     | (388,245)–(919,359)  | 532×115 |
| `blimp.white-blue.nose-left.png`  | (338,261)–(1113,460) | 776×200 |
| `blimp.white-blue.nose-right.png` | (262,261)–(1037,460) | 776×200 |

The white-and-blue one carries a much bigger panel — 776×200 against 532×115, nearly three times the
area — so it takes a more detailed ad. The silver one is subtler and more period.

## Why flat side elevation

Drawn square-on with no perspective on purpose: the panel stays an **axis-aligned rectangle**, so an
ad composites with a plain resize and paste. Any perspective view would make it a trapezoid and cost
you a homography per ad. The trade is realism — a blimp overhead would really be seen from slightly
below. If you want that, it needs a separate render and the panel corners would have to be tracked
as four points rather than a box.

## Compositing

The panel is deliberately flat and evenly lit, so a straight paste sits correctly. If you want it to
sit *into* the envelope rather than on top of it, multiply your ad against the panel's existing
luminance — there is a faint gradient across it that sells the curvature.

The panel rectangles were found by flatness, not colour: the ad area is the only large region inside
the envelope with no curvature gradient. That detector is in the session notes if more blimps get
made — it beats colour thresholding, which fails on the white envelope where panel and skin are the
same colour.

## Blank means blank

Both panels came back genuinely empty first try. Anything sign-shaped normally attracts invented
signage, so the prompt names it four ways — no writing, no letters, no logo, no marking — and that
held.

## Disposition

Verified by content hash, 2026-09-30 — see `README.md` in this directory.

| delivered as | went to |
|---|---|
| `blimp.silver.nose-left.png`      | curio, same name |
| `blimp.silver.nose-right.png`     | curio, same name |
| `blimp.white-blue.nose-left.png`  | curio, same name |
| `blimp.white-blue.nose-right.png` | curio, same name |

All four kept under the delivered names.

A **second, undocumented set** also arrived and is still in `intake/blimp/` — the same four views with
a `.night.` tag (`blimp.silver.night.nose-left.png` and so on). Those bytes exist nowhere else, so
they were not kept. This note does not describe them; if they are ever wanted, the panel rectangles
above were measured on the day set and would need re-measuring.

**Why the night set was dropped:**
