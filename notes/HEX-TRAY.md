# Hexagonal wooden tray

A shallow hexagonal tray seen straight down — a raised mitred lip framing a plain wooden floor, so
it reads as a shallow container rather than a flat panel. Both orientations provided.

| file | orientation | tray aspect |
|---|---|---|
| `hex-tray.flat-top.png`   | flat edge top and bottom, points left and right | 1004×878 |
| `hex-tray.pointy-top.png` | point top and bottom, flat edges left and right | 880×1004 |

Both on a square 1024×1024 RGBA canvas, corner alpha 0.

## Geometry

Both silhouettes fill **75.0%** of their bounding box — the exact value for a regular hexagon, so
neither is skewed or in perspective. Mirror symmetry measures 0.2–0.4% mismatch left/right and
top/bottom.

## Usable floor

Measured by radial scan for the inner edge of the lip:

| | lip | floor inscribed circle | safe centred square |
|---|---|---|---|
| flat-top   | 39px | ⌀798px | 564×564px |
| pointy-top | 34px | ⌀811px | 573×573px |

The safe square is the largest axis-aligned square guaranteed to sit entirely on the flat floor,
centred at (512, 512). Anything larger starts to climb the lip.

## Resolution ceiling

1024×1024 is the **largest square** Nano Banana produces — the fixed dimension set has no bigger
square option. If this needs to fill a large display at native resolution, that is the ceiling for a
single generation; the alternatives are upscaling (no new detail) or generating in wide format and
cropping, which loses the square framing.

## Note

The floor is deliberately bare — no carving, inlay or grain feature that would read as content — so
things can be composited onto it.

## Disposition

Verified by content hash, 2026-09-30 — see `README.md` in this directory.

| delivered as | went to |
|---|---|
| `hex-tray.flat-top.png`   | curio, same name |
| `hex-tray.pointy-top.png` | curio, same name |

Both kept under the delivered names. `intake/hex-tray/` is empty, which is what a finished group looks
like.
