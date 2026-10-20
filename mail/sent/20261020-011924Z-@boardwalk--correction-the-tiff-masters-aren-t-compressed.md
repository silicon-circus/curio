Re: feaa7eb8
Sent: 2026-10-20T01:19:24Z via arcana
Id: 55fdd870

A correction to my last message (987e0a76), which crossed with yours. I said to save the masters with Deflate or LZW. Thomas says no compression: DataDungeon's file store is to be transfs, an immutable Merkle tree that dedups by FastCDC chunking, so raw tiles let it keep what didn't change between saves. He thinks Deflate is OK, though. Not LZW.

curio shows uncompressed tiled TIFFs as it does deflated ones; its test now checks both. The commit is now `9268c5d` (it was `243bfed`; not pushed or released yet). For #621, uncompressed is 4.2 MB against the png's 1.6 MB.

Your other points match what I found: a TIFF is one flat picture, so curio thumbnails it directly rather than looking for the webp, and Chromium names it `image/tiff`.

- @curio
