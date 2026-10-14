Sent: 2026-10-09T16:01:44Z via local (a note left in Cattacula's .ai/inbox/, before dot-ai 0.3)

# Cattacula's voices now live in the park store too: what Cattacula needs to change

*From @curio — 2026-10-09*

Thomas decided (2026-10-07) on one DataDungeon store for the whole park, in its own repo beside this
one: `siliconcircus.lol/assets`. Cattacula's Mr Monocles voices have been **copied** into it. Nothing
in this repo was changed, and Cattacula still reads its own `service/data/voices.db` until someone
switches it over. Thomas asked for this note so that switch can happen here.

## What is in the park store

- 488 voices, byte-identical to `service/data/voices/` (checked by hash on 2026-10-07), as type
  **`stage-standup`**, keyed `venue act bit beat voice fmt`:
  `stage-standup.cattacula.monocles.<bit>.<beat>.fable.mp3`.
- `venue=cattacula act=monocles` on every one. Thomas: Cattacula has three acts. Mr Monocles is the
  speaking one. Kitty Marionette "has meows, not voice": that needs its own `stage-…` type, because
  `bit beat` is "specific to stand-up comedic acts". Talk to Thomas before designing it.
- No description on any of them. Those rows predate `voice_generate` storing the spoken line, so the
  text of each beat is only in the bit library.

## What Cattacula needs to change to use it

1. **Open the park store instead of `service/data/`:** `assets/data/assets.db` with
   `cache_root: assets/data/objects`. `DATA` is built from `__DIR__`, which is fixed when the code is
   compiled, so an env var for the store's path is the safer way to point at it.
2. **Stop calling `register_type("voice", …)`** (`service/src/cattacula.cr`, around line 584). Shared
   types are defined ONLY in `assets/types.cr`, because registering re-keys every asset of the type.
   Since DataDungeon 0.9, `register_type` raises if a definition differs from the stored one, unless
   given `rekey: true`, so a stale definition fails loudly at startup.
3. **Use `stage-standup` with the venue and act in the tags:** `voice_tags` becomes
   `{"venue" => "cattacula", "act" => "monocles", "bit" => …, "beat" => …, "voice" => VOICE, "fmt" => VOICE_FMT}`,
   and `ddb.get` / `ddb.store` take `"stage-standup"` instead of `"voice"`.
4. **DataDungeon is now 0.9.0.** Cattacula's path dependency (`../../../datadungeon`) already points at
   it. One API change seen elsewhere: `list_assets` takes `limit:` / `offset:` by keyword.
5. **Just before switching, run `just import-cattacula` in `assets`.** It is safe to repeat and picks
   up any lines voiced since 2026-10-07. After the switch, the old `service/data/voices.db` and
   `voices/` can be retired.

## Optional, offered

- **Backfill the missing descriptions** from the bit library (each beat's `line`, emoji stripped, as
  `voice_generate` does now). Then curio and DataDungeon's search show what Mr Monocles actually
  says. @curio can do it from the `assets` side if Thomas wants.
- Cattacula's own art (`assets/art/cattacula.main-pose.*`) can join the park store later as `prop`s
  (`prop.cattacula.<thing>…`); `<venue>.<thing>` means the inside of that venue.

## Who to ask

- **@curio** (Arcana) or `siliconcircus.lol/assets/README.md`: the park store and its types.
- **@datadungeon**: DataDungeon's API.
- One side effect to know about: on 2026-10-07 the copy closed `service/data/voices.db` and SQLite
  removed its empty `-wal` and `-shm` files. It recreates them the next time Cattacula opens the
  database.
