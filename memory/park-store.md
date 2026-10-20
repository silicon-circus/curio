# The park store

*Updated 2026-10-20*

`siliconcircus.lol/assets` is a DD store made on 2026-10-07 and curated through curio. Its README
covers the types, keys and serving. It is a separate local repository, and Thomas decided on
2026-10-09 that it gets **no GitHub repo**: its redundancy is to be a replica in object storage (R2 or
similar). Until that replica exists, the store has no off-machine copy.

## Where it stands

- **DD 0.15.4 since 2026-10-19** (assets `3a074cf`, at Thomas's word): TIFF masters, and the 16-bit
  pngs measured again. Done as rehearsed (below, under TIFF): backed up to `data/before-0.15.4/`
  (database and files), `bin/dd-serve-0.15.3` kept, `dd-serve` rebuilt (it links libtiff now, noted in
  the README). `POST /maintenance/measure {"all": true}` changed the 17 16-bit pngs' measurements
  only, the same values as the rehearsal, and added 76 change-log rows. No schema change, integrity
  and foreign keys pass, files untouched, scripts compile. The server was stopped afterwards.
- **On DD's standard types since 2026-10-19** (assets `089a279`): `stage-standup` became `speech`,
  piece 2 of DD's move (`mail/received/20261019-235136Z-@datadungeon--piece-2-of-datadungeon-s-asset-types-the-park-stor-aa12e52b.md`).
  `prop` was standard already and stays as it is. `types.cr` registers `speech` and keeps the
  `rename_type` call, which writes nothing once done. The import reads Cattacula's `voice` until
  Cattacula has `speech`.
  - Rehearsed on a copy, then live, backed up to `data/before-speech/`. Every row the same apart from
    the type, the keys (only their index id), the search rows (`stage`, `standup` gone) and 488
    change-log rows.
  - Through dd-serve, all 564 assets, links and recipes and 14 queries answer as the backup did, and
    all 488 lines match their hashes byte for byte. Integrity is ok, the files are untouched, and a
    second `just types` changes nothing. Reported to `@datadungeon`.
- **DD 0.15.3 since 2026-10-19** (assets `b66492d`, at Thomas's OK to rebuild for the upload route).
  `~> 0.15.0` took 0.15.3 rather than 0.15.1. Neither 0.15.2 nor 0.15.3 converts a store one way:
  the first open added the empty `asset_tracks` table and the `subtype` vocabulary tag, and 0.15.0
  still opens the store. Rehearsed on a copy first, and checked against a snapshot from before: all
  564 assets (1128 responses) and 27 queries the same, a clean check, the scripts compile.
  `bin/dd-serve-0.15.0` is kept.
- **DD 0.15.0 from 2026-10-17** (assets `bad3c6a`, at Thomas's word). It's one way again: 0.14 can't
  open the store, so anything that opens it directly must be on 0.15 too. The store had no purposes
  and no prompt fields, so the first open only dropped the empty `recipe_fields` table (and its two
  indexes) and the wordings' `purpose` column. It was rehearsed on a copy first, and the live result
  has the same schema.
  - Checked against the backup served by 0.14.0: all 564 assets (1128 responses), 27 queries and the
    vocabulary answer the same, and the store's scripts compile unchanged.
  - Every other table holds the same rows. Integrity and foreign keys are ok, all 563 files match
    their hashes and are the backup's files, `/objects` matches the disk, and the store's check is
    clean.
- **DD 0.14.0 from 2026-10-17, for a few hours** (assets `2c5fedd`). The store has no recipes, so it
  needed only the version bump and a `dd-serve` rebuild. It was rehearsed on a copy first, and the
  live result is the same, schema and all.
  - Checked against the backup served by 0.13.1: all 564 assets (1128 responses) and 27 queries
    answer the same, and the store's scripts compile unchanged.
  - Every table holds the same rows. The changes are the ones 0.14 means to make:
    - `named_texts` and `texts` gain DD's 4 rating wordings (`dd:g`, `dd:pg`, `dd:r`, `dd:x`);
    - the vocabulary tables are new (1 tag, 4 values), and so is `recipe_wordings` (empty);
    - `named_texts.field` becomes `kind`, and `recipes` swaps `prompt_sent_id` for `prompt_id`
      and `instructions_id`, with no rows either way;
    - the new tables bring one index and two triggers that refuse changing or deleting a wording.
  - Every other index and trigger is identical, and integrity and foreign keys are ok.
  - All 563 files match their hashes and are the same files as the backup. `/objects` matches the
    disk, and the store's check is clean.
- **DD 0.13.1 from 2026-10-14 to 2026-10-17** (assets `2f1d1ca`). Its first open rebuilt the
  named-texts table, which is empty here, with a `purpose` column. 0.13.0 could still open it.
  - Checked against the backup: every other table is identical, and so is every index and trigger.
  - A 0.13.0 server and the 0.13.1 server answer the same for all 564 assets and 27 queries.
  - Every file matches its hash, and the consistency check is clean.
- **Earlier moves:** 0.12.2 on 2026-10-13 (`6e0ee9b`), and 0.13.0 on 2026-10-14 (`eae6eab`).
- **The 0.13 move was checked closely**, at Thomas's request. Against the backup:
  - every table is identical row for row, and the dropped `description` and `generation` columns
    were empty;
  - every index and trigger was kept;
  - a 0.12.2 server and a 0.13 server answer the same for all 564 assets and 27 queries;
  - all 563 objects match their hashes;
  - every write path works on a scratch copy.
- **It holds 564 assets:**
  - 488 Mr Monocles voices, copied from Cattacula's own store (`stage-standup`);
  - 76 boardwalk props (`prop`), each linked `used-by` boardwalk.

  The rest of boardwalk's art isn't imported yet. It waits on Thomas's vocabulary decisions: art
  style values, and lighting or condition. Since 2026-10-17 DD has no standard art styles, so the
  style values are boardwalk's own to name (see [Art style](#art-style)).
- **#584 and #585**, the piggybank sprite as `16-bit` and as `32-bit`, are the same file on purpose.
  Thomas says one picture works for both, and he'll probably tweak one so they differ.
  - That matters: since 0.11, DD has refused the same bytes twice in one type (`AlreadyStored`).
    The pair predates that rule, so copying the props into a fresh store would fail on the second.
  - Store a tweaked picture as a new take that `supersedes` the old one. The old one is retired, and
    its `used-by` link carries over.

## Whose store: boardwalk's (Thomas, 2026-10-17)

- **This is boardwalk's store,** separate from Cattacula's. Thomas decided on 2026-10-17, after
  choosing **one owner per store** on 2026-10-16 (relayed by `@datadungeon`). Under that rule each
  store is one app's, and sharing means copying records into your own store (`copied-from`).
- **Cattacula's 488 voices stay here for now, as a backup of sorts.** Thomas doesn't want to risk
  losing them, so don't remove them unless he asks. Cattacula's own `voices.db` remains its store.
  - The copies are a snapshot from when they were copied, not kept in step with Cattacula.
  - They sit on the same machine as Cattacula's own store, with no off-machine copy until the
    replica exists.
- Cattacula's switch to this store was put on hold on 2026-10-13, and this decision ends it.
  `@cattacula` confirmed on 2026-10-14 that Cattacula keeps its own `voices.db`, on DD 0.13.0
  (cattacula `55587c5`).
- **How it got here:**
  - On 2026-10-07 Thomas said to turn Cattacula's store into the park store. He was still deciding,
    and I over-read it as a decision.
  - On 2026-10-14 he chose a store per app. His reasons: a corrupted or lost store takes down only its
    own app, and a one-way upgrade no longer forces every app to move at once.
- **Where the store lives, and its README** (which still calls it the park's), are the boardwalk's
  to settle with Thomas.

## Handing the store to the boardwalk

**Thomas, 2026-10-19:** the boardwalk gets its own DataDungeon, as Cattacula and Pirateship have
theirs. curio has looked after this store only because curio grew out of the archive (later
"assets") server the boardwalk depended on, and still does. Getting the boardwalk onto it is curio's
whole job for now; then curio is free of it (see status.md).

Thomas also decided, the same day (told to `@siliconcircus`, not to curio): **the boardwalk mounts
DD's HTTP handler in its own server and serves its own art**, instead of pointing `SC_ASSET_BASE`
anywhere, and `assets/` stays curio's until curio says it's done. The mount waits on that.

**The order (Thomas, 2026-10-19):** curio keeps the store until it has moved onto DD's standard
types, since curio was in on that decision. He expected that with DD 0.15.4 and 0.15.5, within hours.
Then it goes to the boardwalk, and the remaining kinks (the mount, the webp question, the rest of
the art, where the store lives) are worked out with him. `@boardwalk`, `@siliconcircus` and
`@datadungeon` were told, and asked `@datadungeon` to send the park's mapping note to `@curio`.
curio subscribed to `#datadungeon` the same day.

- **Done:** the rebuild on DD 0.15.3 (assets `b66492d`). The repo is clean, and curio has nothing
  in progress in it (2026-10-19).
- **Done 2026-10-19:** the move onto DD's standard types (see Where it stands). **The handover
  now waits on Thomas's word.** He expected it once DD 0.15.4 and 0.15.5 are out, and they are
  (`#datadungeon` posts 3 to 5, through 0.15.6 on 2026-10-20).
- **0.15.5 and 0.15.6 change nothing here** (checked 2026-10-20). 0.15.5 clears a clip's
  `transcript_id` when its transcript is deleted, and none of the park's clips has one. 0.15.6
  renames the standard type `map` to `pass`, and the park has only `prop` (76) and `speech` (488).
  The store's `dd-serve` is still 0.15.4. Rebuilding is optional and needs Thomas's word, as every
  move has.
  - The rule for a recording's words in 0.15.5 (`@datadungeon`'s proposal, which Thomas agreed to):
    before it is made, the prompt in its recipe; after, its `words` track; a `transcript` asset
    only "for words that are a thing of their own". **Open (Thomas, 2026-10-20):** he doubts that
    a `words` track is the same thing as a transcript, and finds the "of their own" wording
    misleading. At his request curio told `@datadungeon` why, and suggested a wording
    (`mail/sent/20261020-150536Z-@datadungeon--thomas-doubts-that-a-words-track-can-stand-in-for.md`). The park's 488 speech copies
    have no recipe and no tracks, so they follow whatever Cattacula does with hers.
- **Open, after the handover:** importing the rest of the boardwalk's art. It waits on the types. Every store is moving onto DD's standard types (`#datadungeon` post 1,
  2026-10-19: Pirateship done, then Cattacula, the park and Owl), and each gets a note with its
  mapping. Mapping names by hand before then is work against a moving target.
  - Who imports isn't settled. Thomas listed it among the kinks to work out after the handover.
- **curio's old files stay put** until the boardwalk's art is out of them: `curio/data/assets`
  (1052 files, named in `data/manifest.tsv`), and their backup.
- **Handed over in the reply** (`mail/sent/20261019-231442Z-@boardwalk--re-boardwalk-and-its-store.md`):
  - this note, for how each move was done and checked, and the small open items;
  - dd-serve: once the handler is mounted, the park doesn't need it. curio's console still reads
    the store through it, with the write token.
- **Thomas's to decide:** clearing the old backups and binaries (curio's leftovers), and retiring the
  old test server online once nothing points at it.
- `@siliconcircus` was told the same (`mail/sent/20261019-231449Z-@siliconcircus--curio-and-the-boardwalk-s-store-where-things-stand.md`). Its notes had
  Cattacula's voices as "in the wrong store", waiting on who removes them; curio passed on Thomas's
  2026-10-17 word that they stay.

## How the park gets its art (2026-10-19)

`@boardwalk` asked how the park would draw from this store instead of curio's `/a/`
(`mail/received/20261019-224126Z-@boardwalk--boardwalk-and-its-store-b37442ef.md`). What curio found:
- **No `/a/` here since curio 0.4.0** (2026-10-09). The boardwalk's default `SC_ASSET_BASE` is
  `http://localhost:19463/a/`, which this curio answers with a 404, so the park in development has
  had no art from curio since then. `@boardwalk` believed otherwise.
- **The deployed `curio.siliconcircus.lol` is the old server, curio 0.2.2**, serving 102 assets. It
  still answers `/a/boardwalk.deck.w640.webp` with a webp (checked 2026-10-19).
  `deploy/env.sample` points there. Production boardwalk is on 0.2.3 and draws bundled art.
- **Thomas settled the format (relayed by `@boardwalk`, 2026-10-19):** the park serves webp only,
  and every name it asks for becomes a stored webp; nothing converts on request. Resizing may come to
  DD later. **The originals are eight-bit TIFFs in 64×64 tiles** (Thomas, worked out with
  `@datadungeon`, 2026-10-19), not GIMP's `.xcf` as first asked (below).
- **The format was the gap, not the width.** The park never asks `?w=`, and its two `w640` names are
  files stored under those names. But of the 48 names in the boardwalk's `src/assets.json`, only 12
  are stored under the name asked for. The other 36 are `.png` masters that the old server answered
  as `.webp`, converting on the fly (`src/serve.rs`, `src/derive.rs` at `v0.3.2-server`). The park
  asks only for webp (47 names, and one mp3), so those 36 webps never existed as files.
  - DD's handler serves stored bytes and doesn't convert. Its shard has a png-to-webp encoder
    (`webp.cr`), but nothing calls it on request.
  - The props in the store are the masters too: #621's `curio-name` is
    `boardwalk.topiary.barrel.night.real.png`, and the park asks for the `.webp`.
  - Ways offered, none decided: store the webp the park draws, made at import; draw the png; or
    convert in the boardwalk, cached by file hash.
- **Where the 48 are:** 15 have their master in the store, 33 only in curio's old files, and none
  is missing from both (by name without width or format, 2026-10-19). The registry's other names
  weren't checked. `@siliconcircus` counted 32 of all 102 in `maps/boardwalk.tsv`.
- **curio is leaving the `silicon-circus` org.** Thomas told `@siliconcircus` on 2026-10-19 that
  curio "will be moving out of silicon-circus org any way. it is now a general tool for use with
  datadungeon." He hasn't said it to curio; ask him.
- **There is no deployment** (Thomas, 2026-10-19): the old site online, `curio.siliconcircus.lol`
  (curio 0.2.2, the park's 102 names), is only for testing. `@siliconcircus` had said production
  draws from it and that retiring it would leave every venue on its 8-bit fallback; that holds only
  for the test site. Retiring it is still Thomas's call. `park.sh doctor` shows the local curio as
  serving when `/a/` 404s, since it probes `/health`. That's the boardwalk's to fix.

## Keeping masters as TIFF (Thomas, 2026-10-19)

Thomas worked it out with `@datadungeon`: **eight-bit TIFFs in 64×64 tiles**, instead of `.xcf`.
**Deflate-compressed, as DD 0.15.4 writes them** (Thomas, settled 2026-10-20, relayed by
`@datadungeon`: `mail/received/20261020-023012Z-@datadungeon--tiff-masters-stay-deflate-compressed-thomas-2026-1-31b4371e.md`).
DD's file store is to be transfs, an immutable Merkle tree that dedups by FastCDC chunking. On
2026-10-19 Thomas had said uncompressed for its sake, with Deflate maybe OK; DD's numbers settled it:
on transfs a picture's first copy is 1.5 MB deflated against 1.9 MB uncompressed, and a small edit
costs only 22 KB more; off transfs, uncompressed is three times the size. curio's form option
(`normalize`/`tiff`) gives these masters; an uncompressed TIFF uploaded without it is kept as it is.
The park still serves webp only, each webp linked `derived-from` its master (`@boardwalk`,
`mail/received/20261020-010854Z-@boardwalk--tiff-not-xcf-feaa7eb8.md`, which crossed with curio's
answer `mail/sent/20261020-011046Z-@boardwalk--masters-as-tiled-tiff-not-xcf-curio-shows-them.md`).
- **curio shows them** (`9268c5d`, first committed as `243bfed`; not released yet): it decodes a
  TIFF as it does a png, so the card and the panel show its webp. The add form says the browser can't show it, instead of a broken
  picture. Chromium names a `.tif` `image/tiff`. A test checks that tiled TIFFs, uncompressed and
  deflated, render byte for byte as their pngs do.
- **Tried on a scratch copy** (served by 0.15.3, test curio 0.8.0 plus the change): #621 made into a
  tiled TIFF by libvips and added through the form, then shown in the card and panel, cut-out intact.
  Deflate, LZW and uncompressed all decode to the same thumbnail, 1024 wide in under 0.1 s. **ZSTD
  doesn't decode** (the `tiff` crate leaves it out by default); GIMP doesn't write it anyway.
- **Size of #621:** its png 1.6 MB (16-bit); as a tiled TIFF, 4.2 MB uncompressed, 0.9 MB with
  Deflate, 1.1 MB with LZW. Eight bits drops the depth of the 16-bit masters (about a third of the
  boardwalk's pngs). Thomas chose eight bits.
- **DD's side on 0.15.3:** stored as `image/tiff`, unconverted, on disk as `<hash>.bin`, with no width,
  height or colour (its probe doesn't read TIFF). **Closed in DD 0.15.4** (`a229a4a`, 2026-10-20,
  `#datadungeon` post 3, and
  `mail/received/20261020-012414Z-@datadungeon--re-tiled-tiff-masters-0.15.4-closes-both-gaps-the-f42b5aea.md`):
  - A `dd-serve` built from 0.15.4 needs libtiff (`require "data_dungeon/tiff"`). It stores a TIFF as
    `.tif` and measures it like a png.
  - `"image": {"compression": "normalize", "target_format": "tiff"}` in an upload's asset JSON makes
    DD rewrite a TIFF or png as 8-bit 64×64 deflate tiles, so nobody tiles by hand. Deflated, which
    Thomas says is OK. DD measured on transfs: a small stroke over 37 pictures costs 1.9 MiB
    uncompressed against 2.6 deflated, but the first copy 70 MiB against 57.
  - `POST /assets/:id/webp {"tags": {"fmt": "webp"}, "quality": 85}` stores a master's webp as its
    own asset, linked `derived-from` it, with its description and fields.
  - `.xcf` is stored as `.xcf`, unread.
- **0.15.4 tried on a copy** (2026-10-19, `dd-serve` built from 0.15.4 in a copy of the repo, which
  links libtiff 4.7.2; only `shard.lock` changes). Opening it changes no schema. Measuring again
  changed 17 pictures, exactly the 17 16-bit pngs, and only `dominant_color` (17), `content_h` (3)
  and `content_y` (2); #557 went `#514C50` → `#130D0F`. An uncompressed tiled TIFF of #621 added
  through curio's form is stored as `.tif`, measured the same as #621 itself, and shown. #621's png
  uploaded with `normalize`/`tiff` came back 8-bit, 64×64 tiles, deflated (0.9 MB). The webp route
  made #622's webp, linked `derived-from` it, and curio shows the link. The live store moved the
  same day (see Where it stands).
- **The props need measuring again on 0.15.4.** Before it, DD read a 16-bit png with no gamma chunk
  as linear light (too bright), and interlaced ones wrongly besides. 17 of the store's 27 pngs are
  16-bit; #557, Cattacula's night front, is recorded as `#514C50` but is `#130D0F`. On 0.15.4,
  `POST /maintenance/measure {"all": true}` measures every picture again, changing only measured
  fields.
- Who converts the 76 png masters, and when, isn't decided. curio isn't to start it.

## Keeping masters as XCF: tried 2026-10-19, not chosen

`@boardwalk` asked what `.xcf` masters would mean for curio
(`mail/received/20261019-235320Z-@boardwalk--webp-served-xcf-originals-77744973.md`). It was tried on a
scratch copy: GIMP 3.2.6 made an XCF from #621's png in batch (`gimp-console` with its Python), and it
was added through curio 0.8.0's form. Answered in
`mail/sent/20261020-000347Z-@boardwalk--re-webp-served-xcf-originals-tried-it-nothing-in-t.md`.
- **What works:** Chromium names it `image/x-xcf`. DD stores it as a `prop` keyed
  `….barrel.xcf`, unconverted, and it downloads intact.
- **What's wrong in curio:** the card says "no object", and the panel says the store has none, because
  the browser can't draw an XCF. The form's preview is broken. There's no thumbnail (415): the
  `image` crate has no XCF reader. Don't add one, since only GIMP flattens layers, masks and filters
  faithfully.
  - The fix would have been to show an XCF by its webp (found by `derived-from`). Not needed: TIFF
    was chosen instead (above).
- **DD's side:** no width or height (its probe doesn't read XCF, though the header holds both), and it
  sits on disk as `<hash>.bin`.
- **Size:** 2.5 MB as XCF against 1.6 MB as png (+62%) at GIMP 3's defaults.

## Settled ways of keying (Thomas, 2026-10-08 and 2026-10-09)

- **Types are named for the kind of asset, not the venue.** Every key starts with `venue`.
- **Shared definitions live only in the store's `types.cr`.** A venue never registers a shared type,
  because registering re-keys every asset of that type.
- **One image type, `prop`.** A front is a big prop, and a sprite is essentially a prop.
- **A key says what a thing is, never who uses it.** Who uses an asset is a `used-by` link, which
  also names the area (`boardwalk.holidayland`).
- **The key is an index, not an identifier.** Identity is the row id, and the bytes are the
  `file_hash`. Nothing should depend on how a key is spelled.
- **No number ever goes in a key.** Views, sizes, anchor, light and real size are picture fields.
- **Stand-up has a key of its own** (Thomas, 2026-10-08): `bit beat` are "specific to stand-up
  comedic acts", whose sets are bits of beats, so the voices had their own type, `stage-standup`,
  keyed `venue act bit beat voice fmt`. Since 2026-10-19 the type is DD's standard `speech`, and the
  structure lives in the park's key for `speech`, unchanged, not in the type's name.
  - **Open (Thomas, 2026-10-19):** he'll talk to Cattacula about stand-up. DD's `subtype` tag
    (0.15.2, one shared tag for an app's own kinds of a type, which can be in a key) is an option:
    stand-up and Kitty's meows as kinds of `speech`. Adding it to the key would re-key the 488.
- **Don't propose a direction word as a key part.** I suggested one and withdrew it: Thomas pointed
  out it's only a coarse form of the view vector.

## Art style

**The styles are boardwalk's own.** DD 0.14 has no standard art styles (Thomas, 2026-10-17): each app
names, words and tags its own, as `boardwalk:<style>` wordings of kind `style` with tag values of
its own. The props' own `style` values (`real`, `8-bit`, `16-bit`, `32-bit`) aren't touched.

Thomas, thinking aloud on 2026-10-13, not decided:
- `pixel32` might be made by pixelizing photoreal on the fly.
- `pixel8` might be made in the game, or kept as ASCII art.

Then only photoreal (or fantasy) and pixel would need storing.

## Smaller open items

- `goldie.webp` is for Thomas to check.
- Cattacula's `cat.shadow`, `night.lights` and `night.trans-stripes` are undecided.
- Kitty Marionette meows rather than speaks. In 2026-10-08's plan she needed her own `stage-…` type.
  Under DD's standard types a meow is `speech` ("words or not"), and 0.16 refuses unlisted type
  names, so her meows would be `speech` too, fitting its `bit beat` key or changing it. Thomas's to
  design, with the boardwalk once it's theirs.
- Pirateship's `i` (a line's index in its pool) should be renamed when Thomas gets to it; `line` was
  suggested.
- The gangway (#587) has a made-up real height of 3.2 m from a test. Thomas was told to correct it.
