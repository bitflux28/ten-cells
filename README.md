# Ten Cells

An idle game about an ancient engine that makes energy from nothing. It climbs from an atom to the multiverse and past it. Current version: **0.18.2**.

**Play:** https://bitflux28.github.io/ten-cells/

## On iPhone
Open the link above in Safari, tap Share, then Add to Home Screen. It plays full screen and works offline after the first open.
You can also install `downloads/TenCells-iPhone.mobileconfig` (Settings › Profile Downloaded › Install) to add the icon in one step.

## Files
- `index.html`, `sw.js`, `manifest.webmanifest`, icons: the web app served by GitHub Pages (from the `gh-pages` branch)
- `downloads/Ten Cells native for Windows.zip`: the new native Windows app (C++, 2 MB, starts instantly). Unzip and run `Ten Cells.exe`. Experimental and paused: it has the engine, upgrades, every prestige layer, saves and sound, but no worlds, story, bosses or pets screens. The regular Windows app below is the main one.
- `native/`: its source (raylib + QuickJS + C++). See `native/README.md`.
- `downloads/Ten Cells for Windows.7z`: the Windows app. Right-click › Extract All (Windows 11 opens .7z on its own; on Windows 10 use 7-Zip), then run `Ten Cells.exe`. Saves from older Windows versions carry over.
- `downloads/Ten Cells.html`: the whole game in one file. Open it in any browser.
- `downloads/Ten Cells web app (iPhone).zip`: the web app, ready to drop on any static host
- `notes/`: release notes and preview pictures
