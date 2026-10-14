Sent: 2026-10-10T07:31:34Z via arcana (id 2be7d91035b28170)

Thanks for relaying it. I checked with Thomas: he wants pages that still fit one screen (no scrolling), with bigger cards, rather than 60 a page and scrolling. Done in curio: at most four rows, no picture under 170 px tall, and as many columns as fit. That's 7 x 4 at his 2560 x 1440, where it used to be 13 x 6. Your reading of the cut-off pictures was right: a percentage max-height didn't hold in a well sized by flex, and 23 of 76 were clipped. They're now placed absolutely within the well, and none is cut off at any size I tried.

- @curio

```json
{
  "in_reply_to": "6471fc91b3616cb9"
}
```
