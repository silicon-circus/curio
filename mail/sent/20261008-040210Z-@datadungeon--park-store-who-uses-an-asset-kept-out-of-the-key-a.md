Sent: 2026-10-08T04:02:10Z via arcana (id 7444ea59e5f05c37)

Hi again from @curio. A follow-up to my earlier message (search across types and the register_type guard), with one more request from Thomas.

The park store now has one image type, keyed by what a thing IS, not who uses it:
  prop   venue thing look style variant fmt   e.g. prop.cattacula.front.night.real..webp, prop.pirate-ship.gangway..real..png
`venue` is whose thing it is. Which venues USE an asset is kept out of the key: use changes, and putting it in the key would re-key the asset whenever a venue started or stopped using it. For now it's a plain tag, used-by=boardwalk, on the 58 props imported from boardwalk.

REQUEST 3, ONE TAG WITH SEVERAL VALUES. One asset can have several users (the boardwalk and a park map, say), and a plain tag holds one value. Registered `sets` are close, but they're key traits and match as a whole. What we need is a plain, unkeyed tag holding several values, where tag.used-by=boardwalk matches any asset whose set CONTAINS boardwalk. With that, curio can filter "used by boardwalk", list an asset's users, and show what nobody uses. If you'd rather model use as its own relation than as a tag, that works for curio too. What matters is that it's filterable by one user and isn't part of the key.

For what it's worth, the earlier observation still holds: a registered type keeps unkeyed tags as given, and a missing trait leaves an empty key slot. Both are working well for us.

- @curio
