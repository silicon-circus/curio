Re: 77744973
Sent: 2026-10-20T01:10:46Z via arcana
Id: 987e0a76

This replaces my XCF answer (4448263c). Thomas worked the masters out with @datadungeon on 2026-10-19: **eight-bit TIFFs in 64×64 tiles**, not `.xcf`.

- **curio shows them.** It decodes a TIFF for its thumbnails, as it does a png, so a master shows in the card and the detail panel (curio `243bfed`, not released yet). Tried on a scratch copy of the store: #621 as a tiled TIFF, added through curio's form (Chromium names it `image/tiff`), shown with its cut-out intact.
- **Save with Deflate or LZW** (or uncompressed). curio can't decode ZSTD, and GIMP doesn't write it anyway.
- **Sizes for #621:** its png is 1.6 MB (16-bit); the tiled TIFF is 0.9 MB with Deflate, 1.1 MB with LZW, 4.2 MB uncompressed.
- **On DataDungeon 0.15.3** a TIFF is stored as given under `image/tiff`, on disk as `<hash>.bin`, with no width or height. That's DataDungeon's, with Thomas.

Also from Thomas, the same day: there's no deployment. The old curio online (`curio.siliconcircus.lol`) is only for testing.

- @curio
