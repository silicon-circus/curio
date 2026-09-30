# Honey Basket

The reward container for the bee game — earned badges get pinned to the weave, rewards go in the bowl.

Ten variants: two styles × bare/flowered × three camera slants. All 1024², fitted into a 900×880 box,
keyed on magenta, corner alpha 0 on every file.

| file | style | weave | slant |
|---|---|---|---|
| `honeybasket-neon-bare-30.png`        | neon | bare | ~30° |
| `honeybasket-neon-bare-15.png`        | neon | bare | ~15° |
| `honeybasket-neon-bare-08.png`        | neon | bare | ~8°  |
| `honeybasket-neon-flowered-30.png`    | neon | flowered | ~30° |
| `honeybasket-neon-flowered-30-alt.png`| neon | flowered | ~30° † |
| `honeybasket-real-bare-30.png`        | realistic | bare | ~30° |
| `honeybasket-real-bare-15.png`        | realistic | bare | ~15° |
| `honeybasket-real-bare-08.png`        | realistic | bare | ~8°  |
| `honeybasket-real-flowered-30.png`    | realistic | flowered | ~30° |
| `honeybasket-real-flowered-steep.png` | realistic | flowered | steep |

**† `neon-flowered-30-alt` carries a cast shadow** at the base — Nano drew one onto the backdrop and
it survived the key, because dark-magenta-on-magenta is far enough from the key colour to read as
subject. It cannot be stripped in post: the shadow and the basket's own dark base weave sit at the
same luminance, so any threshold that removes one removes the other. Use it on a dark ground, or
prefer `neon-flowered-30`.

## Badge area

The front weave is **100% solid** on every variant — no gaps for a pin to fall through. The usable
rectangle, in canvas coordinates, is approximately:

    x 234–789,  y 582–864

Narrower variants (`neon-bare-30`, `neon-bare-08`) start nearer x 260. Measure per file if you need
it tight; the y range is identical across all ten.

## Design notes

The interior is deliberately **empty**. A "flower basket" prompt fills itself with flowers, which
would leave nowhere for earned rewards — so on the flowered variants the blooms are tucked into the
*outside* of the weave only, never crossing the opening. On the bare variants there is nothing at all,
which is what the badges want.

The handle lies **flat in the picture plane**, left rim to right rim, so it never crosses the badge
area. Measured symmetry is 0.2–1.9% left/right mirror mismatch.

## If you regenerate these

Two things bit us and are worth avoiding:

1. **Do not explain *why* in the prompt.** The spec once read "...no decoration, because badges get
   pinned to the weave later" — and Nano drew a basket already covered in badges. Explanation inside
   a prompt is indistinguishable from instruction.
2. **Do not stack negations.** A version with ~90 extra words of "no this, not that" came back with a
   *vignetted* magenta backdrop and a white glow around the subject, so nothing keyed and the bbox
   was the whole frame. State each requirement once. Photographic style words ("straight on, level
   with the rim") are especially prone to pulling the render into a lit studio scene.

## Disposition

Verified by content hash, 2026-09-30 — see `README.md` in this directory.

**None of these went to curio.** Four were routed to `bloom-static`, which vendors its own art because
it ships as a store build and cannot fetch by name at runtime. The rest were not kept.

The delivered files were also renamed in place inside `intake/honey-basket/` to the dotted convention,
so the names in the table above are one rename behind what is on disk:

| in this note | on disk now | went to |
|---|---|---|
| `honeybasket-neon-bare-08.png` | `honey-basket.neon.bare.8deg.png` | `bloom-static/www/img/` |
| `honeybasket-real-bare-08.png` | `honey-basket.real.bare.8deg.png` | `bloom-static/www/img/` |
| — *(not in this note)* | `honey-basket.neon.titled.8deg.png` | `bloom-static/www/img/` |
| — *(not in this note)* | `honey-basket.real.titled.8deg.png` | `bloom-static/www/img/` |
| `honeybasket-neon-bare-15.png` | `honey-basket.neon.bare.15deg.png` | **not kept** |
| `honeybasket-neon-bare-30.png` | `honey-basket.neon.bare.30deg.png` | **not kept** |
| `honeybasket-neon-flowered-30.png` | `honey-basket.neon.flowered.30deg.png` | **not kept** |
| `honeybasket-neon-flowered-30-alt.png` | `honey-basket.neon.flowered-30deg-alt.png` | **not kept** |
| `honeybasket-real-bare-15.png` | `honey-basket.real.bare.15deg.png` | **not kept** |
| `honeybasket-real-bare-30.png` | `honey-basket.real.bare.30deg.png` | **not kept** |
| `honeybasket-real-flowered-30.png` | `honey-basket.real.flowered.30deg.png` | **not kept** |
| `honeybasket-real-flowered-steep.png` | `honey-basket.real.flowered.steep.png` | **not kept** |

Every original is still in `intake/honey-basket/`, so nothing is lost — the eight unkept ones exist
there and nowhere else. Note that only the **8°** slants survived, and that two `titled.8deg` variants
arrived which this note never documented. The cast-shadow warning above applies to
`neon-flowered-30-alt`, which was not kept anyway.

**Why only the 8° slants:**
