# Porting curio to Rust

Not a translation. The Crystal works; what it lacks is bounded surfaces, a test suite, and a backup
story that can answer "what was this file yesterday". Those get designed here and built once, rather
than bolted onto a rewrite.

Decided in session, 2026-09-30. **Crystal keeps serving until Rust reaches parity**, then the `.cr`
files and `shard.yml` go in one commit. `Cargo.toml` sits beside `shard.yml`; Cargo takes
`src/main.rs`, Crystal takes `src/curio.cr`, and they do not collide.

## The stack, and the one place a naive port regresses

    axum                  HTTP. The default choice, and the one I am most reliable in.
    image                 decode
    fast_image_resize     downscale — WITH PREMULTIPLIED ALPHA
    webp (libwebp)        lossy encode

`image::imageops::resize` does **not** premultiply alpha, and this store is mostly RGBA: 41 of 60
sampled PNGs are `srgba`, and the die-cut props (`bee-badge.top.png`, `compass.dark-wood.png`,
`hydrant.red.day.png`) are `alpha-min=0` — fully transparent at the edges. Resizing those
channel-by-channel drags the arbitrary colour of transparent pixels into the visible edge and haloes
exactly the silhouettes that were carefully die-cut. `fast_image_resize` premultiplies explicitly
and is SIMD-accelerated, so it is also faster than ImageMagick.

**Verify this rather than trusting it.** Render `bee-badge.top.png` at `w=320` both ways and compare;
1024² RGBA with a thin gold satin-stitch border on full transparency is the case that shows fringing
if it exists.

The `image` crate's WebP encoder is lossless-oriented, so lossy encode goes through libwebp
bindings. That costs the pure-Rust static binary and buys the reference encoder.

**Dropping ImageMagick is the largest single win**: no subprocess, so no missing-binary 500, no
unbounded `magick` fan-out, no delegate/coder surface, and a concurrency limit becomes a semaphore
around in-process work with predictable memory.

## Downward only — in format AND in size

The existing rule is "derive downward only; never fabricate a lossless original out of lossy bytes."
It extends to dimensions: **never fabricate pixels that were never there.**

Evidence that upscaling is not wanted: the only widths ever requested in the live cache are two
`w200` and one `w320`, and *no project outside curio uses `?w=` at all* — every reference in the park
is curio's own README, comment, or console blurb. Interpolation cannot add detail anyway (soft, with
ringing, tolerable to ~1.5x). If a bigger master is needed, re-render it — the art is generated in
the first place — and put it in `assets/`. AI super-resolution belongs in authoring, not serving.

So `w` is clamped to the master's width. The 257 MB-from-one-request case disappears as a
consequence of the rule, not as a limit bolted on to stop it.

## Bounded surfaces

`?w=8192&q=0` wrote 257 MB twice from a 40-byte request, and 40 concurrent requests produced 41
`magick` processes at ~10 GB RSS. Four separate causes, four separate fixes — an allowlist of widths
was the wrong answer to all of them:

| cause | fix |
|---|---|
| upscaling | clamp `w` to the master's width (above) |
| unbounded key space | **quantise** `w` up to a ladder, do not allowlist — a caller asking 641 gets 704 and a correct image, nothing 404s, `srcset` keeps working, key space drops from 8192 to ~30 per name |
| `q` on a lossless target | `q` applies only when the target is lossy. On PNG output it means zlib level, so `q=1` *inflates* the file: measured 415 kB at q=1 against 257 kB at q=95 |
| no concurrency limit | one semaphore around rendering |
| unbounded cache | size cap with eviction. Renditions stop being written into a permanent store at all |

## Folders

    assets/   what you edit. The filename is the URL
    watch/    drop it named; files itself after showing the plan and asking
    intake/   the one chore: keep or bin
    backup/   current/ + history/<stamp>/ — read-only (below)
    cache/    renditions, disposable, size-capped
    trash/    what you threw away

`names/` becomes **`assets/`**: it named the mechanism rather than the contents, the same flaw
`archive` had. The URL is already `/a/`, which now reads as "assets" instead of a leftover.

`objects/` is **deleted**. Measured: it saved 35 files of dedup out of 1051, held 749 objects
referenced by no current name, and kept **no record of what any of them used to be called** — so
"version history, and it's free" was retention without recall. Its removal also takes `ingest`,
`publish`, `object_path`, the never-hardlink rule, and the reflink essay's reason for existing.

## Backup — built in, not a timer

On XFS with reflink, a read-only mirror of `assets/` costs **504 KB and 27 ms** for 1.4 GB apparent.
Measured. So there is no reason not to always have one.

    backup/current/            read-only reflink mirror of assets/ as of the last sync
    backup/history/<stamp>/    only the versions that were REPLACED, read-only

`sync`, for each name whose bytes differ from the manifest:

1. move `current/<name>` to `history/<stamp>/<name>` — **that is the pre-edit version**
2. refresh `current/<name>` from the new master, `chmod a-w` again

New names are added to `current/`. Deleted names have their `current/` copy moved to `history/`, so
unpublishing keeps the bytes. Nothing is written when nothing changed.

Why this works when "copy the old bytes at sync time" does not: by the time sync runs, the old bytes
are already overwritten. They survive because `current/` captured them at the *previous* sync —
exactly how `objects/` worked, addressed by name and date instead of by hash.

Mechanics verified: a read-only reflink copy refuses an in-place overwrite (`permission denied`,
file unchanged) while `mv` into `history/` still succeeds, because rename needs write on the
directory, not the file. Read-only stops a tool saving over it; it is not meant to stop a deliberate
`chmod`.

## The manifest, and what the hash is actually for

The hash stays — **as an identifier, not a filename**. Its job is identity across a rename, which
nothing else can provide:

| change | signal | result |
|---|---|---|
| edit, same name | hash changed | detected |
| rename, same bytes | new name's hash already known | detected |
| rename **and** edit | new name, unknown hash | indistinguishable from delete + add |

Today's sync does no rename detection at all — it is silently delete-plus-add — so this is a new
capability, not one to preserve.

Two caveats. 30 hashes are shared by 65 names (largest group 4), so for ~6% "renamed from *which*?"
is ambiguous even on a hash match; the bytes are safe regardless. And the inode helps partially:
measured, it survives `mv` **and** an in-place truncate-write, but not write-and-rename, which is
what GIMP and `magick foo.png foo.png` do — so store it as a hint, never rely on it.

For the ambiguous case, **sync asks**, in the shape `watch/` already uses:

    boardwalk.deck.png disappeared, boardwalk.dock.png appeared (bytes not seen before)
      renamed and edited?   or unrelated?

*"A folder he is still arranging looks exactly like a folder he has finished arranging, and only he
knows which."* Same principle. Plus `curio rename <old> <new>` for stating it up front.

## Duplicates: reflink, never hardlink

Aliasing one asset under several names is legitimate — `pirateship.deck-plank.webp` and
`boardwalk.deck-plank.webp`, `pirateship.oar.prop.webp` and `pirateship.training.oar.webp`, three
separate `*.favicon.svg`.

**Reflink, not hardlink.** A hardlink is one inode under two names, so an in-place edit through one
alias silently rewrites the other — the exact corruption the reflink section of the README exists to
reject. A reflink shares extents and diverges on write. Measured: a reflink copy of a 2.3 MB
duplicate costs 0 KB.

So: on `sync`/`keep`/`watch`, if an asset's bytes match one already held, write it as a reflink. Free,
and independent under editing. Plus a `--dedup` pass for the 48.6 MB already sitting in 65 names —
though 37% of those are import cruft (`_venues-used-backup`, `_wcfranks-prev`) that probably wants
deleting rather than deduping.

## No Python

Five Justfile recipes shell out to `python3`. All are the binary's own work, and three of them curl a
running server to ask about files on the same disk. `--verify`, `--find`, `--todo`, `--uncache` and
`glb-flatten` move into `curio`. The Justfile keeps only what it is good at: setting `CURIO_PORT` /
`CURIO_DATA` / `CURIO_PUBLIC` and remembering flags.

This also deletes a bug rather than fixing it — `just find` currently splices `{{ terms }}` into a
shell string and then a Python literal, and executes injected code (`just find "x'; import os; ..."`
→ reproduced).

## Security invariants the port must keep

Found by independent review; several are already fixed in Crystal and must not be lost in translation.

- **Containment.** Any request-supplied path joined to a directory must be proved to stay inside it,
  lexically *and* via realpath for symlinks. `keep`, `discard`/`unpublish` all reached arbitrary
  files — read, publish, **and delete** — before this existed.
- **Loopback by default.** Bind `127.0.0.1`; exposure is opt-in via `CURIO_BIND`.
- **Writes prove they are local.** Loopback `Host`, and an `Origin` that is this server when one is
  sent. CORS is not a control: a form POST needs no preflight, and nothing validated `Host`, so DNS
  rebinding made an attacker's page same-origin.
- **`ACAO: *` on `/a/` and `/health` only.** Not on `/api/*` or `/intake/*` — any page could read the
  whole index and the bytes of anything in the undecided pile. *(not yet fixed in Crystal)*
- **Content-type allowlist on `/a/`.** It currently serves `text/html` and `image/svg+xml`, which is
  stored XSS on curio's own origin, cached a year, same-origin to every mutating route. *(not yet
  fixed)*
- **Clamp `limit`; production error pages.** `?limit=-1` returns Kemal's *development* exception page,
  61 kB with a backtrace, carrying `ACAO: *`. *(not yet fixed)*
- **Cache key on the relative path**, not `File.basename` — two collection files collide and serve
  each other's image. Latent only while `assets/` is flat. *(not yet fixed)*

## Caching, decided now rather than later

`max-age=31536000` with no validator contradicts a store whose premise is in-place editing: edit a
master and every client holds the old bytes for a year with no way to revalidate. The route comment
claims "a weak validator for the name itself"; it was never implemented.

`ETag` from the manifest hash for masters and from the cache key for renditions, plus a short
`max-age`. This also makes the truncation class of bug recoverable instead of permanent.

## Order of work

1. Scaffold, config, manifest, `/health`. Tests from here on, not after.
2. `GET /a/*name` for stored assets — containment, content-type allowlist, ETag.
3. Derivation: decode → premultiplied downscale → lossy encode, semaphore, quantised `w`, capped
   cache, staged-then-renamed writes. **Compare against ImageMagick output on the die-cut props.**
4. `sync`: manifest, rename detection, backup `current/` + `history/`, reflink dedup.
5. `watch`, `intake`, `keep`/`trash`/`unpublish` with the `Host`/`Origin` gate.
6. `--verify --find --todo --uncache`, `glb-flatten`.
7. Console against the Rust server; `derivable` in `/api/serve`.
8. Parity check against the Crystal on the live store, then delete the `.cr` files, `shard.yml` and
   the Python recipes in one commit.

## Carry over verbatim

The comments are the asset, not the code. Move them across; do not regenerate them.

The reflink measurement and the hardlink corruption table. "A `.glb` is not automatically
self-contained." The derivation rules and why quality follows the source. `watch/` never overwriting,
and why. The naming convention. Tom's quote at the top of `curio.cr`.

## Open

- **Is `trash/` still pulling its weight?** `history/` holds "this was replaced", `trash/` holds "I
  threw this away", and unpublishing is arguably the last kind of replacement. Two attics may be one
  too many.
- **Backup retention.** History grows with editing, and an archived version starts costing real bytes
  once its extent stops being shared. Keep everything, keep N stamps, or thin to one per day after 90?
  "Keep everything" is honest while edits are rare and easy to change later.
- **Gamma-correct resizing.** ImageMagick's default resizes in gamma-encoded sRGB, which is
  technically wrong and slightly darkens high-contrast detail; a naive port inherits it. Linearizing
  first would be *better than today* — but renditions would visibly differ from current ones.
  Improvement, not parity.
- **The `w` ladder.** Quantisation step: 64px increments, or geometric?
