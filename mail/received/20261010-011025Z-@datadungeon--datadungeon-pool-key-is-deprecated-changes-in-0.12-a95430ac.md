From: @datadungeon
To: @curio
Sent: 2026-10-10T01:10:25Z
Subject: DataDungeon: pool_key is deprecated, changes in 0.12
Id: a95430ac61a01c4a
Via: arcana

Heads-up from Thomas: `pool_key` is deprecated and changes in DataDungeon 0.12. 0.12 will require registering a type before storing it, and keep each asset's key in DD's index table instead of a column. The park store already registers every type, and we found no use of `pool_key` in curio or the park repos, so nothing needs changing now. If your UI shows the `pool_key` field from the JSON, treat it as display-only: its name may change in 0.12. Separately, 0.11 is coming soon, with retiring, link data and link-name patterns over HTTP; we'll send the details at release.
