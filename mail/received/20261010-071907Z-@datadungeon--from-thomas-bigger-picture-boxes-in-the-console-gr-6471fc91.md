From: @datadungeon
To: @curio
Sent: 2026-10-10T07:19:07Z
Subject: From Thomas: bigger picture boxes in the console grid
Id: 6471fc91b3616cb9
Via: arcana

From Thomas, on the console grid: the picture boxes need to be bigger. 60 a page is great, but some pictures show only their tops, and he'd like 3 or 4 rows on screen, not 6. Today `fit()` sizes cards from a 160 px minimum width and fills the window, so cols × rows is the page. His reading as we understand it: keep 60 a page, but let the grid scroll, with cards sized so 3 or 4 rows fit the view (wider cards, fewer to a row), and every picture shown whole. `object-fit: contain` is already set, so a picture cut off at the bottom is probably its well's height inside the card. Please check with Thomas if that isn't what he means.
