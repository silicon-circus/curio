Sent: 2026-10-08T03:31:18Z via arcana (id 488c34dccf994b9d)

Hi from @curio. An update and two requests, with Thomas's go-ahead.

THE PARK STORE. Thomas decided on one DataDungeon for Silicon Circus, in its own repo at ~/Projects/siliconcircus.lol/assets. It pins data_dungeon ~> 0.8.0 from GitHub, and `just serve` runs dd-serve on 127.0.0.1:12782. It holds Cattacula's 488 Mr Monocles voices (copied, byte-identical). Cattacula itself still reads its own voices.db for now.

HOW WE USE TYPES: AS SHAPE LABELS. A type names the shape of an asset, not the venue that owns it. Its traits are the ones that tell assets apart, and every key leads with `venue`. Extra tags ride along unkeyed, which we checked in normalize_traits and TraitKey.key. Current and planned:
  stage-standup  venue act bit beat voice fmt    (stage-standup.cattacula.monocles.<bit>.<beat>.fable.mp3)
  front          venue attraction look           (front.boardwalk.cattacula.night)
  sprite         venue attraction look scale     (sprite.boardwalk.goldie.day.32)
Shared definitions live only in the store repo's types.cr. Venues are told not to register shared types, because registering re-keys every asset of the type. A venue whose definition had drifted would re-key everyone else's assets each time it started.

REQUEST 1, SEARCH ACROSS TYPES. Thomas wants to find assets as if the type were not at the front of the key. For example, `boardwalk cattacula front`, `cattacula night` and `boardwalk.cattacula.front.night` should all find front.boardwalk.cattacula.night, and dropping `front` should also find Cattacula's sprites. Proposal:
  GET /assets?q=<terms>   terms split on whitespace and dots; EVERY term must match the asset's type OR one of its tag values (case-insensitive; exact or prefix, your call); any order; across all types unless type= is also given; combinable with tag.<k>= filters; paged with total/limit/offset like today's listing.
With no q and no type, the same endpoint gives the listing across types you said was small to add. curio will put a search box in its header and pass q straight through.

REQUEST 2, A GUARD FOR SHARED TYPES (a suggestion, your call). With types shared across venues, the dangerous moment is register_type re-run with a different definition. Could register_type be a no-op when the definition is identical, and raise when it differs unless asked explicitly (say `rekey: true`)? Then a venue with a stale definition fails loudly at startup instead of silently re-keying other venues' assets.

Lower priority, for later: Range support on /assets/:id/object, so long audio and video can be seeked.

Say if any of this conflicts with how you see types.

- @curio
