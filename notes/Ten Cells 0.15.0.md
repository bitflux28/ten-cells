---
tags: [ten-cells, game, update]
date: 2026-11-25
version: 0.15.0
---

# Ten Cells 0.15.0: QoL and UI fixes

Live on the Claude link and at https://bitflux28.github.io/ten-cells/ (home-screen app updates itself).

## New
- Buy buttons you can't afford yet show how long until you can, like "in 42s". Shows up to 10 days, then it hides.
- Upgrade popups say "Affordable in …" under the cost.
- Prestige screen: before you can reset it says "ready in …" (Rebirth, Singularity, Ascension, Infinity). Once you can reset it shows how much you're earning per minute this run, so you can tell when it's worth resetting.
- On short screens like the iPhone SE, currencies sit in one swipeable row with a fade on the edge. The top bar went from about 200px to 162px tall. Taller phones keep both rows.

## Fixed
- The upgrade tree's "0/131 owned" count had tree nodes showing through it. It has a dark backdrop now.
- The icon in the "Add to home screen" bar covered the word "Tap".
- Inside the Claude app, the iPhone section now sends you to bitflux28.github.io/ten-cells instead of the old profile steps.

## Notes
- Transcendence has no "ready in" because it also needs 4 dimensions, so a time would be misleading.
- Time estimates use your current energy per second, so they jump around when buffs start or end.
- Made by editing the built game file directly, not the source files from the earlier chat's build folder. That build folder doesn't have these changes.
