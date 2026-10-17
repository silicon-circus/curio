# Status

*Updated 2026-10-17*

curio is the web UI for DataDungeon (DD) stores. Thomas decided this on 2026-10-07: DD is the asset
server and stays UI-free. The old server, which served the park's files by name, is tagged
`v0.3.2-server`.

## Releases

| Version | Date | Commit | What |
|---|---|---|---|
| 0.4.0 | 2026-10-09 | `510fadd` | the DD web UI |
| 0.5.0 | 2026-10-13 | `b7e8924` | runs on DD 0.12.x: the key from 0.12, falling back to `pool_key` |
| 0.6.0 | 2026-10-14 | `accd565` | DD 0.13: the "How it was made" recipe panel, finding by a shared file, description or named wording, changing a shared description everywhere, Permanent |
| 0.7.0 | 2026-10-17 | `ac96b8b` | DD 0.14: the recipe panel shows the prompt and instructions as sent, with each named wording marked; "any version" takes hash versions; a store with many types scrolls the type bar instead of widening the page |

curio runs on DD 0.11.2 through 0.14. A 0.13 store is detected by `has_recipe` appearing on an
asset, and an older store simply doesn't show the 0.13 parts. A 0.14 recipe is told from a 0.13 one
by its `wordings` list.

## Open

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
  - **A wording's purpose** (`owner:name/purpose`) is deprecated (Thomas, 2026-10-17), and a later
    release removes it. Don't build an "any purpose" button.
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
- **Still DD's to add:** orders other than newest first, and range requests (seeking long media).
  They are listed in the README.
- **The hub is deferred** (Thomas, 2026-10-16, relayed by `@datadungeon`). See [hub.md](hub.md).
- The park store's own open items are in [park-store.md](park-store.md).
