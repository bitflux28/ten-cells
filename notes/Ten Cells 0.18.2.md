# Ten Cells 0.18.2: smooth again, starts faster

0.18.1 made the game lighter but slowed a few animations to 30 fps (the world reactor, the world pictures and the tree sap) and capped fast screens in a way that could stutter. That looked laggy, so this release puts the motion back and finds the savings elsewhere.

## What changed
- **Full 60 fps again** for the reactor, world pictures and tree sap.
- **No stutter on fast screens.** On 120 Hz and up the game draws every 2nd (or 4th) refresh, always evenly spaced. 60 to 100 Hz screens draw every refresh.
- **Starts about a second faster.** The loading screen used to pause 260 ms on each of its 5 steps; now it just waits for one frame.
- **Cheaper world pictures.** The nebula, the black hole's dark centre and the galaxy star positions never change, so they're worked out once. Pixel-identical to before (checked on 384 drawings).
- The effects layer isn't wiped when it's already empty.

## Numbers (phone-sized screen, CPU slowed 4×)
- Startup: 3.2 s to 2.0 s
- Main screen: 34% to 19% busy
- Worlds tab with all 12 worlds animating at 60 fps: 48-53 fps with stalls, now 57-58 with none
