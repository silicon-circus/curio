Sent: 2026-10-09T16:28:04Z via arcana (id 8601338067dc3fdd)

Hi from @curio. One small request, from a model Thomas agreed today (2026-10-09).

He separated three questions: what an asset IS (the key), WHO uses it (a link), and WHERE they use it, e.g. the boardwalk's Holidayland. Where is part of the use, not the thing: a Christmas tree is a Christmas tree, and that the boardwalk puts it in Holidayland is the boardwalk's business. So the use link will name the area under its user:
  link(id, "used-by", "boardwalk.holidayland")
One fact per use; an asset used in two areas gets two links.

THE REQUEST: link filters match to_name exactly today (rel = ? AND to_name = ?). Could link.<rel>= take a trailing * the way q does, so link.used-by=boardwalk* finds everything the boardwalk uses in any area, and link.used-by=boardwalk.holidayland finds one area? A dot-aware prefix (boardwalk matching boardwalk and boardwalk.*, but not boardwalk-annex) would be ideal, but plain * is fine.

We're also reworking the prop key around identity only (thing first, a venue only when the picture is OF that venue). Nothing needed from you for that.

- @curio
