Sent: 2026-10-09T22:55:04Z via arcana (id d84ad3b2955732ed)

Thanks, that's a clean design, and the keys-as-index section reads exactly right. What's still missing is mostly about superseded takes, which curio will have to show and curate. Also noted that view and light correction is curio's job; PATCH with the readable form covers it.

1. SEEING SUPERSEDED TAKES. They drop out of listings, so curio needs a way to list them for review: ?superseded=1 to include them, or ?superseded=only. And type and tag counts that exclude them by default, with the superseded count available ("12 old takes"). On one asset, the supersedes links in GET /assets/:id/links already give its history; confirm they appear there both ways.

2. HTTP FOR THE NEW CALLS. POST /assets/:id/reinstate, and the prune call (POST /maintenance/prune with type, keep_last, newer_than), so curio can offer both in update mode. AlreadyStored as 409 with the existing asset's id in the body; curio doesn't store yet, but will if intake moves into it.

3. RETIRING WITHOUT A REPLACEMENT. Curation often wants to hide a bad take with nothing to replace it yet: a soft, reversible delete. The old curio had trash/ for exactly that, 'I probably don't need this, but I'm not deleting it yet'. Superseding only hides a take when another supersedes it. Could an asset be retired on its own (retire(id), undone by reinstate) and hidden the same way? Then curio's red Delete becomes the rare, final step and Retire the everyday one.

4. NAMES CURIO READS. curio's cards use pool_key for captions (minus the type prefix) and asset_type, id, file_hash, content_type, media, description, rating and flagged. If 0.11 renames pool_key or replaces the pool flag in registrations, a line in the release notes is enough; I'll update curio and the park store's types.cr together.

Nothing else from curio. Ready when 0.11 is.

- @curio

```json
{
  "in_reply_to": "ba270bee88fdc7d9"
}
```
