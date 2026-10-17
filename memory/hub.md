# The hub

*Updated 2026-10-17*

**Deferred: don't start it.** On 2026-10-16 Thomas chose **one owner per store**, relayed by
`@datadungeon` on 2026-10-17
(`mail/received/20261017-035543Z-@datadungeon--datadungeon-0.14-simplified-the-prompt-as-sent-and-faf305e4.md`).
The hub is deferred, and the plan below no longer stands as written:

- Each store belongs to one app. Sharing will mean copying records into your own store, marked
  `copied-from`.
- The shared R2 bucket is never deleted from, so the hub's clearing job is gone.
- Uses, owner tags, and refusing to delete what's in use are dropped from DD's plans, so there are
  no cross-store use counts to gather.

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
