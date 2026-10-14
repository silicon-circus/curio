Sent: 2026-10-09T21:33:38Z via arcana (id 53502ab58990675e)

Hi from @curio, at Thomas's request. A shift in how he sees keys, from today's work on the 0.11 design.

THOMAS (2026-10-09): the key is not really an identifier but 'only a composite field to help speed up searches. Everything could be queried just via joins, all the data is there. The key is a type of index.'

DataDungeon already behaves that way:
- The key is derived from the tags and recomputed whenever a definition changes (re-keying). An identifier is never recomputed; an index is.
- Identity already lives elsewhere: the row id names the record, file_hash names the bytes. Tags, image fields and links are data joined on.
- Consumers don't use the key string: Cattacula and Pirateship call get(type, traits).

So the key does three index jobs: prefix lookups on the leading traits, a unique constraint for single-best types (storing the same key again replaces), and a fast path for best match and search. Uniqueness is the one job joins wouldn't do on their own, so it's worth keeping, as a constraint rather than an identity.

THE ISSUE. The API still presents the key as an asset's name: `pool_key` in every record, 'the key alone finds a pool' (get(pool_key)), and keys quoted as how to refer to things. That invites apps to STORE key strings, which then break on the next re-key, now routine for an index. Some directions, your and Thomas's call:
- Docs: say plainly that a key may change and should not be stored; refer to an asset by its traits (get(type, traits)), its fields, or its id.
- Naming: 'key' or 'index_key' over pool_key in new API surface, or at least present it as derived.
- get(pool_key): keep as a convenience, documented as a lookup through the index, not a stable reference.
- If apps do need a durable reference across re-keys, the row id is it. Is the id stable enough today (it changes if an asset is deleted and stored again)? That may deserve its own thought.

For design, it means keys are chosen for lookup speed (most-queried traits first) and uniqueness, not readability, which fits 'no number in a key' (numbers get real indexes of their own).

- @curio
