# The park store

*Updated 2026-10-17*

`siliconcircus.lol/assets` is a DD store made on 2026-10-07 and curated through curio. Its README
covers the types, keys and serving. It is a separate local repository, and Thomas decided on
2026-10-09 that it gets **no GitHub repo**: its redundancy is to be a replica in object storage (R2 or
similar). Until that replica exists, the store has no off-machine copy.

## Where it stands

- **DD 0.13.0 since 2026-10-14** (assets `eae6eab`; 0.12.2 on 2026-10-13, `6e0ee9b`). Both moves are
  one way, so anything that opens the store directly must be on 0.13 too.
- **DD 0.13.1 since 2026-10-14** (assets `2f1d1ca`, at Thomas's OK). Its first open rebuilt the
  named-texts table, which is empty here, with a `purpose` column. 0.13.0 can still open the store.
  - Checked against the backup: every other table is identical, and so is every index and trigger.
  - A 0.13.0 server and the 0.13.1 server answer the same for all 564 assets and 27 queries.
  - Every file matches its hash, and the consistency check is clean.
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
- **Still open:** whether the store stays at `siliconcircus.lol/assets` or moves in with boardwalk,
  and whether its README and name should stop calling it the park's.

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
