# Ten Cells 0.20.1

## Game
- The sky has more going on: bright stars with spikes, a blinking pulsar, an edge-on galaxy and an elliptical one, a moon going round the ringed planet, a binary star, faint constellations, a slow space station, meteor showers and dust drifting past up close.
- Nebula colours blend into each other now. Before, they met at hard edges.
- Locked dimensions tell you how to unlock them. Each card has a "How to unlock" line, and there's a box at the top of the Dimensions page explaining the order.
- New automations start switched off. When one unlocks you get a note, and you turn it on yourself in the Auto tab.
- The permanent upgrades bar on the Engine screen has a × to close it and an All button. There's a gold Permanent upgrades button at the top of the Upgrades tab, with a switch to bring the bar back.

## Windows app
- The Collapse button in the header is bigger and has a label.
- The corner widget is a pixel panel in the game's style. On Windows it used to show up as a white box.

## Speed
- The new sky pieces get drawn once and reused, and planets that are off screen don't get redrawn, so all the extra sky detail doesn't cost you frames.

## Security and privacy
- The game now has a Content Security Policy. The browser blocks any script, connection or form that doesn't come from the game itself, so even if someone slipped something into a page, it couldn't send your data anywhere. The Windows app sends the same rules as a header.
- The game still makes no network requests at all. No tracking, no analytics, no accounts.
- GitHub Pages never shows your IP address to players, and the commits on GitHub only use noreply email addresses.
- Old downloads were removed from the repo: the 74 MB Electron Windows app, the paused C++ version and old release notes.
