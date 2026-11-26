# Ten Cells

An idle game about an ancient engine that makes energy from nothing. It climbs from an atom to the multiverse and past it. Current version: **0.21.4**.

**Play:** https://bitflux28.github.io/ten-cells/

## Files
- `index.html`, `sw.js`, `manifest.webmanifest`, icons: the web app served by GitHub Pages (from the `gh-pages` branch)
- `wasm/`: the Rust source of the small WebAssembly module that paints the sky (built into `index.html`)
- `downloads/Ten Cells for Windows.zip`: **the Windows app** (103 MB). Unzip it and run `Ten Cells.exe`. It's the same game with its own browser engine (Electron) built in, so no browser is needed. F11 is full screen. The app checks once at launch whether a newer version exists (`downloads/app.json`) and offers the download.
- `downloads/Ten Cells.html`: the whole game in one file. Open it in any browser.
- `notes/`: release notes for the latest versions
- `tests/`: the browser tests used before each release

## Privacy and security
- The game itself makes no network requests. (The Windows app makes a few at launch: it reads `version.json` from GitHub and, if the game is newer, downloads the new game file and offers a restart, so game updates need no new zip. It also reads `downloads/app.json` to see if a newer app shell exists.) No accounts, no tracking, no analytics. Your save stays on your device.
- A Content Security Policy blocks any script, connection or form that doesn't come from the game itself.
- Playing from GitHub Pages doesn't show anyone your IP address. GitHub hosts the page; the game never talks to anything.
