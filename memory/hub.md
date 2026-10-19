# The hub

*Updated 2026-10-19*

**curio's next goal, once the boardwalk is on its own store** (Thomas, 2026-10-19): to be the central
hub through which DataDungeon stores share assets, and which does their garbage collection. Don't
start it before the handover in [park-store.md](park-store.md#handing-the-store-to-the-boardwalk) is
done. Its shape isn't decided.

- **One owner per store still stands** (Thomas, 2026-10-16). Sharing was to mean an app copying
  records into its own store, marked `copied-from`. Now the hub is to be the way stores share. The
  clipboard-store idea in status.md may be part of it.
- **Garbage collection is the hub's again.** On 2026-10-17 the shared bucket was never to be
  deleted from, so the hub's clearing job had gone. Thomas now names it as the hub's.
- **Also the hub's, from 2026-10-17: sharing styles between apps.** DD has no standard styles, so
  the hub would keep a lookup of the styles each store uses. One store could then adopt another's
  style, with its name and words.
- **To settle when it starts:** what sharing does (a copy made through the hub, or something the
  stores refer to), and what is collected (files that no store's records use, in a shared bucket or
  in each store's own). Ask `@datadungeon` what DD plans for copying between stores.

## Deferred from 2026-10-16 to 2026-10-19

On 2026-10-16 Thomas chose **one owner per store**, relayed by `@datadungeon` on 2026-10-17
(`mail/received/20261017-035543Z-@datadungeon--datadungeon-0.14-simplified-the-prompt-as-sent-and-faf305e4.md`),
and the hub was deferred:

- Sharing was to mean copying records into your own store, marked `copied-from`.
- The shared R2 bucket was never to be deleted from, so the hub's clearing job was gone.
- Uses, owner tags, and refusing to delete what's in use were dropped from DD's plans, so there are
  no cross-store use counts to gather. That still holds.

Before that (2026-10-14), Thomas had said the hub waits until the other work is done.

## The earlier plan (2026-10-14), kept for reference

Each app has its own DD store, and all of them share one bucket that keeps each file once, by hash.
curio was to build the hub, a service on the Arcana bus that knows every store. It would:

- search all stores as one;
- merge duplicates by file hash;
- filter by rating and art style;
- count use across stores;
- clear files that no store uses from the shared bucket.

What the hub needed from each store went to `@datadungeon` on 2026-10-14
(`mail/sent/20261014-030759Z-@datadungeon--what-the-hub-needs-from-each-store.md`), and its reply is
`mail/received/20261014-031719Z-@datadungeon--datadungeon-v0.13.0-and-your-hub-list-2b166665.md`.
Owl's library code, a small version of the hub, would have been its first user.
