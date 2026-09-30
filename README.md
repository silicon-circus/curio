# curio

The Silicon Circus asset server. Everything the park draws, plays or loads is served from here **by
name**, so no repo holds a copy of anything.

    just run                             # http://localhost:26037

(`just run` builds `--release` and exports the paths. A plain `shards build` quietly writes a debug
binary to the same path, which is easy to leave behind by accident.)

## Requirements

| | |
|---|---|
| **Crystal** ≥ 1.19.1 | `shards install` pulls Kemal, the only dependency |
| **ImageMagick 7** — `magick` | every resize and every format conversion shells out to it |
| **A reflink filesystem** | XFS, btrfs, bcachefs, APFS, OpenZFS 2.2+ — see below. ext4 works, it just stops being free |
| `just` *(optional)* | a convenience layer over `bin/curio`; nothing requires it |

Nothing checks for ImageMagick. The server starts without it, serves every master happily, and fails
only on the first request that needs a conversion — so confirm it before wondering why
`/a/foo.webp` is a 500:

    magick -version

## The workflow

1. Something generates an asset and drops it in **`data/intake/`**.
2. You look at it in the console. Keep it (with a name) or bin it. **This is the only chore.**
3. Or, if you already know the name: put the file in **`data/watch/`** named the way you want it
   served, then run `just watch`. It prints exactly what it intends to do and asks before doing any
   of it — a folder you are still arranging looks identical to one you have finished arranging, and
   only you know which.
4. Reference it: `/a/boardwalk.cattacula.night.real.webp`, `?w=640` for a resize.

Need a touch-up? Open the file in `data/names/` and edit it. Refresh. Done — same name, same URL,
nothing to update, and the version you replaced is kept.

## The folders

| | |
|---|---|
| `watch/` | drop it with the name you want; filed automatically |
| `intake/` | not looked at yet — keep or bin |
| `names/` | **your working set.** The filename is the URL. Edit these freely |
| `objects/` | the archive: every version ever synced, by hash, immutable |
| `cache/` | renditions. Disposable — `rm -rf` and it rebuilds |
| `trash/` | what you threw away, in case you didn't mean it |

## Why names are reflink copies and not hardlinks

This is the part that makes editing safe, and it took a measurement to get right.

A hardlink is the same inode under two names. So an in-place edit — GIMP overwriting,
`magick foo.png foo.png`, anything that opens for writing rather than writing-and-renaming — reaches
*through* the name and rewrites the object. The store then holds bytes that don't hash to their own
filename, and every other name pointing at that object has silently changed too:

```
start              obj=original bytes    name=original bytes     same inode
in-place write     obj=EDITED in place   name=EDITED in place    same inode   ← corrupted
write + rename     obj=original bytes    name=EDITED via rename  broken link  ← safe
```

A **reflink copy** is a separate inode sharing the same extents. Editing it diverges only the blocks
you touched, and `objects/` can't be reached from `names/` at all.

On a filesystem with reflink this costs nothing:

```
writing a 200 MB master   190.7 MB
reflink copy of it          0.0 MB      shared extents
5 MB file, one byte edited  0.1 MB      only the changed block
```

Supported by **XFS** (reflink=1, the mkfs default since 2018), **btrfs**, **bcachefs**, **OCFS2**,
**OpenZFS 2.2+**, **APFS**, **ReFS**, and **NFS 4.2**. *Not* by **ext4**, where `cp` falls back to a
real copy — still correct, just no longer free. Reflinks can't cross a mount, so `data/` must live
on one filesystem.

Note that `du` over-reports as a result: it counts shared extents once per file, so `data/` reads as
3.0 GB when the disk cost is 1.6 GB. Trust `df`.

## Rules the server keeps

**Objects are never written except by sync, and never deleted.** Touching up a picture doesn't
destroy what it replaced — the old bytes keep their hash and simply stop having a name. That's
version history, and it's free.

**watch/ never overwrites.** A file whose name is already served is left exactly where it is and
reported in the console — not moved, not renamed, not merged, and not silently dropped even when the
bytes are identical. "I moved it to watch/ and it vanished" is indistinguishable from "it worked",
and those are very different things to have happened.

**Cache keys carry size and mtime**, not just the name. Edit a master and its renditions invalidate
themselves on the next request. A stat, not a hash — this runs on every request, and hashing a 3 MB
master to serve a 14 kB thumbnail is not a trade.

## Asking for a format that isn't on disk

`/a/foo.webp` is answered from `foo.png` if no `foo.webp` exists. The conversion happens once, is
cached like any other rendition, and the second request is a static file read.

**An existing webp wins over a generated one.** You always get the format you asked for; the only
question is where those webp bytes come from. 42 names are published as *both* `.png` and `.webp`,
and for those the file you made by hand is served as-is rather than re-encoded out of the png.
Derivation fills a gap, it does not overrule a master.

**Downward only.** png/jpg → webp, and nothing in the other direction. A `.png` conjured out of
`.webp` bytes would be a lossless-looking file that is nothing of the sort, and every name it was
served under would be a quiet lie about what the archive holds. Ask for a format no master can
legally produce and you get a 404, not a fake.

**Quality follows the source**, because 82 does not mean the same thing twice. Encoding webp from a
png is compressing a picture, and the artifacts are the first the file has ever carried — 82 is
plenty. Encoding webp from a jpg is a *second* lossy pass over bytes that are already dented, and
the encoder cannot tell inherited artifacts from detail, so it gets more room: 90. Same-format
resizing keeps the old 88. `?q=` overrides all three.

Both dials compose: `?w=640` on a derived format resizes and converts in one pass, one cache entry.

    /a/afterimage.tron.vista.png            1.75 MB   the master
    /a/afterimage.tron.vista.webp            172 kB   derived, q82
    /a/afterimage.tron.vista.webp?w=320     12.6 kB   derived and resized

## Commands

`bin/curio` is the whole program. The Justfile is a convenience layer over it, and it exports
`CURIO_PORT`, `CURIO_DATA` and `CURIO_PUBLIC` for every recipe that starts the binary — so a dev run
never depends on the path baked into it.

    bin/curio              serve
    bin/curio --watch      file what is in watch/, after printing the plan and asking
    bin/curio --sync       hash names/, file anything new into objects/
    bin/curio --migrate    hardlink an older layout in (non-destructive)

    CURIO_PORT    26037
    CURIO_BIND    127.0.0.1    0.0.0.0 to expose it — read Deploying first
    CURIO_DATA    ../data      resolved against the SOURCE DIR AT COMPILE TIME
    CURIO_PUBLIC  ../public    likewise

And through `just`, which on its own lists every recipe:

    just dev             run from source; an edit to src/ is live on restart
    just run             build --release, then serve
    just check           does it compile — no binary, fastest feedback
    just watch [yes]     the plan, then ask. `just watch yes` skips the asking
    just sync            file edits in names/ away into objects/
    just verify          re-read every object, check it still hashes to its own name
    just find TERMS      search the names — `just find cattacula night`
    just todo            what waits in intake/, and what watch/ is holding
    just du              what the store actually costs (du lies; df does not)
    just uncache         throw away every rendition
    just glb-flatten     rebase a kit's .glb onto flat tagged names
    just ping / open     health, and the console in a browser

Anything `just` can do that `bin/curio` cannot is a gap in the binary rather than a feature of the
Justfile. The current list is `find`, `todo`, `verify` and `uncache`.

## The HTTP surface

| | | |
|---|---|---|
| `GET` | `/a/*name` | the asset, by name. `?w=` width, `?q=` quality |
| `GET` | `/` | the console |
| `GET` | `/health` | liveness and the counts |
| `GET` | `/api/serve` | what is published. `?q=` filters, `?limit=` caps |
| `GET` | `/api/intake` | what is waiting to be kept or binned |
| `GET` | `/api/watch` | what `watch/` would do, as a plan — it does not run it |
| `GET` | `/intake/*file` | preview a file that has not been kept yet |
| `POST` | `/api/keep` | keep an intake file under a name |
| `POST` | `/api/trash` | bin an intake file |
| `POST` | `/api/unpublish` | remove a name. The object stays |
| `POST` | `/api/sync` | as `--sync` |

Those four `POST` routes change the store and **none of them authenticates**. The `GET` routes send
`Access-Control-Allow-Origin: *` so any park repo can embed an asset from its own origin; the `POST`
routes deliberately do not, so a page in another tab cannot drive them.

## Deploying

Two things will bite before anything else does.

**There is no authentication, and four routes change the store.** That is the right shape for a tool
serving one machine's browser and the wrong shape for anything else — so it binds `127.0.0.1`, and
exposing it is something you have to type. `CURIO_BIND=0.0.0.0` belongs only behind something that
terminates the public side and forwards **reads only**:

    location /a/     { proxy_pass http://127.0.0.1:26037; }
    location /health { proxy_pass http://127.0.0.1:26037; }
    # everything else — / and /api/ and /intake/ — is simply not published

That is the whole security model, and it is a routing decision rather than a feature: the mutating
surface is never reachable, and the console is reached over SSH or a tunnel instead of being
published. Before curio can accept a write from anywhere but localhost it needs a credential, and it
does not have one yet.

**The binary is not relocatable.** `CURIO_DATA` and `CURIO_PUBLIC` default to `../data` and
`../public` resolved against the source directory *at compile time* — Crystal bakes `__DIR__` in. A
binary built in `/home/you/curio` and copied to a server therefore goes looking for
`/home/you/curio/data`, finds nothing, and serves an empty store while reporting `"ok": true`. The
counts in `/health` are the tell. Build in place, or set both:

    CURIO_BIND=0.0.0.0 \
    CURIO_DATA=/srv/curio/data \
    CURIO_PUBLIC=/srv/curio/public \
    bin/curio

`data/` has to live on one filesystem, because reflinks cannot cross a mount.

Nothing in the park hardcodes the port: boardwalk resolves names through `SC_ASSET_BASE`, so
repointing every venue at a different store — staging, a colleague's — is one variable.
