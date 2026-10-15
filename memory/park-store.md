# The park store

*Updated 2026-10-14*

`siliconcircus.lol/assets` is a DD store made on 2026-10-07 and curated through curio. Its README
covers the types, keys and serving. It is a separate local repository, and Thomas decided on
2026-10-09 that it gets **no GitHub repo**: its redundancy is to be a replica in object storage (R2 or
similar). Until that replica exists, the store has no off-machine copy.

## Where it stands

- **DD 0.13.0 since 2026-10-14** (assets `eae6eab`; 0.12.2 on 2026-10-13, `6e0ee9b`). Both moves are
  one way, so anything that opens the store directly must be on 0.13 too.
- **DD 0.13.1 is out, and the store isn't on it yet.** On first open, 0.13.1 rebuilds the
  named-texts table once; it's empty in this store. `@datadungeon` upgraded a copy, and nothing else
  changed. **Open:** whether to move the store up waits for Thomas's OK.
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
  style values, and lighting or condition.
- **#584 and #585**, the piggybank sprite as `16-bit` and as `32-bit`, are the same file on purpose.
  Thomas says one picture works for both, and he'll probably tweak one so they differ.
  - That matters: since 0.11, DD has refused the same bytes twice in one type (`AlreadyStored`).
    The pair predates that rule, so copying the props into a fresh store would fail on the second.
  - Store a tweaked picture as a new take that `supersedes` the old one. The old one is retired, and
    its `used-by` link carries over.

## One store for the park: not settled

- On 2026-10-07 Thomas said to turn Cattacula's store into the park store. He was still deciding,
  and I over-read it as a decision.
- On 2026-10-14 he settled it (relayed by `@datadungeon`): **each app gets its own store**, all
  sharing one bucket, and curio builds the hub ([hub.md](hub.md)).
- His reasons:
  - a corrupted or lost store takes down only its own app;
  - a one-way upgrade no longer forces every app to move at once.

  Where boardwalk's store will live under this plan is still open.
- Cattacula's switch to this store is **on hold** (2026-10-13). `@cattacula` confirmed on
  2026-10-14: Cattacula keeps its own `voices.db`, and is on DD 0.13.0 (cattacula `55587c5`).

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
