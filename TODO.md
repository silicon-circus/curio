# TODO

## Settled — how the Kenney GLBs get published

**Done, 2026-08-30.** None of the three options below; a fourth turned up once the constraint was
looked at properly. The atlas reference is *relative*, and a relative uri resolves against the
`.glb`'s own URL — so it does not have to be embedded to survive, it only has to point at a
**sibling**:

```
images[0].uri:  "Textures/colormap.png"  ->  "pirateship.3d.kenney.colormap.png"

/a/pirateship.3d.kenney.barrel.glb  ->  /a/pirateship.3d.kenney.colormap.png   200
```

That is option 3's flat tagged names with none of option 3's cost. The atlas stays **one** file,
fetched once and cached across every model, instead of 10 kB of base64 duplicated into each and
decoded 16 times. Each file grows by **12 bytes** — the length of the longer string — and nothing
else in it changes at all: BIN chunk byte-identical, every other JSON value identical. And because
the uri stays relative rather than becoming an absolute `/a/` URL, the files remain portable: any
directory holding the model beside its atlas works, including a local copy.

The tool is **`just glb-flatten <src-dir> <prefix> <stem>...`**, kit-agnostic — it reads whatever
external image uri each `.glb` declares and rebases it. It splices the bytes inside the JSON chunk
rather than re-serialising, because a round-trip through a JSON parser reformats every float in the
file (`0.1` -> `0.10000000149011612`) and turns a one-string edit into a rewrite of everything. It
asserts glTF 2.0 magic, header length against file size, the uri actually being present, and then
re-parses its own output to check the BIN chunk survived — a bad rewrite has to fail loudly rather
than produce a file that loads in one parser and not another.

### What was actually used

Sixteen, not "about a dozen" — traced through `loadSharedModel`, the `prop.model` tables in
`hub-shell.js`/`cannons.js`/`masts.js`, the `kenney()` calls in `bottle.html`, `loadModel` in
`world3d.html`, and the array in `training-camp-room.js`:

```
barrel  bottle-large  cannon  cannon-ball  cannon-mobile  chest  crate  mast
palm-detailed-bend  patch-sand  patch-sand-foliage  rocks-sand-a  rocks-sand-b
ship-pirate-large  ship-pirate-small  structure-platform
```

*(A naive grep for stems also matched `flag`, `hole`, `structure` and `bottle` — a slot-machine
symbol, a `redraw` mode, an attraction type. `chest` looked like the same kind of false positive and
was not: `bottle.html:306`. Worth checking each hit in context rather than trusting the count.)*

### The bigger finding: quaernius was 165 MB of nothing

The kit that had "no problem at all" was the problem. **361 files, 171.8 MB, and not one reference
to it anywhere in `siliconcircus.lol/`** — the only mention was this file. It is that large *because*
its 72 `.gltf` inline their buffers as `data:` URIs, and it ships 72 `.blend` sources including a
17 MB `Scene_AllModels.blend`. Weighing 94% of the kit bulk in `names/` and earning none of it.

All three kit directories are now in `trash/` — 734 files, 181 MB — after hashing every one and
confirming its bytes were already in `objects/` (zero missing). `names/` is **flat**: 941 entries,
no subdirectories.

```
names/    941 flat entries          objects/  1571
```

Objects grew by **16, not 17**: the atlas deduped against the copy already stored from the kit.

*(Worth knowing for next time: a `.glb` is not automatically self-contained. It is a container —
header, JSON chunk, binary chunk — and the JSON can still carry a relative `uri` pointing at a file
on disk, which is exactly what Kenney's do. Reading only the `.gltf` files and concluding the kit had
no external references was wrong, and it took parsing the GLB JSON chunk to see it.)*

### The options as they stood

**1. Leave the kit whole.** Costs 9.3 MB, zero work, ugly nested URLs, 357 files riding along.

**2. Publish a slim collection** — chosen `.glb` plus `Textures/colormap.png`. Still a directory,
still nested URLs.

**3. Make each one self-contained** *(the preference at the time)* — re-embed the atlas as a `data:`
URI. Flat tagged names, but +10 kB per model and a base64 decode per file instead of one cached
fetch. Superseded by the sibling rebase, which buys the same flat names for +12 bytes.

The originals are safe three ways over: `objects/` has every byte, `trash/` still holds the extracted
trees, and `vendor/*.zip` has the untouched downloads (379 / 361 / 3 files, verified).

---

## Still open

The Rust port ([PORT.md](PORT.md)) closed most of what used to be here. What is left, honestly sorted.

- **Repoint pirateship at curio.** `training-camp-room.js:11` (`MODEL_ROOT`) and three hardcoded
  `"../assets/3D/kenney-pirate/"` in `resources.js:115`, `world3d.html:55`, `bottle.html:293`; then
  delete its local 3 MB copy. Repoint and confirm the scenes load *before* deleting — the local copy
  has all 72 models and curio serves 16, so anything choosing a model name at runtime rather than as a
  literal string would 404, and a static trace cannot see it.

- **Writes still have no credential.** `keep`, `trash`, `unpublish` and `sync` now require a loopback
  `Host` and a same-origin `Origin`, which closes DNS rebinding and the form-POST CSRF. But that is a
  locality check, not authentication: any local process, and anything inside the reverse proxy, can
  still drive them. A shared token in a header checked by the same middleware would be enough, and
  would let the console be reachable without a tunnel.

- **Backup retention.** `backup/history/` grows with editing, and an archived version starts costing
  real bytes once its extent stops being shared. Keep everything, keep N stamps, or thin to one per
  day after ninety? "Keep everything" is honest while edits are rare and trivial to change later. Note
  that this is only safe *because* `trash/` is separate: nothing in history was put there on purpose.

- **Gamma-correct resizing.** Scaling happens in gamma-encoded sRGB, which is technically wrong and
  slightly darkens high-contrast detail. ImageMagick did the same, so this is parity rather than a
  regression. Linearising first would be *better than the park has ever had* — but renditions would
  visibly differ from the current ones, so it is a deliberate change, not a fix.

- **The intake workflow scatters, and nothing records where anything went.** Art arrives from
  @mjanime in the toplevel `intake/` (not `data/intake/`, so the console never sees it), grouped by
  subject with a README. Naming, rejecting and routing are all done by hand, and a file can end up in
  `assets/`, in a game repo that vendors its own art (`bloom-static`, some `wcfranks` arcade games), or
  nowhere. Nothing writes down which — the dispositions in `notes/` had to be reconstructed by hashing
  bytes weeks later, and one pairing could not be recovered at all.
    Two cheap parts before any larger design: have `keep` work on the toplevel `intake/` too, so
  routing into curio stops being a manual copy that leaves its source behind; and record the delivered
  name alongside the published one, so the mapping is a fact rather than an inference.

- **intake/ can't take a directory.** Dropping a folder there lists its files individually. `watch/`
  handles collections properly; the console's keep/bin path does not. Needs *keep as collection* and
  *keep contents individually* as separate buttons, and a thumbnail rule for a folder.

- **`data/objects/` is 1.6 GB of nothing.** Nothing reads it since the port; `backup/` replaced it.
  `hold/` (8.4 MB, four honeycomb backdrops also in bloom-static) is likewise unknown to the code.
  `vendor/` (46 MB of untouched upstream zips) is worth keeping. Reclaimable whenever, no hurry at
  533 G free.

## Settled by the port, kept for the reasoning

- **Cache invalidation was going to move out of the cache key into `sync`.** It did not, and the
  reasons for wanting it are gone: the key still carries the source's size and mtime, but `cache/` now
  has a size ceiling with oldest-first eviction, so it no longer grows without bound — which was the
  real complaint. The basename collision that made it urgent is fixed by keying on the name relative
  to `assets/`. Worth knowing the runtime stat is still there, and is now a deliberate choice rather
  than an unexamined one.

- **`bin/curio` should absorb the Justfile conveniences.** Done: `--verify --find --todo --uncache
  --dedup --rename`. No Python anywhere, which also deleted `just find`'s shell-into-python injection
  rather than fixing it.
