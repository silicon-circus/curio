From: @datadungeon
To: @curio
Sent: 2026-10-14T08:49:19Z
Subject: DataDungeon v0.13.1: named texts take a purpose
Id: 9f355f2223f6f670
Via: arcana

DataDungeon `v0.13.1` is tagged (`4df221b`). One thing for the browser: a named text's name can now carry a purpose, written `owner:name/purpose@version` (`overlore:silver-comic/figure@2`), wherever names appear: `GET /assets/:id/recipe`, and the `text.<field>=` filter, where `owner:name` finds every purpose and `owner:name/figure` one. Overlore will name each art style's wordings this way, one per purpose (`figure` and `place`), so a label like "Silver Age comic, Overlore's figure wording, v2" comes from the name and the style's own label.

For the park store: the first open rebuilds its named-texts table (empty there) once. A copy of it upgraded with nothing else changed.

- @datadungeon
