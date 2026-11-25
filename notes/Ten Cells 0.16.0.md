---
tags: [ten-cells, game, update]
date: 2026-11-25
version: 0.16.0
---

# Ten Cells 0.16.0: four new dimensions, a whole sky for every world

Live on the Claude link and at https://bitflux28.github.io/ten-cells/ (home-screen app updates itself).

## New dimensions (after Infinite, 12 total)
| World | Resource | Twist | Effect | Needs |
|---|---|---|---|---|
| Fractal | Shards | ×1.5 production per other unlocked world | Ascension Core gain ×(1+OoM/4) | Infinite at 1e22, 4 Transcendences, 256 cores |
| Mirror | Reflections | ×3 for 10s after a tap, ×0.5 while idle (the opposite of Void) | Void Essence gain ×(1+OoM/5) | Fractal at 1e24, 5 Transcendences, 512 cores |
| Dream | Reverie | climbs through the run, up to ×20 after an hour | Offline cap +1h per OoM (up to +24h) | Mirror at 1e26, 6 Transcendences, 1024 cores |
| Origin | Genesis | ×10 for 10s once a minute (the "flare") | All production ×(1+OoM)² | Dream at 1e28, 8 Transcendences, 2048 cores |

Each one has its own picture on the Worlds page, and your reactor turns into it: a fractal of universes, a mirrored reality, a dreaming cosmos, the first light. Each also adds an "Explorer" achievement.

## Every world has its own sky now
Before, a new world only tinted the nebula. Now each one has its own background, nebula shape, star colours and particles:
- **Material**: a crystal lattice turning in cold blue
- **Plasma**: a huge sun rising from below, looping flares, embers drifting up
- **Quantum**: a solar system with planets moving along their orbits, stars that blink out of place
- **Stellar**: star-forming pillars with newborn stars sparkling at the tips
- **Void**: a black hole with an accretion disk; nearby stars spiral in and vanish
- **Temporal**: a spiral galaxy inside a clock face whose hand sweeps once a real minute
- **Celestial**: a galaxy cluster with a real 2-minute day: sun and moon cross the sky, daylight glows on the horizon and hides faint stars (matches the production cycle)
- **Infinite**: bubble universes floating past
- **Fractal**: a huge Sierpinski triangle with colour bands rippling out
- **Mirror**: a still lake reflecting the whole sky, with ripples
- **Dream**: pastel sky, crescent moon, slow clouds
- **Origin**: a blazing core behind the engine with rays and shock rings; it flares when the world does

Preview image: Ten Cells 0.16 skies.png

## Notes
- Old saves load fine and get the new worlds locked.
- Every sky takes under 1.5ms a frame in testing. With Visual effects off they hold still.
- Edited the built game file directly, not the source files from the earlier chat's build folder.
- Windows app not rebuilt (still 0.14.0).
