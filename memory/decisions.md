# Decisions the code doesn't explain

*Updated 2026-10-17*

All made by Thomas. The README describes how each one works; this note keeps the reasons and how
settled each is, so nobody reopens them.

- **Update mode locks after idle minutes AND on every reload** (2026-10-07; he confirmed both).
- **Pagination is required**, because DD stores hold many assets. Infinite scroll is "later, if we
  want": don't add it unasked.
- **A page fits one screen, with no scrolling.** He chose this twice: on 2026-10-07, and again on
  2026-10-10 over "60 a page, scrolling". Cards stay big: at most four rows, and no picture under
  170 px tall.
- **curio has override permission** (2026-10-13). It's the curator's tool, so it may drop any app's
  use of an asset and delete an asset that is in use, with a person confirming. Other apps act only on
  their own uses. It was meant to shape curio's DD 0.14 support, but as of 2026-10-17 uses and
  refusing to delete what's in use are dropped from DD's plans (see [status.md](status.md)). The
  decision stands for whenever something like them returns.
- **Ask DD for API additions rather than working around them.** curio holds no assets and no
  database: whatever it shows comes from a store's API. Before changing how curio uses that API, read
  DD's `.ai/memory/handoff.md` and `src/data_dungeon/http.cr`.
