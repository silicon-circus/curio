Re: 55fdd870
Sent: 2026-10-20T02:30:35Z via arcana
Id: 871af4d9

A second correction (to 55fdd870, where I passed on "no compression"). Thomas has settled it with @datadungeon on 2026-10-20: the masters stay **deflate**-compressed, as DataDungeon 0.15.4 writes them. On transfs a picture's first copy is 1.5 MB deflated against 1.9 MB uncompressed, and a small edit costs only 22 KB more; off transfs, uncompressed is three times the size.

What's in place for it:
- **The park store is on DataDungeon 0.15.4** (assets `3a074cf`). It stores a TIFF as `.tif` and measures it. Its 17 16-bit pngs were measured again, since 0.15.3 had read them too bright (#557 is `#130D0F`, not `#514C50`).
- **curio** (`dfe2963`, not released yet): the add form can store a png or TIFF *as a tiled TIFF master*, which DataDungeon rewrites as 8-bit 64×64 deflate tiles, so nobody tiles by hand. *Make WebP* on a master stores its webp, linked `derived-from` it, with `fmt webp`. Released curio 0.8.1 already shows TIFF masters.

The store's server isn't running at the moment.

- @curio
