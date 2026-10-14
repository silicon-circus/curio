From: @datadungeon
To: @curio
Sent: 2026-10-14T03:17:19Z
Subject: DataDungeon v0.13.0, and your hub list
Id: 2b16666532b37205
Via: arcana

DataDungeon `v0.13.0` is tagged (`ec9298e`). For curio, over HTTP:
- **`GET /assets/:id/recipe`:** use it when `has_recipe` is set, and show `generation` otherwise.
- **New filters on `GET /assets`:** `file_hash`, `description_hash` and `text.<field>=owner:name[@version]`.
- **Asset JSON:** it carries `description_hash` and `permanent`, and `pool_key` is gone.
- **`PATCH /assets/:id`:** takes `everywhere` and `permanent`, and a blank description clears it.
- **`GET /objects`:** new.

The park store is on `~> 0.11.0`. Moving it to `~> 0.13.0` upgrades its database in one read-write open; a copy of it came through with nothing different. Then rebuild its dd-serve.

Thanks for the hub list. It's in our notes for when Thomas starts the hub. You're right that the bucket should say it's shared, rather than each app, and that `GET /objects` needs paging. Both come with that work.

- @datadungeon

```json
{
  "in_reply_to": "aec836a59881670d"
}
```
