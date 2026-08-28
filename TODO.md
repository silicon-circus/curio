# TODO

## Decide how the Kenney GLBs get published

**Undecided.** Leaning towards making them self-contained, to avoid any potential issues — but not
settled.

### The finding

Only about a dozen models out of the Kenney pirate kit are actually used, and all of them are
`.glb`. The question was what forces those to stay inside a kit directory at all. The answer is
exactly one file:

```
72 .glb, 3.0 MB total
   every one of them references  Textures/colormap.png   (10 kB, shared atlas)
whole kit on disk: 9.3 MB, 370 files
```

Nothing else in the kit is referenced by anything in use. The `.obj` chain to `.mtl` to the same
atlas, the `.fbx` are self-contained, the `.blend` are source. The other two kits don't have this
problem at all — the quaernius kit's 72 `.gltf` inline their buffers as `data:` URIs and reference
nothing; the tron cycle is one `.gltf` plus one `scene.bin`.

*(Worth knowing for next time: a `.glb` is not automatically self-contained. It is a container —
header, JSON chunk, binary chunk — and the JSON can still carry a relative `uri` pointing at a file
on disk, which is exactly what Kenney's do. Reading only the `.gltf` files and concluding the kit had
no external references was wrong, and it took parsing the GLB JSON chunk to see it.)*

### The options

**1. Leave the kit whole.** Already done, costs 9.3 MB, zero work. Reference the dozen you use at
`/a/pirateship.3d.kenney.pirate.kit/Models/GLB format/barrel.glb`. The URLs are ugly and the other
357 files ride along for ever.

**2. Publish a slim collection** — a `kenney-pirate/` holding only the chosen `.glb` plus
`Textures/colormap.png`. ~0.5 MB, keeps Kenney's files byte-identical, and the full kit stays in
`objects/` and `data/ok/` if another model is ever wanted. Still a directory, still nested URLs.

**3. Make each one self-contained** *(current preference)* — re-embed the atlas into each `.glb` as
a `data:` URI, so the file depends on nothing. Each model then lives loose in `names/` under a proper
tagged name — `pirateship.3d.kenney.barrel.glb` — with a flat URL and no collection machinery at all.

Costs: roughly +10 kB per model (the atlas is duplicated into each), and the files stop being
byte-identical to Kenney's originals. Worth checking whether three.js/GLTFLoader handles an embedded
base64 image as fast as an external one — it should, but it is a decode rather than a fetch.

The originals are safe either way: `data/ok/` has the tree, `objects/` has every byte, and
`vendor/*.zip` has the untouched downloads.

---

## Smaller things

- **Collections should be one row in a listing.** `/api/serve` and `just find kenney` return 371
  rows, one per file inside the kit. It reads exactly like the flat mess it replaced — someone
  searching with Finder was misled by it once already. Group by collection, expandable.

- **Format derivation.** `/a/foo.webp` from a `foo.png` master, deriving downward only (png/jpg →
  webp, never the reverse) so a master is always available in its own format and nothing fabricates a
  lossless original out of lossy bytes.

- **intake/ can't take a directory.** Dropping a folder there lists its files individually. `watch/`
  handles collections properly now; the console's keep/bin path does not. Needs *keep as collection*
  and *keep contents individually* as separate buttons, and a thumbnail rule for a folder (Kenney
  kits carry a `Previews/`; otherwise first image, or a count).
