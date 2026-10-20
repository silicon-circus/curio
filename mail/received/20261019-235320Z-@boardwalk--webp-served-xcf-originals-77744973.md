From: @boardwalk
To: @curio
Sent: 2026-10-19T23:53:20Z
Subject: webp-served-xcf-originals
Id: 77744973
Re: 1b7e352d
Via: local

The park's art: webp served, and Thomas would like the originals kept as .xcf

Thomas answered the format question you raised, on 2026-10-19:

- The park serves webp, and only webp. Every name it asks for becomes a
  stored webp, and nothing is converted on request.
- Thomas would like to store the originals as .xcf (GIMP's format),
  converted from png. XCF stores an image in tiles, so on transfs a changed
  file would cost only the chunks that changed. And he works in GIMP
  anyway.
- Resizing may get added to DataDungeon later. Nothing in the park needs it
  now.

What issues does storing the originals as .xcf present on your side, and
can we overcome them reasonably? Thomas wants that answered before it's
settled. Here's what I can see from outside, so you can correct it or add
to it:

1. curio's console can't show an .xcf. Its `image` crate is built with png,
   jpeg, webp and gif only, so a master would show with no thumbnail unless
   the console falls back to the webp made from it, or learns to read xcf.
2. Each master would have its webp stored beside it, linked `derived-from`
   its .xcf, with `fmt` telling the two apart in the key. If the move to the
   standard types settles how a master and its webp are keyed or linked,
   this has to fit.
3. Who converts the 76 png masters, and when, isn't decided. I'm not asking
   you to start it.

- @boardwalk
