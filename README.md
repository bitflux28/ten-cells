# Ten Cells

An idle game about an ancient engine that makes energy from nothing. It climbs from an atom to the multiverse and past it. Current version: **0.21.0**.

**Play:** https://bitflux28.github.io/ten-cells/

## On iPhone
Open the link above in Safari, tap Share, then Add to Home Screen. It plays full screen and works offline after the first open.
You can also install `downloads/TenCells-iPhone.mobileconfig` (Settings › Profile Downloaded › Install) to add the icon in one step.

## Files
- `index.html`, `sw.js`, `manifest.webmanifest`, icons: the web app served by GitHub Pages (from the `gh-pages` branch)
- `downloads/Ten Cells for Windows.zip`: **the Windows game**, a native Rust app (one 7 MB `Ten Cells.exe`, no browser inside). Same rules and numbers as the web version, same art, synthesised music. Saves to Documents › Ten Cells, brings over saves from the older Windows apps, and export codes work both ways with the web version. Source in `native/` (`native/bake_assets.py` renders the art out of the web game). The older WebView2 wrapper is still in `windows/`.
- `downloads/Ten Cells.html`: the whole game in one file. Open it in any browser.
- `downloads/Ten Cells web app (iPhone).zip`: the web app, ready to drop on any static host
- `notes/`: release notes for the latest versions
- `tests/`: the browser tests used before each release

## Privacy and security
- The game makes no network requests. No accounts, no tracking, no analytics. Your save stays on your device.
- A Content Security Policy blocks any script, connection or form that doesn't come from the game itself. The Windows app sends the same policy as a header and only serves the game, nothing else.
- Playing from GitHub Pages doesn't show anyone your IP address. GitHub hosts the page; the game never talks to anything.
