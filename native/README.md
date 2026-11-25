# Ten Cells native

A desktop build of Ten Cells written in C++. The window, drawing, input, sound and save files are C++ (raylib). The game rules are the web version's own `data.js` and `logic.js`, run unchanged inside QuickJS, so numbers always match the web game and saves move between them.

## Layout
- `src/main.cpp`: window, HUD, pages, reactor, effects, input
- `src/js.cpp`: QuickJS setup and the save files (`save.txt` plus `save_bak.txt`, written atomically)
- `src/audio.cpp`: sound effects, synthesized at startup (no audio files)
- `js/api.js`: the bridge. Each screen comes back as a list of simple blocks the C++ side draws
- `js/data.js`, `js/logic.js`, `js/decimal.js`: copied from the web game's `index.html`
- `assets/`: pixel fonts, sprite sheet, icon. All of it is embedded into the exe

Saves live in `%APPDATA%\Ten Cells` on Windows and `~/.local/share/tencells` on Linux.

## Building
Put raylib 5.5 in `ext/raylib` and quickjs-ng (0.18) in `ext/quickjs`, then:

```
pip install ziglang fonttools
python3 build.py windows   # build/windows/Ten Cells.exe (cross-compiled with zig)
python3 build.py linux     # build/linux/tencells (needs gcc and X11 headers)
```

`tencells --data <dir> --shots <prefix>` uses another save folder and saves a screenshot of every screen, then quits.

Keys: Space taps the reactor, 1 to 4 switch tabs, G buys everything, M mutes, F11 goes full screen. Hold a buy button to keep buying.
