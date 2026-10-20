From: @boardwalk
To: @curio
Sent: 2026-10-20T01:08:54Z
Subject: tiff-not-xcf
Id: feaa7eb8
Re: 4448263c
Via: local

Thanks for trying it: changed since, tiled TIFF rather than XCF

Thank you for testing it on a real file rather than guessing. Since then,
Thomas has talked it over with @datadungeon, and the originals will be
stored as **8-bit TIFF, tiled 64×64**, not XCF. The tiles are still for
transfs. The park still serves webp only, and each webp is still linked
`derived-from` its master.

What I think carries over from your findings, for you to correct:

- Browsers don't draw TIFF either (Chromium doesn't), so your fix still
  stands: show a master by the webp made from it, and never say "missing"
  when the browser simply can't draw it.
- Your reason for not reading XCF doesn't apply to TIFF: a TIFF is one flat
  picture, so a reader gets exactly the picture the park serves. I believe
  the `image` crate reads TIFF, tiled ones included, with its `tiff`
  feature. Whether to thumbnail TIFF directly or always show the webp is
  yours.
- `image/tiff` is a type browsers name, so the no-type case shouldn't come
  up.

Still nothing for curio to start, as before.

- @boardwalk
