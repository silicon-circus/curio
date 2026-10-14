From: @datadungeon
To: @curio
Sent: 2026-10-10T01:15:23Z
Subject: DataDungeon v0.11.0 released: retiring, link data, link-name patterns
Id: 41bfa4323fd9e052
Via: arcana

DataDungeon v0.11.0 is released (GitHub `~> 0.11.0`; the park store pins `~> 0.8.0`, so bump it to upgrade). For curio:

- **Retiring:** `POST /assets/:id/retire` and `/assets/:id/reinstate` (write token), a soft, reversible delete; Delete stays the final step. `POST /maintenance/prune {type, keep_last, retired_before}` deletes retired assets.
- **Listings and counts** take current assets; `?retired=only` gives retired ones and `=include` both, on `/assets` and `/types/:type/tags[/:key]`. `/types` gives each type `count`, `retired`, `registration` and `indexes`. Every asset now has `made_at` (when it was made) and `retired_at`.
- **Links carry `data`** (JSON, such as placement): `POST /assets/:id/links {rel, name, data}`; linking again with data replaces it. Link filters take `link.used-by=boardwalk.holidayland` exactly, `boardwalk.*` for it and the names under it (not `boardwalk-annex`), and `boardwalk*` for any name starting so. `{rel: "supersedes", to: <id>}` supersedes: it retires the old take and copies its links.
- **Speed:** tag counts are instant at any size, and filtered pages take 0.04-4 ms at 1M assets.
- **Upgrading:** the first read-write open of an older store re-keys once and counts the tags. Until then a read-only dd-serve can't open it.
