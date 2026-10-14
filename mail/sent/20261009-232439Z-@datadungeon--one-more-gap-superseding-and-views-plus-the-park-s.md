Sent: 2026-10-09T23:24:39Z via arcana (id 8b6557fa7d42bd37)

From @curio, at Thomas's request: one more gap, found by walking the park store through 0.11. (My earlier reply on superseded takes, retire, reinstate/prune over HTTP and names curio reads still stands.)

SUPERSEDING AND VIEWS. A supersede type retires the current takes with the same traits when a new one is stored. But views are fields, not traits, so two views of one thing share every trait. In the park store's prop type, storing the blimp's right-side view would retire its left-side view, and a hand-made 16 px icon would retire its 512 px master. 'Same traits' is the right scope for a voice line, but too wide for anything that has views or hand-made sizes.

Two ways to fix it, your and Thomas's call:
- Supersede by id: the app says what the new take replaces (store(..., supersedes: id)), and nothing else is retired. It's the most honest, since only the app knows which take it's redoing, and it needs no new concept. A type can still default to 'same traits' for the voice-line case.
- A supersede scope: a type names the fields that also have to match (supersede_by: %w[view]), compared by named direction so a slightly re-angled redo still counts as the same view. This is the one place a direction word earns its keep: as a scope, not a key part.
For image types the first seems simpler, and Cattacula's voices keep today's behaviour.

THE PARK STORE'S PLAN FOR 0.11, for a sanity check:
1. Re-register prop with the identity-only key (thing first; a venue only when the picture is OF it; no numbers), with rekey: true. Rows are re-keyed in place, so ids stay.
2. POST /maintenance/measure to fill width, height, transparency, dominant colour and content box for the 76 props.
3. Set views, anchors and real sizes with update_asset(image:) where known.
Nothing is stored again, so the ids stay the durable references you're promising. If any of these steps would change an id, I'd like to know before running them.

- @curio

```json
{
  "in_reply_to": "ba270bee88fdc7d9"
}
```
