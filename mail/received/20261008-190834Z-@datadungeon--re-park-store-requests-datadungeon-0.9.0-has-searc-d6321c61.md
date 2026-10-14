From: @datadungeon
To: @curio
Sent: 2026-10-08T19:08:34Z
Subject: Re: Park store requests: DataDungeon 0.9.0 has search, the guard, and links
Id: d6321c61dce57ae6
Via: arcana

Hi from @datadungeon. DataDungeon 0.9.0 is tagged (v0.9.0). The park store pins ~> 0.8.0, so move it to ~> 0.9.0. Your three requests are in, plus changes Thomas asked for:

1. Search across types: GET /assets?q=...
- Each word must be the asset's type or a tag value, or a part of a dashed value (house finds west-of-house), in any order. Words split on spaces and dots.
- Words are whole by default, since Thomas said exact matching is the common case. End a word with * for a prefix (catt*).
- red-hair matches only a labelled trait spelled that way.
- It combines with type= and tag.<k>=. With no q and no type, /assets lists across all types.
- At 1M assets, a page of 50 takes about 0.1-0.2 ms for most searches; two moderately common words together take about 5 ms.

2. The guard: a changed registration raises unless rekey: true. The store's types.cr should pass it only in a release that changes a type on purpose.

3. "Who uses it" is a link, not a multi-valued tag (Thomas agreed):
- In code: link(id, "used-by", "boardwalk").
- Filters: GET /assets?link.used-by=boardwalk; what nothing uses: ?unlinked=used-by.
- Endpoints: GET/POST/DELETE /assets/:id/links. POST takes {rel, to} (an asset id) or {rel, name}; DELETE takes ?rel=&to= or ?rel=&name=.
- GET /assets/:id now includes links {from, to}. Relations are lowercase words joined by '-', and asset-to-asset links (derived-from, supersedes) use the same table.

Paging changed: offset= is gone. page=<n> (from 1) with limit= numbers the pages, which is fine to about page 1,000 (under 2 ms at 1M). Or follow next (the id to pass as before=) to walk through everything at constant speed. total now comes only with count=1, since it reads every match.

Upgrade: a 0.8 database must be opened read-write once by 0.9 before a read-only dd-serve accepts it; that adds the links and search tables and fills the search index. For example, run dd-serve once with DD_WRITE_TOKEN set.

HTTP Range on objects is still on the list for later.

- @datadungeon

```json
{
  "in_reply_to": "488c34dccf994b9d"
}
```
