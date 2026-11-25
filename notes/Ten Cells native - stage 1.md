# Ten Cells native, stage 1

The Windows version is now a real C++ app instead of a wrapped web page. The download dropped from 115 MB to under 2 MB, and it opens straight away.

## What's in it
- The tap reactor, all 14 energy makers, buy x1/x10/x25/max, and Buy Everything
- The full upgrade tree, Research and Automation
- Every prestige layer and shop: Rebirth perks, Singularity and the Black Hole, Ascension, Transcend and the Reality Laws, the Infinity tree, Ultra, and Challenges
- Events, abilities, buffs, achievements and offline progress
- Saves every 5 seconds and on close, with a backup copy. Export and Import use the same codes as the web and iPhone versions
- Pixel look with the same fonts, sprites and colours, a starfield sky, particles and a spinning atom reactor
- Sound effects, generated in the app

## How it works
The rules are the web game's own code, run inside a small JavaScript engine (QuickJS) built into the exe. So the numbers match the web game exactly, and a fix to the web rules carries straight over. Everything you see and hear is C++ (raylib).

## Moving your save over
In the old app or the website: Stats > Settings > Export. Then in the native app: Stats > Settings > Import. It reads the code from your clipboard.

## Keys
Space taps, 1 to 4 switch tabs, G buys everything, M mutes, F11 is full screen. Hold a buy button to keep buying.

## Coming in stage 2
Worlds and their skies, the story with the 3D cast, bosses, pets and the achievements screen. Their bonuses already count in stage 1, you just can't open those screens yet.

Download: `downloads/Ten Cells native for Windows.zip` in the GitHub repo. Source is in `native/`.
