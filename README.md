# curio

A web UI for DataDungeon stores. Filter a store's assets by tag, see what is
there, and curate it — **read-only until you turn on update mode**.

    just run                 # http://127.0.0.1:19463
    curio --stores           # does every configured store answer?

DataDungeon is the asset server, and it stays free of UI code. curio holds no assets and no database:
everything it shows comes from a store's JSON API, and everything it changes is a call to that API.
What it adds is what a browser cannot safely do on its own — hold the tokens, keep the write switch
honest, and make thumbnails, so a page of pictures is not a page of masters.

*Until 0.3.2 curio was the asset server itself, serving the park's files by name. That program is
tagged `v0.3.2-server`.*

## Requirements

| | |
|---|---|
| **Rust** stable (1.90+) | `cargo build --release`; the only build step |
| a DataDungeon store, **0.9 or later** | reachable over its API: `dd-serve` beside a database, or an app that mounts `DataDungeon::API`. curio uses 0.9's search, links and paging |
| `just` *(optional)* | remembers the commands; nothing requires it |

## Configuring stores

curio reads `~/.config/curio/curio.toml` (or `--config PATH`, or `CURIO_CONFIG`):

```toml
[[store]]
name = "pirateship"
url = "http://127.0.0.1:7077"     # dd-serve, or an app's mounted API (a path like /dd is kept)
read_token = "…"                  # if the store asks for one
write_token = "…"                 # only for a store you mean to curate

[[store]]
name = "cattacula"
url = "https://cattacula.example/dd"
read_token = "…"
```

A list, because that is what DataDungeon is in production: each app's store lives on that app's
server. Optional settings at the top of the file: `port` (19463), `bind` (127.0.0.1), `idle_minutes`
(10) and `thumbs_max_mb` (1024).

**The tokens live in that file and nowhere else.** Not in the page, not in a URL, not in the browser's
storage — curio's server attaches them on the way through, sending the read token on reads and the
write token only on writes. `chmod 600` the file; curio says so on startup if you have not.

A store with no `write_token` is read-only in curio whatever the switch says, and the console says so
on every asset.

To serve a store for curio, run `dd-serve` beside its database (`shards build dd-serve` in
DataDungeon). Without `DD_WRITE_TOKEN` it opens the database read-only:

    DD_DATABASE=voices.db DD_CACHE_ROOT=voice-blobs DD_PORT=7077 dd-serve

## Update mode

curio starts read-only. The red switch in the corner turns update mode on, and then you can edit tags,
description, rating and flag, delete an asset, make a custom tag value standard, and clean up a store.

It turns itself off again:

- **after `idle_minutes` with no use.** Any request the console makes while it is on counts as use;
  leaving the tab alone does not. The countdown and the shrinking fuse under the switch show how long
  is left.
- **whenever the page is reloaded.** Turning the switch on hands the page a random key, which it keeps
  in a JavaScript variable and sends with every write. Nothing stores it, so a reload forgets it —
  nothing has to detect the reload.

The server is the authority: a page whose timer missed still has its write refused. Turning it on in a
second tab ends the first tab's key. While it is on and the store in view is writable, the big-top
stripes run across the top of the window — red means update mode, and nothing else in the console is
red.

A write needs three things, checked in this order, and each refusal says which was missing: a write
token for that store in curio's config (403), update mode on with this page's key (423), and
DataDungeon agreeing (its own answer).

## Thumbnails

DataDungeon streams an asset's bytes at full size and nothing else — making smaller copies is a
viewer's job, and DataDungeon stays lean. curio makes a webp of each picture once and keeps it in
`~/.cache/curio/thumbs` (`CURIO_CACHE`), with a size ceiling and oldest-first eviction.

**Keyed by the asset's `file_hash`**, which is the SHA-256 of its bytes: a hash always means the same
picture, in any store, so nothing is ever invalidated, and the browser may cache a thumbnail as
immutable. The hash arrives in the URL, so it is **checked, not trusted**: the bytes are hashed when
they arrive, and a thumbnail is kept only under the hash it actually has.

Scaling is premultiplied-alpha Mitchell — cut-out props have transparent edges that a naive resize
haloes — and pictures are never enlarged. SVG is shown as itself; audio, video and text get a player
or a preview in the detail panel instead of a thumbnail.

## Reading the console

- **Search** from the box in the header (or press `/`). Every word must be an asset's type or one of
  its tag values, in any order, so `cattacula front night`, `boardwalk cattacula` and
  `cattacula.front.night` all find `prop.cattacula.front.night.real..webp`; `catt*` matches the start
  of a word. A search starts across every type, and choosing a type then narrows it.
- **Types** across the top, with counts, come from the store, after **All**. No type or tag name is
  written into curio: every store has different ones, and they get renamed.
- **Use** at the top of the filters: what a venue uses (`used by boardwalk`), or what nothing uses,
  across every type. Who uses an asset is a *link* in DataDungeon, not a tag, because it changes and
  one asset can have several users; the detail panel lists an asset's links both ways, and in update
  mode adds them (`used-by` a name, or `#id` for another asset) and removes them.
- **Filters** on the left are the type's tag names; a registered type's key traits come first, in key
  order. The bar behind each value is its share of the type. Filtering on several tags means all of
  them, and every filter, search and use filter shows as a chip above the grid.
- **Pages** that each fit one screen, newest first: as many cards as the window holds, so the grid
  never scrolls. The total is asked for once per search rather than on every page, because counting
  reads every match. Resizing the window or opening the detail panel reshapes the page and keeps the
  selected asset in view. The page is in the URL, with the store, type, filters and the open asset,
  so a reload comes back to the same place — but never in update mode.
- **Page Up / Page Down** turn pages. **← →** step through assets with the detail panel open,
  turning the page at either end. **Esc** closes it.

## Who can reach it

**Loopback only, and every request must prove it came from curio's own page.** curio holds every
store's tokens, so a read is as privileged as a write: anything that can reach this port can read
every configured store. So every request needs a loopback `Host` — which defeats DNS rebinding — and
one that carries an `Origin` needs it to be the very origin it was sent to, which refuses a page from
any other site or any other local port. Comparing against the request's own `Host` is what lets the
console work through an SSH tunnel on a different local port. And a request a browser labels as coming
from another site (`Sec-Fetch-Site`) is refused even when it carries no `Origin`, so another page
cannot embed curio's URLs as images or audio to probe what a store holds.

    ssh -L 8080:127.0.0.1:19463 workstation      # then http://localhost:8080

Asset bytes are served with `Content-Security-Policy: sandbox`, because a store can hold HTML or SVG,
and either one opened as a document would otherwise run its script with curio's origin.

That is a defence against web pages, not against this machine: any local process can drive curio, the
same as it could drive `dd-serve`.

## Commands

    curio                    serve the console
    curio --stores           ask every configured store for its types, and say which answer
    curio --config PATH      use this config

    CURIO_CONFIG             the config (default ~/.config/curio/curio.toml)
    CURIO_PORT, CURIO_BIND   override the config's port and bind address
    CURIO_CACHE              thumbnails (default ~/.cache/curio/thumbs)
    CURIO_CACHE_MAX_MB       thumbnail cache ceiling (default 1024)

## The HTTP surface

Everything under `/api/stores/<name>/` is that store's DataDungeon API, passed through:

| | | |
|---|---|---|
| `GET` | `/` | the console, compiled into the binary |
| `GET` | `/health` | liveness |
| `GET` | `/api/stores` | the stores and whether each is writable — never a token |
| `GET` `POST` `DELETE` | `/api/update` | update mode: status, on (returns the key), off |
| `GET` | `…/types`, `…/types/:type/tags[/:key]` | types, tag names, a tag's values, with counts |
| `GET` | `…/assets?type=&q=&tag.<k>=&link.<rel>=&unlinked=&limit=&page=&count=1` | a page of assets |
| `GET` | `…/assets/:id`, `…/assets/:id/object` | an asset with its links; its bytes (`?download=1` to save) |
| `GET` | `…/assets/:id/links` | what it links to, and what links to it |
| `GET` | `…/assets/:id/thumb/:file_hash?w=` | a webp thumbnail, width rounded up to a multiple of 64 |
| `POST` | `…/check` | the store's consistency report |
| `PATCH` `DELETE` | `…/assets/:id` | edit or delete — update mode |
| `POST` `DELETE` | `…/assets/:id/links` | link (`{rel, name}` or `{rel, to}`) or unlink (`?rel=&name=`) — update mode |
| `POST` | `…/types/:type/promote`, `…/vacuum` | update mode |

## What DataDungeon does not offer yet

These are DataDungeon's to add, and curio will use them when it does:

- **orders other than newest first.**
- **range requests** on an object, so a long video or audio clip cannot be seeked until it has
  loaded.
- **editing a type's registration**, deliberately: apps keep registrations in memory.
