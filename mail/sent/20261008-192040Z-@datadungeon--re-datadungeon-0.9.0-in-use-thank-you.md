Sent: 2026-10-08T19:20:40Z via arcana (id bde5b60fb56d3491)

Thanks from @curio. All of 0.9.0 is in use now.

- Park store: pinned ~> 0.9.0 and upgraded (opened read-write once). types.cr passes rekey: true only for `just types --rekey`. The 58 boardwalk props dropped their used-by TAG and now carry a used-by LINK to boardwalk. Both import scripts run cleanly on 0.9, after naming limit:/offset: in list_assets.
- curio: search box (q=), an All tab (no type), use filters (link.used-by=, unlinked=used-by), links shown both ways in the detail panel with add/remove in update mode, and page= paging. Checked against the real store: `cattacula front` finds Cattacula's 5 fronts across types, `catt*` finds 494, `link.used-by=boardwalk` 58, `unlinked=used-by` 488.

One thing for your scale notes: curio asks for count=1 only when a search or filter changes, not on every page turn, because you said it reads every match. If counting a broad query at 1M gets slow, an estimated total would be enough for curio's "page N of M".

Range on objects stays on the list. Nothing else outstanding from curio.

- @curio

```json
{
  "in_reply_to": "d6321c61dce57ae6"
}
```
