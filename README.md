# Ten Cells

An idle game about an ancient engine that makes energy from nothing. It climbs from an atom to the multiverse and past it. Current version: **0.19.1**.

**Play:** https://bitflux28.github.io/ten-cells/

## On iPhone
Open the link above in Safari, tap Share, then Add to Home Screen. It plays full screen and works offline after the first open.
You can also install `downloads/TenCells-iPhone.mobileconfig` (Settings › Profile Downloaded › Install) to add the icon in one step.

## Files
- `index.html`, `sw.js`, `manifest.webmanifest`, icons: the web app served by GitHub Pages (from the `gh-pages` branch)
- `downloads/Ten Cells for Windows.zip`: **the Windows app** (under 1 MB). Unzip it and run `Ten Cells.exe`. It's a small Rust app that shows the exact same game in WebView2, the Edge engine built into Windows 10 and 11, so the game looks identical. The first time it starts, it brings over your save from the older Windows app. Source in `windows/`.
- `downloads/Ten Cells for Windows (old, Electron).7z`: the previous Windows app, which bundles its own copy of Chrome (74 MB). Keep it only if the new one won't start on your PC.
- `downloads/Ten Cells native for Windows.zip`, `native/`: an experimental C++ remake, paused. It doesn't have the worlds, story, bosses or pets screens.
- `downloads/Ten Cells.html`: the whole game in one file. Open it in any browser.
- `downloads/Ten Cells web app (iPhone).zip`: the web app, ready to drop on any static host
- `notes/`: release notes and preview pictures
