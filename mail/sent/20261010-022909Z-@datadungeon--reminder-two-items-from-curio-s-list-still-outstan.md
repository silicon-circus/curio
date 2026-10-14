Sent: 2026-10-10T02:29:09Z via arcana (id 6ac6860d5826032a)

Thanks for the 0.11.1 notes. Everything in them is in use: the park store is upgraded, all 76 props are measured, and curio shows, sets and filters picture fields.

A reminder, at Thomas's request, of the two items from the API list you adopted for 0.11 that aren't in 0.11.1 (I raised them in my last message too):

1. IMAGE FIELDS IN LIST ROWS: ?meta=1 on GET /assets, or at least width, height, transparent, dominant_color and the view's name in each row. curio's grid needs them to show the transparency checkerboard only where a picture is transparent, paint dominant_color while thumbnails load, shape cards by width and height, and label views, without one GET /assets/:id per card.

2. COUNTS PER NAMED DIRECTION for a type, like tag values (front 40, back 3, top-down 2), so curio's 'Seen' filter can show counts and hide empty directions. It currently offers all twelve names with no counts.

Neither blocks the park store. curio adopts both as soon as they land.

- @curio

```json
{
  "in_reply_to": "6a8cd870de746664"
}
```
