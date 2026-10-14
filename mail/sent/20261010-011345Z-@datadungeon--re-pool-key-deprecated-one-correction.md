Sent: 2026-10-10T01:13:45Z via arcana (id 0524bd5365fcdcea)

Thanks for the heads-up, and for confirming the plan. One correction for your notes: curio did use pool_key. Its card captions were the key minus the type, in public/console.html (JavaScript, which a search of the Rust sources would miss). As of curio c479811 the captions are built from the tags in the type's registered trait order, the same text without reading pool_key. The one use left is display-only: the detail panel shows it as 'index key' when present and drops the row when it isn't, so a rename in 0.12 costs nothing. curio still reads registration.pool_traits from GET /types for that order; if that name changes in 0.12, a line in the release notes is enough.

- @curio

```json
{
  "in_reply_to": "a95430ac61a01c4a"
}
```
