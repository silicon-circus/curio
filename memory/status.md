# Status

*Updated 2026-10-19*

curio is the web UI for DataDungeon (DD) stores. Thomas decided this on 2026-10-07: DD is the asset
server and stays UI-free. The old server, which served the park's files by name, is tagged
`v0.3.2-server`.

## Now: the boardwalk onto its own store, then the hub (Thomas, 2026-10-19)

- **The boardwalk gets its own DataDungeon,** as Cattacula and Pirateship have theirs. curio has
  looked after the park store only because curio grew out of the archive (later "assets") server
  the boardwalk depended on, and still does.
- **That handover is curio's whole job for now.** Thomas told `@siliconcircus` the same day that
  `assets/` stays curio's until curio says it's done, and that the boardwalk will then mount DD's
  handler and serve its own art.
- **curio keeps the store until it's on DD's standard types** (Thomas, 2026-10-19), since curio was
  in on that decision. He expects it with DD 0.15.4 and 0.15.5, within hours. Then it goes to the
  boardwalk, and the remaining kinks are worked out with him. See
  [park-store.md](park-store.md#handing-the-store-to-the-boardwalk).
- **The park store is on DD's standard types** (assets `089a279`, 2026-10-19), so curio's part is
  done. The handover waits on Thomas's word.
- **The park's masters are eight-bit TIFFs in 64×64 tiles** (Thomas with `@datadungeon`,
  2026-10-19), not `.xcf`, and not compressed (Deflate at most), for transfs's chunking. curio
  shows them since `9268c5d` (not released). See
  [park-store.md](park-store.md#keeping-masters-as-tiff-thomas-2026-10-19).
- **There is no deployment** (Thomas, 2026-10-19): the old curio online is only for testing.
- **curio is to leave the `silicon-circus` org:** Thomas to `@siliconcircus`, 2026-10-19, "it is now
  a general tool for use with datadungeon". He hasn't said it to curio yet.
- **Then curio's goal is the hub,** through which DD stores share assets, and which does their
  garbage collection. See [hub.md](hub.md).

## Releases

| Version | Date | Commit | What |
|---|---|---|---|
| 0.4.0 | 2026-10-09 | `510fadd` | the DD web UI |
| 0.5.0 | 2026-10-13 | `b7e8924` | runs on DD 0.12.x: the key from 0.12, falling back to `pool_key` |
| 0.6.0 | 2026-10-14 | `accd565` | DD 0.13: the "How it was made" recipe panel, finding by a shared file, description or named wording, changing a shared description everywhere, Permanent |
| 0.7.0 | 2026-10-17 | `ac96b8b` | DD 0.14: the recipe panel shows the prompt and instructions as sent, with each named wording marked; "any version" takes hash versions; a store with many types scrolls the type bar instead of widening the page |
| 0.8.0 | 2026-10-19 | `f5ed8fb` | adding a file by hand (DD 0.15.1's `POST /assets`): a form in the detail panel, a dropped file, *New take…* |
| 0.8.1 | 2026-10-19 | `dd6b739` | TIFF masters show as pictures: curio thumbnails them (`9268c5d`), as DD 0.15.4 stores and measures them |

curio runs on DD 0.11.2 through 0.15. A 0.13 store is detected by `has_recipe` appearing on an
asset, and an older store simply doesn't show the 0.13 parts. A 0.14 recipe is told from a 0.13 one
by its `wordings` list.

## Open

- **Adding a file by hand: released in 0.8.0** (`4358d05`, released as `f5ed8fb`, tag `v0.8.0`, pushed
  2026-10-19). Thomas's curio has run 0.8.0 since then; the form was checked on it, with nothing
  added. Thomas wanted
  pictures added by hand, with no prompt, and a button in curio for it (relayed by `@datadungeon`;
  `mail/received/20261019-055743Z-@datadungeon--datadungeon-v0.15.1-an-upload-route-for-curio-s-bu-35bdbf0d.md`).
  He OK'd the plan the same day.
  - **What's built:** *+ Add a file* above the grid, or a file dropped anywhere on the page, opens a
    form in the detail panel. *New take…* on an asset adds a file in its place. curio's
    `POST …/assets` passes the form through with the write token, only in update mode, up to 256 MB.
  - **Tried on a copy of the park store**, served by 0.15.3: an add, measured and with no recipe; a
    file the type holds (409, with a button to the holder); the same file and tags again ("already
    there"); a new take (old retired, its link copied, `supersedes` recorded); a drop with the switch
    on and off. Two new tests cover the route (34 in all).
  - **DD 0.15.1** (`f9c4bc6`) adds only `POST /assets`, so `~> 0.15.0` takes it. The park store has
    it since 2026-10-19 (0.15.3, see park-store.md).
  - The request is `multipart/form-data` with the write token. `file` holds the bytes, and its
    `Content-Type` is the asset's. `asset` is JSON: `type`, `tags`, `custom`, `description`,
    `permanent`, `made_at` and `supersedes`.
  - The asset is stored as not generated, with no recipe, and a picture is measured. The answer is
    `{asset, tags}`, as for `PATCH`.
  - Bytes the type already holds get a 409 with `asset_id` and `retired`. The same bytes with the
    same tags return that asset.
  - `description` says what the picture shows. It isn't a prompt.
  - `supersedes` makes *New take…* the way to do the piggybank tweak (see park-store.md).
- **DD 0.15.2 and 0.15.3 are tagged** (2026-10-19). No message reached curio about them; seen in
  the tags when the park store was rebuilt. Both are additive and neither converts a store one way.
  - **0.15.2:** DD's standard asset types, in `GET /vocabulary` (`types`), and a shared `subtype`
    tag for an app's own kinds of a type.
  - **0.15.3:** a take's tracks, what a tool measured of a sound or video over time
    (`GET`/`PUT /assets/:id/tracks/:name`, `tracks` in `/vocabulary`), kept in a new `asset_tracks`
    table. curio shows neither yet.
- **DD 0.15.0 is tagged** (`58f1fa6`, 2026-10-17;
  `mail/received/20261017-224318Z-@datadungeon--datadungeon-v0.15.0-is-tagged-25613558.md`). It's a
  small cleanup, and its first read-write open converts a store one way, as before.
  - Wording names are `owner:name[@version]`. The purpose is gone, and a name with a `/` is refused.
  - `recipe_fields` is dropped. 0.14 had kept 0.13's prompt fields read-only.
  - A store from before 0.12 isn't converted any more: open it read-write with 0.14 first.
  - **curio needs no change.** It never wrote or showed purposes. Thomas's curio 0.7.0 reads the
    park store on 0.15 (all 564 assets, since 2026-10-17).
  - The park store moved to 0.15 on 2026-10-17, at Thomas's word. See [park-store.md](park-store.md).
- **DD 0.14.0 is tagged** (`401dd4a`, 2026-10-17;
  `mail/received/20261017-185402Z-@datadungeon--datadungeon-v0.14.0-is-tagged-948f3178.md`).
  - **A recipe is the prompt as sent.** `GET /assets/:id/recipe` gives `prompt`, `instructions`
    (speech and chat only) and `wordings: [{kind, name, text}]`, and leaves out a missing prompt. The
    fields, `joined` and `prompt_sent` are gone. Each wording's words are in the prompt or the
    instructions word for word, though recipes stored before 0.14 aren't checked.
  - **A version is a hash:** the first 10 hex characters of the words' SHA-256
    (`overlore:silver-comic/figure@860f00fedb`). `@2` gets a 400. DD's own owner is `dd`.
  - **No standard art styles** (Thomas, 2026-10-17). Each app names, words and tags its own
    (`overlore:silver-comic`, kind `style`). Sharing a style between apps falls to the hub, later.
  - **The vocabulary keeps only the ratings,** each with DD's words (`dd:pg`). `GET /vocabulary`
    gives them as `wording: {kind, name, text}`, named `wording` because it isn't only for styles.
  - **A wording's purpose** (`owner:name/purpose`) is deprecated (Thomas, 2026-10-17), and 0.15
    removes it. Don't build an "any purpose" button.
  - **Dropped:** uses, owner tags, and refusing to delete what's in use. MatchIndex, per-app indexes,
    views, light and keyframes are frozen at what exists. Not in 0.14 either: several values per tag,
    retyping, areas as fractions, and `plug`/`placard`.
  - The messages before the tag are in `mail/received/` from 2026-10-17.
- **curio's 0.14 support shipped in 0.7.0** (`7c5a761`, released as `ac96b8b`, tag `v0.7.0`, pushed
  2026-10-17).
  - Tested on scratch copies of Overlore's store (916 recipes), one converted by 0.14 and one left on
    0.13.1. The cases: a picture with a style wording, speech with instructions, the wording finds,
    and made-up recipes with repeated, missing and instruction-side wordings. The 32 tests pass.
  - **Fixed in `97b8874`:** a store with many types (Overlore has 15) made the type bar wider than
    the window, so the page scrolled sideways and the detail panel went off-screen. This had been
    so since before 0.14. The bar now scrolls on its own and keeps the chosen type in sight.
  - Thomas's curio has run 0.7.0 since 2026-10-17.
- **Masters in curio, from DD 0.15.4: done in `dfe2963`, not pushed or released** (Thomas said yes
  2026-10-19). The add form offers *Store it: as a tiled TIFF master* for a png or TIFF (DD's
  `normalize`/`tiff`), and a tag naming a png's format follows the choice (`fmt` png ↔ tif) in view.
  *Make WebP* on a png or TIFF asks for its webp (quality 85, a format tag saying `webp`) and opens
  it; one already made is found, not made twice (DD answers a repeat with the existing webp, not a
  409). A store on an older DD says so. Tried on two copies of the park store, served by 0.15.4 and
  0.15.3. Release waits on Thomas.
- **TIFF masters in curio: released in 0.8.1** (`dd6b739`, tag `v0.8.1`, pushed 2026-10-19), held
  until DD 0.15.4 at Thomas's word and tried on a copy of the park store served by it first (see
  park-store.md). Thomas's curio runs 0.8.1. curio decodes a TIFF (uncompressed, Deflate or LZW; not
  ZSTD) for its thumbnails, so a master shows in the card and the panel. Tried on a scratch copy of
  the park store. 35 tests. Replaces the XCF fix, which isn't needed.
- **Still DD's to add:** orders other than newest first, and range requests (seeking long media).
  They are listed in the README.
- **Idea, not decided: a curio store as a clipboard** (Thomas, 2026-10-17). curio would get a DD store
  of its own as a holding place: boardwalk's art waiting on vocabulary could wait there, and assets
  would pass through it between apps' stores, marked `copied-from`. Points raised:
  - Things have to leave it, or it becomes a store no app owns. No app should use anything from it.
  - Types must match the destination's, or keys change on paste.
  - curio can't add assets yet. dd-serve takes uploads from 0.15.1 (`POST /assets`, below), so the
    add button would be the first piece.
  - Ask `@datadungeon` first whether copying between stores is planned. Hold that mail for Thomas.
  - Since 2026-10-19 sharing is to go through the hub, so this may become part of it.
- **The hub is next, after the boardwalk handover** (Thomas, 2026-10-19). It had been deferred since
  2026-10-16. See [hub.md](hub.md).
- The park store's own open items are in [park-store.md](park-store.md).
