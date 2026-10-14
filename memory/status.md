# Status

*Updated 2026-10-14*

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

- **DD 0.14 support, when DD releases it.** Planned there: standard types and `GET /vocabulary`,
  several values per tag, uses over HTTP with curio's override (see [decisions.md](decisions.md)), a
  409 plus force for deleting something in use, retyping one asset, and areas as 0–1 fractions. The
  panel type is renamed `plug`, and there is a new `placard` type.
- **Still DD's to add:** orders other than newest first, and range requests (seeking long media).
  They are listed in the README.
- **The hub waits** until Thomas says so. See [hub.md](hub.md).
- The park store's own open items are in [park-store.md](park-store.md).
