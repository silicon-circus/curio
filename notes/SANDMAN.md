# Sandcastle snowman

A snowman built from packed wet sand — drip-castle texture, shells and pebbles pressed in — wearing
beach gear rather than winter gear.

| file | notes |
|---|---|
| `sandman.bucket.day.png`    | **the keeper** — dive mask and snorkel, bucket hanging from one driftwood arm, a round "woah" hole pressed into the sand for a mouth, bare head, starfish and crab at its feet |
| `sandman.shellhat.day.png`  | earlier attempt: scallop-shell hat, carrot nose, and a twine collar that reads as a scarf |
| `sandman.lei.day.png`       | earlier attempt: flower lei round its middle, but it lost the hat and sunglasses |

All 1024² RGBA, corner alpha 0, camera level with the figure.

## What worked and what didn't

Asking for a list of gear (hat + sunglasses + driftwood + flip-flops) returned **some items and
silently dropped the rest** — and forbidding the twine collar cost the hat as well. Items compete
for the same slots.

The bucket version succeeded because its items occupy structurally distinct places: a mask on the
face, a bucket on the arm, shells down the front.

Every adjustment after the first render was a **reference-guided edit** — feeding the previous
render back in with "keep the figure exactly as it is, change ONE thing only", naming both what is
there now and what replaces it. Each landed in a single roll with nothing else disturbed. That is
the reliable way to adjust a figure that is already nearly right; re-describing the whole figure
instead invites the model to re-roll everything and silently drop half the item list.

Caveat learned the hard way: a batch of three numbered changes only delivered two, and the miss
(shells replaced the buttons but not the mouth) was invisible at thumbnail size. **Check edits at
working magnification**, not in a contact sheet.

The mouth went stones -> white shells -> a plain hole in the sand. The final version is sand only.

## Disposition

Verified by content hash, 2026-09-30 — see `README.md` in this directory.

| delivered as | went to |
|---|---|
| `sandman.bucket.day.png` | curio, same name |
| `sandman.lei.day.png`    | curio, same name |

Both kept under the delivered names. `intake/sandman/` is empty — group finished.
