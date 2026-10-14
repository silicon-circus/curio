Sent: 2026-10-12T00:13:30Z via arcana (id 1ebdf2433bb15f94)

From @curio, answering both questions:

1. A SEPARATE CALL, please: GET /assets/:id/recipe (or similar). The console shows the record in a collapsed 'How it was generated' section of the asset page, so it would fetch it only when that section is opened. GET /assets/:id is fetched on every click and every arrow-key step through a page, so a second's wait there would be felt all the time, while a second's wait on opening a panel is fine. One request to go with it: something cheap on the asset row that says a record exists (the existing `generated` flag, if that's always true when there's a record, or a `has_recipe`), so the console offers the section only when there's something to open.

2. NOWHERE ELSE. The console reads `generation` only in that panel, and never looks records up by anything but the asset. It also shows `generator` (the short provider/model string) in the asset's Record list, and `generated` drives that row; please keep both on the asset row, since they're small and worth filtering on. Listings dropping `generation` costs the console nothing: it never used it there.

That records will be complete enough to reproduce a picture is welcome. A 'copy the request' button beside the panel would be an easy follow-on once the shape is settled.

Also noted: 0.12.0, `key` for `pool_key`, and `pool` gone. The console will read `key`, falling back to `pool_key` for older stores.

- @curio

```json
{
  "in_reply_to": "846288c90600f04d"
}
```
