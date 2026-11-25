# Ten Cells 0.18.1: runs lighter

A speed-up release. Nothing in the game changed and nothing looks different. It just does less work for the same picture.

## What changed
- **Worlds tab:** it used to stutter, dropping to around 41-48 fps on a slow phone with all the world pictures animating. Now it holds 60. Locked worlds are drawn once instead of every frame, the colour-to-grey conversion for locked worlds is worked out once per colour, and the small world pictures animate at 30 fps, like the prestige portal already did.
- **Fast screens:** on 120 Hz and 144 Hz screens (newer iPhones, gaming monitors) the game now holds 60 fps. Before, every animation ran 2 to 2.4 times faster there and used that much more battery.
- **Reactor, portal, skies:** the shaded pixel balls (atom nucleus, planets, stars in the world pictures) are drawn once and reused. The portal's progress ring is redrawn only when it changes. Pixel lines are drawn a whole straight run at a time, not one pixel at a time. Around a quarter less work on the main and prestige screens.
- **Windows app:** dropped 54 language packs the game never uses. The download goes from 81 MB to 74 MB and the installed folder from 281 MB to 230 MB.

## How it was checked
- Profiled every tab on a phone-sized screen with the CPU slowed 4×, before and after.
- Compared the new drawing code with the old code: 3,810 cases, pixel-identical.
- All the earlier test suites still pass, and the Windows wrapper still starts (also on a non-English system).
