// Ten Cells desktop shell: the same game file as the web version, shown by the Chromium bundled in this app.
const { app, BrowserWindow, Menu, dialog, shell, net } = require('electron');
const path = require('path');
const fs = require('fs');
if (!app.requestSingleInstanceLock()) app.quit();
app.commandLine.appendSwitch('autoplay-policy', 'no-user-gesture-required');
app.commandLine.appendSwitch('disable-renderer-backgrounding');
app.commandLine.appendSwitch('disable-background-timer-throttling');
// The game file can be updated in place: a newer copy is saved next to the save data and used from the next start, no new download needed.
const BUNDLED = path.join(__dirname, 'index.html'), UPDATED = path.join(app.getPath('userData'), 'game.html');
const gver = f => { try { const m = /BUILD_V = "([\d.]+)"/.exec(fs.readFileSync(f, 'latin1')); return m ? m[1] : '0'; } catch (e) { return '0'; } };
let broken = false;
const log = m => { try { fs.appendFileSync(path.join(app.getPath('userData'), 'update.log'), new Date().toISOString() + ' ' + m + '\n'); } catch (e) { } };
const gameFile = () => (!broken && fs.existsSync(UPDATED) && !newer(gver(BUNDLED), gver(UPDATED)) ? UPDATED : BUNDLED);
let win;
app.on('second-instance', () => { if (win) { if (win.isMinimized()) win.restore(); win.focus(); } });
app.whenReady().then(() => {
  Menu.setApplicationMenu(null);
  win = new BrowserWindow({ width: 540, height: 960, minWidth: 360, minHeight: 560, backgroundColor: '#0b0a1f', title: 'Ten Cells', icon: path.join(__dirname, 'icon.ico'), autoHideMenuBar: true, show: false,
    webPreferences: { backgroundThrottling: false, spellcheck: false } });
  win.once('ready-to-show', () => { win.show(); win.webContents.focus(); });
  win.webContents.on('before-input-event', (e, i) => { if (i.type === 'keyDown' && i.key === 'F11') { win.setFullScreen(!win.isFullScreen()); e.preventDefault(); } });
  win.webContents.setWindowOpenHandler(() => ({ action: 'deny' }));
  win.loadFile(gameFile());
  // a downloaded game file that fails to load must never brick the app: fall back to the bundled one
  win.webContents.on('did-fail-load', (e, code, desc, url, isMain) => { if (!isMain || code === -3 || broken || gameFile() === BUNDLED) return; broken = true; log('load failed ' + code + ' ' + desc); win.loadFile(BUNDLED); });
});
// Once per launch, ask GitHub which app version is newest (a tiny file, nothing about you is sent). Any failure is ignored.
const newer = (a, b) => { const x = a.split('.').map(Number), y = b.split('.').map(Number); for (let i = 0; i < 3; i++) if ((x[i] || 0) !== (y[i] || 0)) return (x[i] || 0) > (y[i] || 0); return false; };
let asked = '', busy = false;
async function checkUpdate() {
  try {
    const r = await net.fetch('https://raw.githubusercontent.com/bitflux28/ten-cells/root/downloads/app.json', { signal: AbortSignal.timeout(8000) });
    const v = (await r.json()).v;
    if (typeof v === 'string' && newer(v, app.getVersion()) && asked !== v) { asked = v;
      const { response } = await dialog.showMessageBox(win, { type: 'info', title: 'Ten Cells', message: 'The Engine has a new shell', detail: `Version ${v} is out (you have ${app.getVersion()}). The game itself updates by itself; this one needs a fresh download. Your save carries over.`, buttons: ['Download', 'Later'], defaultId: 0, cancelId: 1 });
      if (response === 0) shell.openExternal('https://github.com/bitflux28/ten-cells/raw/root/downloads/Ten%20Cells%20for%20Windows.zip');
    }
  } catch (e) { }
}
async function updateGame() {
  try {
    const have = gver(gameFile()), r = await net.fetch('https://raw.githubusercontent.com/bitflux28/ten-cells/root/version.json', { signal: AbortSignal.timeout(8000) });
    const v = (await r.json()).v;
    if (typeof v !== 'string' || !newer(v, have)) return false;
    const g = await net.fetch('https://raw.githubusercontent.com/bitflux28/ten-cells/root/downloads/Ten%20Cells.html', { signal: AbortSignal.timeout(60000) });
    const html = await g.text();
    if (!g.ok || html.length < 500000 || !html.includes('BUILD_V = "' + v + '"')) return false;
    try { fs.writeFileSync(UPDATED + '.tmp', html); fs.renameSync(UPDATED + '.tmp', UPDATED); } catch (e) { log('rename failed, writing directly: ' + e.message); fs.writeFileSync(UPDATED, html); }
    log('saved game ' + v);
    const { response } = await dialog.showMessageBox(win, { type: 'info', title: 'Ten Cells', message: 'A new pulse reaches the Engine', detail: `Version ${v} has settled into the cells. Restart and let the Engine wake with it. Your save carries over.`, buttons: ['Wake the Engine', 'Later'], defaultId: 0, cancelId: 1 });
    if (response === 0) { app.relaunch(); app.exit(0); }
    return true;
  } catch (e) { log('update failed: ' + e.message); return false; }
}
// first check 4 s after launch, then every 30 seconds while the app is open (each version is announced once)
const tick = async () => { if (busy) return; busy = true; try { if (!(await updateGame())) await checkUpdate(); } finally { busy = false; } };
app.whenReady().then(() => { setTimeout(tick, 1500); setInterval(tick, 30000); });
app.on('window-all-closed', () => app.quit());
