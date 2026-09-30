# curio

The Silicon Circus asset server. Everything the park draws, plays or loads is served from here **by
name**, so no repo holds a copy of anything.

    just run                             # http://localhost:19463

## Requirements

| | |
|---|---|
| **Rust** stable (1.90+) | `cargo build --release`; the only build step |
| `just` *(optional)* | remembers flags and sets three environment variables; nothing requires it |

That is the whole list. Decoding, scaling and encoding happen in process — there is no ImageMagick
to install and no subprocess to fail. A **reflink filesystem** (XFS, btrfs, bcachefs, APFS,
OpenZFS 2.2+) makes the backup mirror free rather than merely correct; see below.

## The workflow

1. Something generates an asset and drops it in **`data/intake/`**.
2. You look at it in the console. Keep it (with a name) or bin it. **This is the only chore.**
3. Or, if you already know the name: put the file in **`data/watch/`** named the way you want it
   served, then run `just watch`. It prints exactly what it intends to do and asks before doing any
   of it — a folder you are still arranging looks identical to one you have finished arranging, and
   only you know which.
4. Reference it: `/a/boardwalk.cattacula.night.real.webp`, `?w=640` for a resize — never larger than the master.

Need a touch-up? Open the file in `data/assets/` and edit it. Refresh. Done — same name, same URL,
nothing to update, and the version you replaced is kept.

## A store is a project

You point curio at a project root, the way you point git at a repository. One global store for
everything still works — set `CURIO_DATA` and never move it — but per-project isolates one asset set
from another, makes a project portable and backed up as a unit, and matters more as more consumers
than boardwalk appear.

The directories are split by **how much you would mind losing them**, which is why two of them do not
live in the project at all:

| | | |
|---|---|---|
| `assets/` | in the project | **your working set.** The filename is the URL. Edit these freely |
| `intake/` | in the project | not looked at yet — keep or bin |
| `watch/` | in the project | drop it in named; filed after showing you the plan |
| `trash/` | in the project | what **you** threw away |
| `manifest.tsv` | in the project | what curio knows without re-reading 1.4 GB |
| the cache | `~/.cache/curio/<project>` | renditions. Regenerable, so out of the working tree |
| the backup | `~/.local/share/curio/<project>/backup` | `current/` mirrors `assets/`, `history/<stamp>/` keeps what was replaced |

`<project>` is the root's own name plus a digest of its absolute path, so two projects called `art`
cannot collide. Override either with `CURIO_CACHE` and `CURIO_BACKUP`; `CURIO_BACKUP=none` turns the
backup off entirely, which is what a headless server wants — its masters arrived from a machine that
already holds their history.

A safety copy is never kept inside the thing it is a copy of, and XDG *data* rather than XDG cache
for the backup, because it is the one directory here whose loss cannot be undone by regenerating it.

`trash/` and `backup/history/` are not two attics. They differ by who decided and why: `trash/` is
yours — *"I probably don't need this, but I'm not deleting it yet"* — and is never pruned on a
schedule. `history/` is the system keeping bytes nobody asked it to keep, which is exactly why it
*can* be thinned. Mixing them would mean a retention policy that quietly deletes things you set aside
on purpose.

There is no `objects/`. It saved 35 files of deduplication out of 1051, held 749 objects that no name
pointed at, and kept no record of what any of them used to be called — so "version history, and it's
free" was retention without recall.

## Why the backup mirror is reflinks and not hardlinks

This is the part that makes editing safe, and it took a measurement to get right.

A hardlink is the same inode under two names. So an in-place edit — GIMP overwriting, anything that
opens for writing rather than writing-and-renaming — reaches *through* the name and rewrites the
other copy. The backup would then hold the very bytes it exists to preserve you *from*, and any
asset aliased under a second name would have silently changed too:

```
start              obj=original bytes    name=original bytes     same inode
in-place write     obj=EDITED in place   name=EDITED in place    same inode   ← corrupted
write + rename     obj=original bytes    name=EDITED via rename  broken link  ← safe
```

A **reflink copy** is a separate inode sharing the same extents. Editing it diverges only the blocks
you touched, so `backup/` cannot be reached through `assets/` at all. The same reasoning is why
`curio --dedup` reflinks byte-identical aliases rather than hardlinking them.

On a filesystem with reflink this costs nothing:

```
writing a 200 MB master   190.7 MB
reflink copy of it          0.0 MB      shared extents
5 MB file, one byte edited  0.1 MB      only the changed block
```

Supported by **XFS** (reflink=1, the mkfs default since 2018), **btrfs**, **bcachefs**, **OCFS2**,
**OpenZFS 2.2+**, **APFS**, **ReFS**, and **NFS 4.2**. *Not* by **ext4**, where `cp` falls back to a
real copy — still correct, just no longer free. Reflinks can't cross a mount, so `data/` must live
on one filesystem. Measured here: a read-only mirror of 1051 assets, 1.4 GB apparent, cost **804 KB
and 1.4 seconds**.

Note that `du` over-reports as a result: it counts shared extents once per file, so `data/` reads as
3.0 GB when the disk cost is 1.6 GB. Trust `df`.

## Rules the server keeps

**Every replaced version is kept, under the name it had.** `sync` files it into
`backup/history/<stamp>/` before refreshing the mirror. That is only possible because the mirror
captured it at the *previous* sync — by the time sync notices an edit, the old bytes are already
overwritten.

**watch/ never overwrites.** A name already being served is left exactly where it is and reported —
not moved, not renamed, not merged, and not silently dropped even when the bytes are identical. "I
moved it to watch/ and it vanished" is indistinguishable from "it worked", and those are very
different things to have happened. Every destination is re-tested at the moment of writing, not only
when the plan was printed.

**Renames are noticed, and guessed at never.** A name gone with its exact bytes under a new name is a
rename, recorded in `backup/history/<stamp>/.renames.tsv`. A rename *and* an edit at once cannot be
linked by the bytes, so sync reports the pair and asks — the same shape `watch` uses, for the same
reason.

**Writes come from this machine only.** curio binds `127.0.0.1` and the four mutating routes require
a loopback `Host` and a same-origin `Origin`. CORS is not a control: a form POST needs no preflight.

**Cache keys carry the source's size and mtime.** Edit a master and its renditions are never asked
for again. A stat, not a hash — with one known blind spot: an edit that changes no byte count inside
the same second is invisible to it. `curio --verify` is what catches that.

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

**Never larger than the master.** The format rule is "downward only"; so is the size rule — curio does
not fabricate pixels that were never there. `?w=8192` on a 1024px picture is the picture. Widths are
quantised up to a multiple of 64 rather than allowlisted, so a caller asking for 641 gets 704 real
pixels instead of a 404 and `srcset` keeps working. Quality applies only to lossy targets: on a PNG
the number is a zlib level, and a low one makes the file *bigger*.

Scaling uses Mitchell rather than Lanczos3, and the reason is the encoder rather than the pixels —
sharpening a downscale is ringing, and ringing is detail a lossy codec must spend bytes on. Measured
against ImageMagick over sixteen real masters at 320px: Lanczos3 ran +19% median, Mitchell −2.6%.
Full-size conversion with no resize is byte-identical to ImageMagick, because at that point both are
just libwebp at the same quality.

Both dials compose: `?w=640` on a derived format resizes and converts in one pass, one cache entry.

**Asking what is servable is one request.** A derived rendition is never an index entry — only the
master is stored — so `/api/serve` reports both: `items` is what is on disk, `derivable` is every
name those masters can additionally answer as, with the master each one comes from. Union the two and
"will curio serve this name?" is set membership, with no need to know the rules out here:

    GET /api/serve?limit=5000   ->   { total: 1051, items: [...],
                                       derivable_total: 485, derivable: [{name, from, url}, ...] }

A name published in its own right is never listed as derivable, so the 42 stems held as both `.png`
and `.webp` appear once, as the files they are.

    /a/afterimage.tron.vista.png            1.75 MB   the master
    /a/afterimage.tron.vista.webp            172 kB   derived, q82
    /a/afterimage.tron.vista.webp?w=320     12.6 kB   derived and resized

## Commands

`curio` is the whole program. The Justfile remembers flags and exports `CURIO_PORT`, `CURIO_DATA` and
`CURIO_PUBLIC`; it implements nothing.

    curio                     serve
    curio --sync [--dry-run] [--yes]
                              file edits away, keep what they replaced, ask about a rename
    curio --watch [--yes]     file what is in watch/, after showing the plan
    curio --verify            re-hash assets/ and check it against the manifest
    curio --find TERMS        search what will be served, derivations included
    curio --todo              what waits in intake/, and what watch/ is holding
    curio --uncache           throw away every rendition
    curio --dedup [--yes]     share extents between byte-identical assets
    curio --rename OLD NEW    state a rename so the history link survives

    CURIO_PORT    19463
    CURIO_BIND    127.0.0.1    0.0.0.0 to expose it — read Deploying first
    CURIO_DATA    ./data       relative to the working directory, NOT to the binary
    CURIO_PUBLIC  ./public
    CURIO_CACHE                renditions (default ~/.cache/curio/<project>)
    CURIO_BACKUP               safety copy (default ~/.local/share/curio/<project>/backup)
                               `none` turns it off — for a server whose masters came from elsewhere
    CURIO_RENDER_JOBS          concurrent renders (default: half the cores, max 4)
    CURIO_CACHE_MAX_MB         cache ceiling (default 2048)

`just` wraps each of these — `just sync`, `just watch yes`, `just find cattacula night`, `just verify`,
`just todo`, `just uncache`, `just dedup`, `just du`, `just ping`, `just open`.

## The HTTP surface

| | | |
|---|---|---|
| `GET` | `/a/*name` | the asset, by name. `?w=` width, `?q=` quality |
| `GET` | `/` | the console |
| `GET` | `/health` | liveness and the counts |
| `GET` | `/api/serve` | what will be served — `items` are stored, `derivable` are names they can answer as. `?q=` filters both, `?limit=` caps both |
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

`deploy/curio.service` and `deploy/env.sample`, the same shape as every other venue in the park.
`EnvironmentFile=/etc/curio/env`; the unit's `StateDirectory=curio` and `CacheDirectory=curio` create
`/var/lib/curio` and `/var/cache/curio` owned by the service user, and both stay writable under
`ProtectSystem=strict` without a `ReadWritePaths` line. There is deliberately no backup directory on
a server: `CURIO_BACKUP=none`, because the masters arrived from a machine that already holds their
history, so recovery is another rsync rather than a restore.

**Build on the target, or in a container matching it.** The one native dependency is libwebp, vendored
and linked statically by `libwebp-sys`, so the binary needs only glibc, libm and libgcc at runtime —
but *which* glibc matters. A binary built on a rolling distribution will refuse to start on a stable
one with version-symbol errors. Building on the box needs a Rust toolchain and a C compiler.


**There is no authentication, and four routes change the store.** So the question is never whether
curio should be reachable, it is by whom — and the answer is the proxy and nothing else. With the
proxy on the same box, **keep `CURIO_BIND=127.0.0.1` in production too**: the browser resolves the
public vhost, the proxy reaches curio over loopback, and the raw port is on no external interface no
matter what happens to the firewall later. curio generates no absolute URLs, issues no redirects and
reflects no origins, so it cannot tell the difference. `0.0.0.0` is only for a proxy on a *different*
host, and then the firewall becomes load-bearing. Either way the vhost forwards **reads only**:

    location /a/     { proxy_pass http://127.0.0.1:19463; }
    location /health { proxy_pass http://127.0.0.1:19463; }
    # everything else — / and /api/ and /intake/ — is simply not published

That is the whole security model, and it is a routing decision rather than a feature. The mutating
surface is never reachable, and the console is reached over SSH or a tunnel instead of being
published.

**Set `CURIO_DATA`.** It defaults to `./data` relative to the *working directory*, so a service
started from elsewhere will not find the store. It says what it resolved and how many assets it found
on the first two lines of its log, and an empty store is always that message rather than a mystery.

    CURIO_BIND=0.0.0.0 \
    CURIO_DATA=/srv/curio/data \
    CURIO_PUBLIC=/srv/curio/public \
    curio

`data/` has to live on one filesystem, because reflinks cannot cross a mount. The backup does too, if
you want it free rather than merely correct — a reflink cannot cross from the project to another
mount, so a backup on a different disk is a real copy. That is the right trade for a backup on a
different disk.

**A headless server wants less than a workstation.** Only `assets/` is persistent there; the cache is
scratch and the backup belongs on the machine that authors the art:

    CURIO_BIND=127.0.0.1 \
    CURIO_DATA=/var/lib/curio \
    CURIO_CACHE=/var/cache/curio \
    CURIO_BACKUP=none \
    curio

A store whose `assets/` already exists is treated as one somebody else filled, so `intake/`, `watch/`
and `trash/` are not created — no empty directories implying chores that happen elsewhere.

**Running it under systemd** with `ProtectHome=read-only` needs the two outside directories declared,
or every derivation fails with a 500 whose only explanation is in the log:

    ReadWritePaths=%h/.cache/curio
    ReadWritePaths=%h/.local/share/curio

Nothing in the park hardcodes the port: boardwalk resolves names through `SC_ASSET_BASE`, so
repointing every venue at a different store — staging, a colleague's — is one variable.
