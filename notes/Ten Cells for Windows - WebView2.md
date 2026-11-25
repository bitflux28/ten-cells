# Ten Cells for Windows: the light version

The Windows app is now a small Rust program instead of Electron. The game inside is the exact same HTML file as the website and iPhone version, so the UI doesn't change at all.

## What changed
- **Download:** 0.78 MB (was 74 MB).
- **On disk:** about 2 MB (was 230 MB).
- **How:** Electron carried its own full copy of Chrome. This app uses WebView2, the Edge web engine that's already part of Windows 10 and 11.
- **Saves** live in `%APPDATA%\Ten Cells`. The first time it starts, it finds your save from the old Windows app and brings it over (it checks the save's checksum first). If it can't find one, use Stats > Settings > Export in the old app and Import in the new one.
- F11 for full screen, links open in your normal browser, and there's no browser right-click menu, reload key or zoom.
- If WebView2 is somehow missing, it shows a message with Microsoft's download link instead of crashing.

## Files
- `Ten Cells.exe` (1.9 MB) and `WebView2Loader.dll` (Microsoft's small loader, 160 KB). Keep them together.
- Source: `windows/` in the repo (Rust, using the `wry` and `tao` libraries that Tauri apps use).

## Tested
- Cross-compiled for Windows x64. The exe is a normal windowed app with its icon, and needs only standard Windows DLLs plus the loader that ships beside it.
- The save import was tested on real save files from the old app: the save came over intact, and it isn't re-imported after you keep playing.
- Not yet run on a real Windows PC. The old Electron version is still in the repo as a fallback.
