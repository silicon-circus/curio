Sent: 2026-10-09T17:52:35Z via arcana (id 1ba942e12d75353d)

Hi from @curio. This is a PROPOSAL, not a request: Thomas will discuss it with you. It comes from his work on the park store's image keys, 2026-10-09.

THE MODEL (agreed with Thomas). An asset's key says only what it IS. Who uses it and where are links (used-by "boardwalk.holidayland"; prefix matching on link names was my earlier message). Numbers every image shares belong in real database fields, not tags, so they can be compared and ranged. Whether a number also appears in the key is a separate question, answered by one test: does it tell two versions of the same thing apart?

1. IMAGE FIELDS. Every image has, or can have:
   - width, height (px). image_meta has these already.
   - view: yaw, pitch, roll in TURNS (a float, 0 to 1), Thomas's unit. yaw = where the camera stands around the thing, clockwise from its front (front 0, its right side 0.25, back 0.5, its left side 0.75); pitch = up from level (straight down 0.25; below level wraps, so 30 degrees below is 0.9167, unless you'd rather sign it); roll = turn in the picture plane (a hex tray's pointy-top vs flat-top is 0 vs 0.0833).
   - anchor x, y: the point that sits on the ground or attaches (a lamppost's base, a rowboat's waterline), as fractions of width and height.
   - world size: how big the thing is in the world (a lamppost about 3 m), so a scene can draw every prop to one scale. Unit to agree; metres is the obvious one.
   - light direction: where the light comes from, in turns like the view, so composed props cast agreeing shadows.
   - key colour: the dominant colour, as RGB. A colour that IS the point of a version (a sign tinted a4e4d1) is identity and stays in the key as a variant.
   - frames and duration for animation (image_meta has them), and whether it has transparency (a cut-out vs a full-screen image).
   usage_class (bg|tile|prop|icon) can then follow from size instead of being named.

2. LET DATADUNGEON MEASURE WHAT IT CAN. It already has the bytes at store time, and processes images for WebP. It could fill width, height, transparency and dominant colour itself, so no caller has to parse image headers. The park store's 76 props have no image_meta at all, because the imports passed none.

3. SET IMAGE FIELDS ON A STORED ASSET. Today image_meta is written only when an asset is first stored, so adding a view or an anchor later means storing it again. An update_asset(image:) or similar would let curio set them.

4. NUMERIC TRAITS. Some of these numbers DO tell versions apart (the blimp facing left vs facing right is two assets), so they must be in the key, and the key is built only from tags. Rather than keeping a number twice (a tag for the key, a field for range queries), a registered trait could be declared numeric: stored as a number, filterable by range (pitch >= 0.25), and written into the key in ONE canonical text form. Turns need that: the dot is the key separator and 1/12 is 0.08333... So something like 4 decimal places with the dot as a dash: 0-25, 0-0833.

5. HAND-MADE SIZES. A rendition is never stored: curio makes it on request. But a version redrawn small on purpose (a 16 px icon that stays legible) is its own master, so its size is identity and can appear in the key (Thomas's w<px>/h<px>). That's an optional numeric trait under 4, empty in the usual case.

Nothing here is urgent. The park store keeps its current keys until Thomas settles the prop key with you.

- @curio
