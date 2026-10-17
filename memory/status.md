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

curio runs on DD 0.11.2 through 0.13. A 0.13 store is detected by `has_recipe` appearing on an
asset, and an older store simply doesn't show the 0.13 parts.

## Open

- **DD 0.13.1** (`4df221b`, 2026-10-14): a named text can now carry a purpose,
  `owner:name/purpose@version` (`overlore:silver-comic/figure@2`). The filter `text.<field>=owner:name`
  finds every purpose, and `owner:name/figure` finds one.
  - curio works unchanged: "any version" strips only `@N`, so it keeps the purpose.
  - **Don't build an "any purpose" button.** Purposes are deprecated (Thomas, 2026-10-17), and a
    later DD release removes them. They came in so that one style could be worded once for figures
    and once for places. Framing is now its own part of a prompt, so one style wording serves both.
- **DD 0.14 support, once DD tags it.** On 2026-10-17 `@datadungeon` wrote that 0.14 is rebuilt,
  simpler: committed as `4426581` on DD's `0.14` branch, not tagged yet. DD will write when it's
  tagged, and curio's pin (`~> 0.13.x`) holds until then. The message is
  `mail/received/20261017-035543Z-@datadungeon--datadungeon-0.14-simplified-the-prompt-as-sent-and-faf305e4.md`.
  - **A recipe is the prompt as sent.** There are no fields any more: `GET /assets/:id/recipe` gives
    `prompt`, `instructions` (speech and chat only) and `wordings: [{kind, name, text}]`. Each
    wording's words appear in the prompt or the instructions word for word.
  - **A version is a hash:** the first 10 hex characters of the words' SHA-256
    (`dd:silver-comic@12063ea05a`). `@2` gets a 400. DD's own owner is now `dd`, not `datadungeon`.
  - **What curio must change:**
    - `recipeView` and `stepView` (`public/console.html:1327-1370`) read `fields` and `prompt_sent`.
      Show the prompt and instructions instead, and mark each wording where it sits.
    - `named()` (`:1363`) strips the version with `/@\d+$/`, so a hex version loses its "any
      version" button.
    - The `text.<kind>=` filter and its proxy (`src/routes.rs:178`) work unchanged.
  - **Still in:** the vocabulary (14 art styles, the ratings), `GET /vocabulary` and `ViewWords`.
  - **Changed before the tag** (`791b22a`, 2026-10-17, Thomas: each app words its own styles;
    `mail/received/20261017-165106Z-@datadungeon--0.14-each-app-words-its-own-styles-dd3641d0.md`):
    - A picture's `style` wording may be the app's own (`owl:watercolor`), not only `dd:`. Pictures
      are still tagged with the vocabulary's art styles.
    - `flavor` is gone from the recommended order.
    - `GET /vocabulary` gives each art style's and rating's words as `wording: {kind, name, text}`.
      The first build called it `style`, but ratings have words now too (`dd:pg`), so the field
      covers both, and `kind` says which.
  - **Dropped:** uses, owner tags, and refusing to delete what's in use. MatchIndex, per-app indexes,
    views, light and keyframes are frozen at what exists.
  - The message doesn't say whether these are still coming: several values per tag, retyping, areas
    as fractions, and `plug`/`placard`. Check DD's changelog at the tag. The 2026-10-15 description
    of 0.14 it replaces never reached `@curio`.
- **Still DD's to add:** orders other than newest first, and range requests (seeking long media).
  They are listed in the README.
- **The hub is deferred** (Thomas, 2026-10-16, relayed by `@datadungeon`). See [hub.md](hub.md).
- The park store's own open items are in [park-store.md](park-store.md).
