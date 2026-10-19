# The park store

*Updated 2026-10-19*

`siliconcircus.lol/assets` is a DD store made on 2026-10-07 and curated through curio. Its README
covers the types, keys and serving. It is a separate local repository, and Thomas decided on
2026-10-09 that it gets **no GitHub repo**: its redundancy is to be a replica in object storage (R2 or
similar). Until that replica exists, the store has no off-machine copy.

## Where it stands

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
anywhere, and `assets/` stays curio's until curio says it's done. The mount waits on that. The
order is Thomas's to set.

- **Done:** the rebuild on DD 0.15.3 (assets `b66492d`). The repo is clean, and curio has nothing
  in progress in it (2026-10-19).
- **Open, waiting:** importing the rest of the boardwalk's art. It waits on the park's type mapping
  from `@datadungeon`. Every store is moving onto DD's standard types (`#datadungeon` post 1,
  2026-10-19: Pirateship done, then Cattacula, the park and Owl), and each gets a note with its
  mapping. Mapping names by hand before then is work against a moving target.
  - **Who imports is Thomas's to say:** curio when the note comes, or the boardwalk with the store.
    Asked him 2026-10-19.
- **curio's old files stay put** until the boardwalk's art is out of them: `curio/data/assets`
  (1052 files, named in `data/manifest.tsv`), and their backup.
- **Handed over in the reply** (`mail/sent/20261019-231442Z-@boardwalk--re-boardwalk-and-its-store.md`):
  - this note, for how each move was done and checked, and the small open items;
  - dd-serve: once the handler is mounted, the park doesn't need it. curio's console still reads
    the store through it, with the write token.
- **Thomas's to decide:** clearing the old backups and binaries (curio's leftovers), and retiring the
  deployed old server once nothing points at it.
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
- **The format is the gap, not the width.** The park never asks `?w=`, and its two `w640` names are
  files stored under those names. But of the 48 names in the boardwalk's `src/assets.json`, only 12
  are stored under the name asked for. The other 36 are `.png` masters that the old server answered
  as `.webp`, converting on the fly (`src/serve.rs`, `src/derive.rs` at `v0.3.2-server`).
  - DD's handler serves stored bytes and doesn't convert. Its shard has a png-to-webp encoder
    (`webp.cr`), but nothing calls it on request.
  - The props in the store are the masters too: #621's `curio-name` is
    `boardwalk.topiary.barrel.night.real.png`, and the park asks for the `.webp`.
  - Ways offered, none decided: store the webp the park draws, made at import; draw the png; or
    convert in the boardwalk, cached by file hash.
- **Where the 48 are:** 15 have their master in the store, 33 only in curio's old files, and none
  is missing from both (by name without width or format, 2026-10-19). The registry's other names
  weren't checked. `@siliconcircus` counted 32 of all 102 in `maps/boardwalk.tsv`.
- `@boardwalk` and `@siliconcircus` say **curio is leaving the `silicon-circus` org.** Thomas's plan
  for curio (the hub) fits that, but he hasn't said anything to curio about the repository moving.

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
- Kitty Marionette meows rather than speaks, so she needs her own `stage-…` type. It isn't designed
  yet.
- Pirateship's `i` (a line's index in its pool) should be renamed when Thomas gets to it; `line` was
  suggested.
- The gangway (#587) has a made-up real height of 3.2 m from a test. Thomas was told to correct it.
