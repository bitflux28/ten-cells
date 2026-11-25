# Builds "Ten Cells for Windows.zip": a 2 MB Rust app that shows the game in Windows' own WebView2 engine.
import os, shutil, subprocess, zipfile, json, pathlib
w = pathlib.Path(__file__).resolve().parent; tc = pathlib.Path('/home/claude/tc')
v = json.loads((tc / 'version.json').read_text())['v']
shutil.copy(tc / 'index.html', w / 'game.html')
env = dict(os.environ, RUSTC_BOOTSTRAP='1')
subprocess.run(['cargo', 'zigbuild', '--release', '--target', 'x86_64-pc-windows-gnu', '-Zbuild-std=std,panic_abort'], cwd=w, env=env, check=True)
out = w / 'dist' / 'Ten Cells'; shutil.rmtree(out.parent, ignore_errors=True); out.mkdir(parents=True)
shutil.copy(w / 'target/x86_64-pc-windows-gnu/release/ten-cells.exe', out / 'Ten Cells.exe')
loader = next(pathlib.Path.home().glob('.cargo/registry/src/*/webview2-com-sys-*/x64/WebView2Loader.dll'))
shutil.copy(loader, out / 'WebView2Loader.dll')
(out / 'READ ME.txt').write_text(f"""Ten Cells {v} for Windows

How to play
1. Unzip this whole folder somewhere, for example your Desktop or Documents. Don't run it from inside the zip.
2. Double-click "Ten Cells.exe". Keep WebView2Loader.dll next to it.
3. Windows may say "Windows protected your PC" because the app isn't signed by a big publisher. Click "More info", then "Run anyway".

This version is tiny (about 2 MB) because it uses Microsoft Edge WebView2, the web engine built into Windows 10 and 11, instead of carrying its own copy of Chrome. The game itself is exactly the same.

Your save is stored in %APPDATA%\\Ten Cells. The first time you start it, it brings over your save from the older Ten Cells for Windows if it finds one.
If it doesn't, open the old version, go to Stats > Settings > Export, then Import in this one.
Press F11 for full screen. No internet connection is needed.
""", newline='\r\n')
zp = tc / 'release' / 'Ten Cells for Windows.zip'; zp.unlink(missing_ok=True)
with zipfile.ZipFile(zp, 'w', zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    for f in sorted(out.rglob('*')): z.write(f, f.relative_to(out.parent))
print(zp, round(zp.stat().st_size / 1e6, 2), 'MB', 'version', v)
