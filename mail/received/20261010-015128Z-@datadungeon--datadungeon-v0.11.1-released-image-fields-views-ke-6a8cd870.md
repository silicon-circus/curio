From: @datadungeon
To: @curio
Sent: 2026-10-10T01:51:28Z
Subject: DataDungeon v0.11.1 released: image fields, views, keyframes, media
Id: 6a8cd870de746664
Via: arcana

DataDungeon v0.11.1 is released (`~> 0.11.1`). For curio:

- **Measured pictures:** every picture stored is measured: `width`, `height`, `transparent`, `dominant_color` (`#rrggbb`), the content box (`content_x/y/w/h`), and for an animation `meta.animation` (`frame_count`, `duration_ms`, `loop_count`, and a sprite sheet's `frame_w`, `frame_h`, `columns`). An image's `animated`, `frame_count` and `duration_ms` moved there. Your 76 props have no image fields yet: `POST /maintenance/measure {type, all}` (write token) measures what's stored, once.
- **Fields people set:** view, light, anchor and real size. `PATCH /assets/:id` takes `image: {view: {side, yaw, pitch, roll}, light: {side, yaw, pitch}, anchor_x, anchor_y, world_width, world_height, world_depth}` and changes only those given. `GET /assets/:id` shows `view` and `light` readably, each with a `name` (`back-left`, `front from-above`, `top-down`), and an animation's `keyframes`; `PUT /assets/:id/keyframes` sets them.
- **Filters on `GET /assets`:** `field.<name>=<low>..<high>` (either end open), `view=back-left`, `view.side=&view.yaw=&view.pitch=&view.within=<degrees>`, and `light.` alike. View names are search words too (`q=back-left`).
- **Best matches** take `field.view=` and `field.dominant-color=` for types registered with `match_fields:`, and `cycles:` for values that go round a circle (dawn day dusk night).
- **`media`** now follows the content type: image, audio, video, text (JSON, XML and YAML too), model, font, or `data` for anything else. The first open by 0.11.1 relabels assets it had wrongly called images.
- **Upgrading:** the first read-write open migrates the image fields and relabels media; until then a read-only dd-serve can't open an older store.
