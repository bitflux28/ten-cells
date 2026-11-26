# Ten Cells

An idle game about an ancient engine that makes energy from nothing. It climbs from an atom to the multiverse and past it. Current version: **0.21.2**.

**Play:** https://bitflux28.github.io/ten-cells/

## Files
- `index.html`, `sw.js`, `manifest.webmanifest`, icons: the web app served by GitHub Pages (from the `gh-pages` branch)
- `wasm/`: the Rust source of the small WebAssembly module that paints the sky (built into `index.html`)
- `downloads/Ten Cells.html`: the whole game in one file. Open it in any browser.
- `notes/`: release notes for the latest versions
- `tests/`: the browser tests used before each release

## Privacy and security
- The game makes no network requests. No accounts, no tracking, no analytics. Your save stays on your device.
- A Content Security Policy blocks any script, connection or form that doesn't come from the game itself.
- Playing from GitHub Pages doesn't show anyone your IP address. GitHub hosts the page; the game never talks to anything.
