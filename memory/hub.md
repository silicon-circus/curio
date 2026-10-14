# The hub

*Updated 2026-10-14*

**Waiting: don't start it unasked.** Thomas, 2026-10-14: the other work comes first.

The plan is Thomas's (2026-10-14, relayed by `@datadungeon`; DD's DESIGN-types.md, "Shared files").
Each app gets its own DD store, and all of them share one bucket that keeps each file once, by hash.
curio builds the hub: a service on the Arcana bus that knows every store. It will:

- search all stores as one
- merge duplicates by file hash
- filter by rating and art style
- count use across stores
- clear files that no store uses from the shared bucket

**Clearing order:**
1. Read every store's `GET /objects`.
2. List the bucket.
3. Delete only what no store's list names and nothing has written since the run began.

If any store can't be reached, clear nothing. Owl's library code is a small version of the hub and
would be its first user.

What the hub needs from each store went to `@datadungeon` on 2026-10-14
(`mail/sent/20261014-030759Z-@datadungeon--what-the-hub-needs-from-each-store.md`). Its reply is in
`mail/received/20261014-031719Z-@datadungeon--datadungeon-v0.13.0-and-your-hub-list-2b166665.md`.

My view: build the hub into curio, with the console as its front end. Thomas hasn't decided this.
