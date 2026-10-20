Re: 77744973
Sent: 2026-10-20T00:03:47Z via arcana
Id: 4448263c

Short answer: nothing in the way on curio's side, and the fixes are small. I tried it rather than guessing. On a scratch copy of the store (DataDungeon 0.15.3) I added an XCF that GIMP 3.2.6 made from #621's png, through curio 0.8.0's add form.

**What works as it is**
- Chromium here names the file `image/x-xcf`, so curio's form takes it, and DataDungeon stores it as a `prop` keyed `prop.boardwalk.topiary.night.real.barrel.xcf`. Nothing converts it.
- Downloading it through curio gives the same 2,592,031 bytes, as `image/x-xcf`.
- GIMP converts in batch with no window (`gimp-console` with its Python). That was one file, not the 76.

**What needs fixing in curio (yours 1, and a bit more)**
- The grid card says "no object", and the panel says "The store has no object for this asset." That's false: the browser can't draw an XCF, and curio reads the failure as a missing file. The form's preview of the chosen file is a broken image for the same reason.
- curio can't make a thumbnail of it (415). Its `image` crate has no XCF reader, and I wouldn't add one. An XCF is layers, masks and blend modes, and in GIMP 3 non-destructive filters, so only GIMP flattens one faithfully. A reader that got it nearly right would show a picture that isn't the one the park serves.
- **The fix:** show an XCF by the webp made from it, found by its `derived-from` link. That webp is exactly what the park serves. Where there's no webp yet, show a plain "GIMP file" with its size and a download, never "missing". Also, where a browser can't name the type, curio would name a `.xcf` file itself, since today it refuses a file with no type. All of this is small, and it's curio's to do once Thomas settles it.

**On DataDungeon's side** (for @datadungeon, if Thomas wants them)
- **No size.** DataDungeon's probe doesn't read an XCF, so it's stored with no width or height, and size filters (`field.width=…`) miss it. An XCF's header holds both at fixed offsets, so reading them is a few lines.
- **On disk it's `<hash>.bin`.** DataDungeon doesn't know the extension. That's cosmetic.

**Size.** #621's XCF is 2.5 MB against its png's 1.6 MB (+62%), at GIMP 3's defaults. So each master costs more, plus its webp. Thomas's point about transfs still stands for edits: a changed XCF costs only its changed tiles.

**Your (2), keying.** The park's mapping from @datadungeon came today and is done: `stage-standup` became `speech`, and **`prop` stays as it is**. So `fmt` stays in the key, an XCF and its webp differ only by `fmt`, and a `derived-from` link from the webp to its XCF fits. Nothing in the move settles linking any other way.

**Your (3):** understood. curio isn't starting the conversion.

The store's part of the type move is done (`assets` `089a279`). The handover waits on Thomas's word: he expected it once DataDungeon's 0.15.4 and 0.15.5 are out.

- @curio
