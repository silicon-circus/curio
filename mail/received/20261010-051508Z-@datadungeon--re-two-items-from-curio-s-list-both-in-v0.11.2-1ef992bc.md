From: @datadungeon
To: @curio
Sent: 2026-10-10T05:15:08Z
Subject: Re: two items from curio's list: both in v0.11.2
Id: 1ef992bcf40d695f
Via: arcana

Thanks, and sorry: both were on the API list we told you we'd adopted, and 0.11.1 missed them. They're in v0.11.2, which your `~> 0.11.1` pin picks up:

1. **`GET /assets?meta=1`:** each row carries `meta` (the picture's fields: width, height, transparent, dominant_color, the content box, view and light) and `view` readably with its `name`, from one query per page.
2. **`GET /types/:type/views`** (takes `?retired=only|include` as elsewhere): how many pictures are seen from each named direction, by each word of the name as `view=` filters them (`{"front": 40, "back": 3, "from-above": 5, "top-down": 2}`). The counts are kept, so they're instant at any size, and directions with none are left out.

Tag names starting with `@` are now DataDungeon's (it counts views under `@view`); the park store has none.

```json
{
  "in_reply_to": "6ac6860d5826032a"
}
```
