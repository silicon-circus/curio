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
| a DataDungeon store, **0.11.2 or later** | reachable over its API: `dd-serve` beside a database, or an app that mounts `DataDungeon::API`. curio uses its search, links, paging, retiring and picture fields; from **0.13**, recipes, shared descriptions and `permanent`, which an older store simply doesn't show |
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
description, rating, flag and whether it's permanent (never evicted for lack of use), retire or delete
an asset, make a custom tag value standard, and clean up a store.

**A description can be shared.** A room's description is one text, used by every picture of the room.
Editing it changes this asset's words alone, unless you tick *Change it on all N assets that share
it*; the old words are kept, since a recipe may record them as what a provider was sent. Saving an
empty description clears it.

**Retire is the everyday way to get rid of something; Delete is the final one.** A retired asset is
hidden from listings, counts and search, and kept: *Show: retired* in the filters finds it again, and
*Reinstate* brings it back. A newer take that supersedes an old one retires it the same way. *Prune
retired…* in Store health deletes a type's retired assets for good.

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
- **Use** at the top of the filters: what a venue uses (`used by boardwalk`, or `boardwalk.*` for it
  and every area under it, such as `boardwalk.holidayland`), or what nothing uses, across every type;
  and whether to show current assets, retired ones, or both. Who uses an asset is a *link* in DataDungeon, not a tag, because it changes and
  one asset can have several users; the detail panel lists an asset's links both ways, and in update
  mode adds them (`used-by` a name, or `#id` for another asset) and removes them.
- **Picture** in the detail panel: what DataDungeon measured (size, the box around the visible
  pixels, transparency, dominant colour) and what was set (view, light, anchor, real size). Views are
  shown by name (`back-left from-above`) and in quarter turns. In update mode they can be set or
  corrected: a view or light from a preset, then fine-tuned; the anchor *at its foot* (the bottom centre
  of the visible pixels); the real width, height and depth in metres. *Seen* in the filters picks
  pictures by view, across every type, and offers only the directions something is seen from, with
  counts; *Measure pictures* in Store health fills in any not yet measured. On the cards, a cut-out
  sits on the checkerboard and any other picture on its own dominant colour, which is also what shows
  while its thumbnail loads; a picture with a view is labelled with it.
- **How it was made** in the detail panel: the recipe DataDungeon keeps (0.13), in the shape every
  app shares. It shows the settings, the cost and where that figure came from, and the prompt by
  field, in the order the fields are joined. If the provider was sent something other than the fields
  joined, what it was sent is shown with the difference marked; that's a camera adding words of its
  own. A named wording (`overlore:silver@2`) lists every asset that used exactly it, and *any
  version* every asset that used the name. *Made from* lists the files it was made from; each opens
  the asset holding it, or says no asset in this store does. An asset made in steps shows each step.
  Assets stored before recipes show their old record as it was written. *Shared by N assets* under a
  description lists them. Each of these is a chip above the grid, across every type, and in the URL.
- **Filters** on the left are the type's tag names; a registered type's key traits come first, in key
  order. The bar behind each value is its share of the type. Filtering on several tags means all of
  them, and every filter, search and use filter shows as a chip above the grid.
- **Pages** that each fit one screen, newest first, so the grid never scrolls: at most four rows of
  cards big enough to see (no picture under 170 px tall), and as many columns as fit. A big screen
  gets bigger cards, not more of them — 7 × 4 at 2560 × 1440. The total is asked for once per search rather than on every page, because counting
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
| `GET` | `…/assets?type=&q=&tag.<k>=&link.<rel>=&unlinked=&retired=&view=&field.<f>=&file_hash=&description_hash=&text.<field>=&limit=&page=&count=1` | a page of assets |
| `GET` | `…/assets/:id`, `…/assets/:id/object` | an asset with its links; its bytes (`?download=1` to save) |
| `GET` | `…/assets/:id/recipe` | how it was made; 404 when it has no recipe |
| `GET` | `…/assets/:id/links` | what it links to, and what links to it |
| `GET` | `…/assets/:id/thumb/:file_hash?w=` | a webp thumbnail, width rounded up to a multiple of 64 |
| `POST` | `…/check` | the store's consistency report |
| `PATCH` `DELETE` | `…/assets/:id` | edit (`{tags, description, everywhere, rating, flagged, permanent, image}`) or delete — update mode |
| `POST` `DELETE` | `…/assets/:id/links` | link (`{rel, name}` or `{rel, to}`, with optional `data`) or unlink (`?rel=&name=`) — update mode |
| `POST` | `…/assets/:id/retire`, `…/assets/:id/reinstate` | hide and keep, or bring back — update mode |
| `POST` | `…/prune` | `{type, keep_last, retired_before}`: delete retired assets for good — update mode |
| `POST` | `…/measure` | `{type, all}`: have the store measure its pictures — update mode |
| `POST` | `…/types/:type/promote`, `…/vacuum` | update mode |

## What DataDungeon does not offer yet

These are DataDungeon's to add, and curio will use them when it does:

- **orders other than newest first.**
- **range requests** on an object, so a long video or audio clip cannot be seeked until it has
  loaded.
- **editing a type's registration**, deliberately: apps keep registrations in memory.
