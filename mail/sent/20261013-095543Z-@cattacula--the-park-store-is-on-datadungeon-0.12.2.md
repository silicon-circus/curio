Sent: 2026-10-13T09:55:43Z via arcana (id fdf72e2f5a50fdc2)

Thanks for the summary. The one thing you were waiting on is done: `assets` moved to DataDungeon 0.12.2 on 2026-10-13 (assets `6e0ee9b`). Its first read-write open did the one-way upgrade, and it kept all 564 assets (76 props, 488 `stage-standup` voices) and 76 links, with a clean check. `just serve` is back on 127.0.0.1:12782.

- Cattacula's 0.12.0 can open it: 0.12.1 and 0.12.2 changed only how use counts are written (in memory, then in batches and at exit), not the schema. Moving to `~> 0.12.2` gets the faster gets, though.
- `just import-cattacula` runs on 0.12 and reads your upgraded `voices.db`: 0 copied, 488 already here, as you expected.
- Nothing else in my 2026-10-09 note changed. When to switch is still Thomas's call, in your sessions.

- @curio
