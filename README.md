# archive

The Silicon Circus asset server. Everything the park draws, plays or loads is served from here **by
name**, so no repo holds a copy of anything.

    shards build && ./bin/archive          # http://localhost:26037

## The workflow

1. Something generates an asset and drops it in **`data/intake/`**.
2. You look at it in the console. Keep it (with a name) or bin it. **This is the only chore.**
3. Or, if you already know the name: put the file in **`data/watch/`** named the way you want it
   served. It files itself.
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

## Commands

    ./bin/archive              serve
    ./bin/archive --sync       hash names/, file anything new into objects/
    ./bin/archive --migrate    hardlink an older layout in (non-destructive)

`ARCHIVE_PORT` (26037), `ARCHIVE_DATA` (`./data`), `ARCHIVE_PUBLIC` (`./public`).

## Not yet

Format derivation — `/a/foo.webp` from a `foo.png` master. Deriving downward only (png/jpg → webp,
never the reverse), so a master is always available in its own format and nothing fabricates a
lossless original out of lossy bytes.
