// Ten Cells for Windows. The game is the same single HTML file as the web and iPhone versions, shown in
// WebView2 (the Edge engine built into Windows 10 and 11) instead of a bundled copy of Chrome.
#![windows_subsystem = "windows"]

use std::{borrow::Cow, fs, path::PathBuf, time::{Duration, Instant}};
use tao::{
    dpi::{LogicalSize, PhysicalPosition},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Fullscreen, Icon, Window, WindowBuilder},
};
#[cfg(windows)]
use tao::platform::windows::WindowBuilderExtWindows;
use wry::{http::Response, WebContext, WebView, WebViewBuilder};
#[cfg(windows)]
use wry::WebViewBuilderExtWindows;

// game.html is the same file as the web version, copied in by make_windows.py
static GAME: &[u8] = include_bytes!("../game.html");
static ICON: &[u8] = include_bytes!("../res/icon64.rgba");
const ORIGIN_HOST: &str = "tencells.localhost";

enum UserEvent { Fullscreen, Open(String), Size(f64, f64), Collapse, Mini(String), Restore, Drag, Quit }

// every webview shares one WebView2 environment, so they must all get the same browser arguments.
// Timers keep full speed while the window is hidden, so a collapsed game keeps earning.
const BROWSER_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,IntensiveWakeUpThrottling --autoplay-policy=no-user-gesture-required --disable-background-timer-throttling --disable-renderer-backgrounding";

// the little atom that sits in the corner while the game is collapsed. It redraws ~15 times a second and nothing else.
const MINI_HTML: &str = r##"<!doctype html><html><head><meta charset="utf-8"><style>
html,body{margin:0;background:transparent;overflow:hidden;user-select:none;cursor:pointer;font:700 11px Consolas,monospace;color:#fff}
#w{position:fixed;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center}
canvas{width:72px;height:72px;image-rendering:pixelated}
#e{margin-top:2px;padding:1px 6px;background:#1b1845;box-shadow:inset 0 0 0 2px #3d3780;text-shadow:1px 1px 0 #0b0a1f;white-space:nowrap}
#x{position:fixed;top:2px;right:2px;width:16px;height:16px;line-height:15px;text-align:center;background:#ff4f73;color:#fff;display:none}
body:hover #x{display:block}
</style></head><body><div id="w" title="Ten Cells: click to open, drag to move"><canvas id="c" width="36" height="36"></canvas><div id="e">Ten Cells</div></div><div id="x" title="Quit Ten Cells">x</div>
<script>
const c = document.getElementById('c'), x = c.getContext('2d'), t0 = performance.now();
const px = (X, Y, col) => { x.fillStyle = col; x.fillRect(Math.round(X), Math.round(Y), 1, 1); };
function draw(t) {
  const a = (t - t0) / 1000; x.clearRect(0, 0, 36, 36);
  [[0, '#45c8f5'], [1.05, '#8a63ff'], [-1.05, '#3466f0']].forEach(([rot, col], k) => {
    const P = th => { const ex = Math.cos(th) * 15, ey = Math.sin(th) * 5.5; return [18 + ex * Math.cos(rot) - ey * Math.sin(rot), 18 + ex * Math.sin(rot) + ey * Math.cos(rot)]; };
    for (let i = 0; i < 48; i += 2) { const [X, Y] = P(i / 48 * 6.283); px(X, Y, col); }
    const [X, Y] = P(a * (1.6 + k * 0.4) + k * 2); x.fillStyle = '#fff'; x.fillRect(Math.round(X) - 1, Math.round(Y) - 1, 2, 2);
  });
  const R = 4.5; for (let dy = -5; dy <= 5; dy++) for (let dx = -5; dx <= 5; dx++) { const d = (dx * dx + dy * dy) / (R * R); if (d > 1) continue; const l = (-dx * 0.55 - dy * 0.6) / R + Math.sqrt(1 - d) * 0.6; px(18 + dx, 18 + dy, l > 0.85 ? '#ffffff' : l > 0.5 ? '#9ab8ff' : l > 0.15 ? '#3466f0' : '#1d3492'); }
  setTimeout(() => requestAnimationFrame(draw), 66);
}
requestAnimationFrame(draw);
let down = null, dragging = false;
addEventListener('pointerdown', e => { if (e.target.id === 'x') return; down = [e.screenX, e.screenY]; dragging = false; });
addEventListener('pointermove', e => { if (down && !dragging && Math.hypot(e.screenX - down[0], e.screenY - down[1]) > 4) { dragging = true; window.ipc.postMessage('drag'); } });
addEventListener('pointerup', e => { if (e.target.id === 'x') { window.ipc.postMessage('quit'); return; } if (down && !dragging) window.ipc.postMessage('restore'); down = null; });
addEventListener('contextmenu', e => e.preventDefault());
window.setE = s => { const [e, r] = s.split('|'); const el = document.getElementById('e'); el.textContent = e; el.title = (r || '') + ' energy per second'; };
</script></body></html>"##;

// opens the atom in the bottom-right corner of the screen the game is on, just above the taskbar
fn open_mini(target: &tao::event_loop::EventLoopWindowTarget<UserEvent>, main: &Window, ctx: &mut WebContext, proxy: tao::event_loop::EventLoopProxy<UserEvent>) -> Option<(Window, WebView)> {
    let b = WindowBuilder::new()
        .with_title("Ten Cells")
        .with_inner_size(LogicalSize::new(96.0, 104.0))
        .with_decorations(false)
        .with_transparent(true)
        .with_resizable(false)
        .with_always_on_top(true)
        .with_window_icon(Icon::from_rgba(ICON.to_vec(), 64, 64).ok());
    #[cfg(windows)]
    let b = b.with_skip_taskbar(true);
    let w = b.build(target).ok()?;
    if let Some(m) = main.current_monitor().or_else(|| w.primary_monitor()) {
        let (mp, ms, sz, k) = (m.position(), m.size(), w.outer_size(), m.scale_factor());
        w.set_outer_position(PhysicalPosition::new(mp.x + ms.width as i32 - sz.width as i32 - (16.0 * k) as i32, mp.y + ms.height as i32 - sz.height as i32 - (64.0 * k) as i32));
    }
    let b = WebViewBuilder::new_with_web_context(ctx)
        .with_html(MINI_HTML)
        .with_transparent(true)
        .with_devtools(false)
        .with_ipc_handler(move |req| { let _ = proxy.send_event(match req.body().as_str() { "drag" => UserEvent::Drag, "quit" => UserEvent::Quit, _ => UserEvent::Restore }); });
    #[cfg(windows)]
    let b = b.with_browser_accelerator_keys(false).with_default_context_menus(false).with_additional_browser_args(BROWSER_ARGS);
    let v = b.build(&w).ok()?;
    Some((w, v))
}

fn app_data() -> PathBuf {
    std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")).join("Ten Cells")
}

// The old Windows app (Electron) kept its save in Chromium's localStorage files. Pull out every copy of the
// save we can find, newest first; the page keeps the first one whose checksum matches, and only if it has no save yet.
fn old_saves() -> Vec<String> {
    let dir = match std::env::var_os("APPDATA") { Some(a) => PathBuf::from(a).join("Infinity Engine").join("Local Storage").join("leveldb"), None => return vec![] };
    let mut files: Vec<(std::time::SystemTime, PathBuf)> = fs::read_dir(&dir).into_iter().flatten().flatten()
        .filter(|e| e.path().extension().map_or(false, |x| x == "log" || x == "ldb"))
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path()))).collect();
    files.sort_by(|a, b| b.0.cmp(&a.0));
    let mut out = vec![];
    for (_, f) in files {
        let Ok(raw) = fs::read(&f) else { continue };
        // values are stored either as Latin-1 bytes or as UTF-16LE
        let latin: String = raw.iter().map(|&b| b as char).collect();
        let utf16: String = String::from_utf16_lossy(&raw.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect::<Vec<_>>());
        let utf16b: String = if raw.len() > 1 { String::from_utf16_lossy(&raw[1..].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect::<Vec<_>>()) } else { String::new() };
        for text in [&latin, &utf16, &utf16b] {
            let mut found: Vec<String> = vec![];
            let mut from = 0;
            while let Some(i) = text[from..].find("{\"h\":").map(|i| i + from) {
                if let Some(end) = json_end(&text[i..]) { found.push(text[i..i + end].to_string()); }
                from = i + 5;
            }
            found.reverse(); // the last write in a file is the newest
            out.extend(found);
            if out.len() >= 12 { return out; }
        }
    }
    out
}

// length of the JSON object at the start of s (string-aware brace matching), if it closes
fn json_end(s: &str) -> Option<usize> {
    let (mut depth, mut in_str, mut esc) = (0i32, false, false);
    for (i, c) in s.char_indices() {
        if i > 4_000_000 { return None; }
        if in_str { if esc { esc = false } else if c == '\\' { esc = true } else if c == '"' { in_str = false } continue; }
        match c { '"' => in_str = true, '{' => depth += 1, '}' => { depth -= 1; if depth == 0 { return Some(i + 1); } } _ => {} }
    }
    None
}

fn js_string(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""), '\\' => o.push_str("\\\\"), '\n' => o.push_str("\\n"), '\r' => o.push_str("\\r"),
            '\u{2028}' => o.push_str("\\u2028"), '\u{2029}' => o.push_str("\\u2029"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn init_script(saves: &[String]) -> String {
    let list = saves.iter().map(|s| js_string(s)).collect::<Vec<_>>().join(",");
    format!(r#"(() => {{
  if (location.hostname !== '{host}') return;
  // behave like an app, not a browser tab
  addEventListener('contextmenu', e => {{ if (!e.target.closest('input, textarea')) e.preventDefault(); }});
  addEventListener('keydown', e => {{ if (e.key === 'F11') {{ e.preventDefault(); window.ipc.postMessage('fullscreen'); }} }});
  addEventListener('click', e => {{ const a = e.target.closest && e.target.closest('a[href^="http"]'); if (a) {{ e.preventDefault(); window.ipc.postMessage('open:' + a.href); }} }}, true);
  try {{
    const K = 'infinityEngine', C = [{list}];
    if (C.length && !localStorage.getItem(K)) {{
      const h = s => {{ let h = 7; for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) | 0; return h; }};
      for (const c of C) {{ try {{ const o = JSON.parse(c); if (o && typeof o.d === 'string' && h(o.d) === o.h) {{ localStorage.setItem(K, c); break; }} }} catch (e) {{}} }}
    }}
  }} catch (e) {{}}
}})();"#, host = ORIGIN_HOST, list = list)
}

fn open_external(url: &str) {
    if !(url.starts_with("https://") || url.starts_with("http://")) { return; }
    #[cfg(windows)]
    { let _ = std::process::Command::new("rundll32").args(["url.dll,FileProtocolHandler", url]).spawn(); }
}

fn fail(msg: &str) -> ! {
    #[cfg(windows)]
    {
        // a plain message box, so a missing WebView2 runtime isn't a silent crash
        let text: Vec<u16> = msg.encode_utf16().chain(Some(0)).collect();
        let title: Vec<u16> = "Ten Cells".encode_utf16().chain(Some(0)).collect();
        #[link(name = "user32")]
        extern "system" { fn MessageBoxW(h: *mut core::ffi::c_void, t: *const u16, c: *const u16, f: u32) -> i32; }
        unsafe { MessageBoxW(core::ptr::null_mut(), text.as_ptr(), title.as_ptr(), 0x10); }
    }
    eprintln!("{msg}");
    std::process::exit(1)
}

fn main() {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let window = WindowBuilder::new()
        .with_title("Ten Cells")
        .with_inner_size(LogicalSize::new(600.0, 960.0))
        .with_min_inner_size(LogicalSize::new(340.0, 560.0))
        .with_window_icon(Icon::from_rgba(ICON.to_vec(), 64, 64).ok())
        .with_background_color((3, 4, 11, 255))
        .build(&event_loop)
        .unwrap_or_else(|e| fail(&format!("Couldn't open a window: {e}")));

    let data = app_data();
    let _ = fs::create_dir_all(&data);
    let mut ctx = WebContext::new(Some(data.join("WebView2")));
    let saves = old_saves();

    let p_ipc = proxy.clone();
    let builder = WebViewBuilder::new_with_web_context(&mut ctx)
        .with_custom_protocol("tencells".into(), |_id, req| {
            // the game is one file; anything else it asks for (like version.json) isn't here
            if matches!(req.uri().path(), "/" | "" | "/index.html") {
                Response::builder().header("Content-Type", "text/html; charset=utf-8").header("Cache-Control", "no-store").body(Cow::Borrowed(GAME)).unwrap()
            } else {
                Response::builder().status(404).body(Cow::Borrowed(&b""[..])).unwrap()
            }
        })
        .with_url("tencells://localhost/")
        .with_initialization_script(&init_script(&saves))
        .with_background_color((3, 4, 11, 255))
        .with_devtools(false)
        .with_hotkeys_zoom(false)
        .with_ipc_handler(move |req| {
            let m = req.body();
            if m == "fullscreen" { let _ = p_ipc.send_event(UserEvent::Fullscreen); }
            else if m == "collapse" { let _ = p_ipc.send_event(UserEvent::Collapse); }
            else if let Some(u) = m.strip_prefix("open:") { let _ = p_ipc.send_event(UserEvent::Open(u.to_string())); }
            else if let Some(t) = m.strip_prefix("mini:") { let _ = p_ipc.send_event(UserEvent::Mini(t.to_string())); }
            else if let Some(sz) = m.strip_prefix("size:") {
                let mut it = sz.split('x').filter_map(|v| v.parse::<f64>().ok());
                if let (Some(w), Some(h)) = (it.next(), it.next()) { let _ = p_ipc.send_event(UserEvent::Size(w.clamp(340.0, 4000.0), h.clamp(560.0, 4000.0))); }
            }
        })
        // anything that tries to leave the game opens in the real browser instead
        .with_navigation_handler(|url| { let ok = url.contains(ORIGIN_HOST) || url.starts_with("tencells:") || url.starts_with("about:"); if !ok { open_external(&url); } ok })
        .with_new_window_req_handler(|url, _| { open_external(&url); wry::NewWindowResponse::Deny });
    #[cfg(windows)]
    let builder = builder
        .with_browser_accelerator_keys(false) // no F5 reload, Ctrl+F find, Ctrl+P print
        .with_default_context_menus(false)
        // keep wry's defaults and let sounds play before the first click, like the old app
        .with_additional_browser_args(BROWSER_ARGS);
    let webview = builder.build(&window).unwrap_or_else(|e| fail(&format!(
        "Ten Cells needs Microsoft Edge WebView2, which comes with Windows 10 and 11.\n\nIt looks like it's missing on this PC. Install it from https://go.microsoft.com/fwlink/p/?LinkId=2124703 and start Ten Cells again.\n\n({e})")));
    let mut webview = Some(webview);
    let mut closing: Option<Instant> = None;
    let mut mini: Option<(Window, WebView)> = None;

    event_loop.run(move |event, target, control_flow| {
        *control_flow = ControlFlow::Wait;
        let restore = |mini: &mut Option<(Window, WebView)>, webview: &Option<WebView>| {
            mini.take();
            if let Some(w) = webview { let _ = w.set_visible(true); let _ = w.evaluate_script("window.__mini = false"); }
            window.set_visible(true); window.set_minimized(false); window.set_focus();
        };
        let start_close = |webview: &Option<WebView>, closing: &mut Option<Instant>| {
            // save one last time, give the page a moment to write it, then close
            if closing.is_none() {
                if let Some(w) = webview { let _ = w.evaluate_script("try { save() } catch (e) {}"); }
                *closing = Some(Instant::now());
            }
        };
        match event {
            Event::WindowEvent { window_id, event: WindowEvent::CloseRequested, .. } => {
                if mini.as_ref().map_or(false, |(w, _)| w.id() == window_id) { restore(&mut mini, &webview); }
                else { start_close(&webview, &mut closing); }
            }
            Event::UserEvent(UserEvent::Fullscreen) => {
                window.set_fullscreen(if window.fullscreen().is_some() { None } else { Some(Fullscreen::Borderless(None)) });
            }
            Event::UserEvent(UserEvent::Open(u)) => open_external(&u),
            Event::UserEvent(UserEvent::Size(w, h)) => { window.set_fullscreen(None); window.set_inner_size(LogicalSize::new(w, h)); }
            Event::UserEvent(UserEvent::Collapse) => {
                if mini.is_none() {
                    // hiding the page stops all drawing; its game logic keeps running and earning
                    if let Some(w) = &webview { let _ = w.set_visible(false); }
                    window.set_visible(false);
                    mini = open_mini(target, &window, &mut ctx, proxy.clone());
                    if mini.is_none() { restore(&mut mini, &webview); }
                }
            }
            Event::UserEvent(UserEvent::Mini(t)) => { if let Some((_, v)) = &mini { let _ = v.evaluate_script(&format!("window.setE && setE({})", js_string(&t))); } }
            Event::UserEvent(UserEvent::Restore) => restore(&mut mini, &webview),
            Event::UserEvent(UserEvent::Drag) => { if let Some((w, _)) = &mini { let _ = w.drag_window(); } }
            Event::UserEvent(UserEvent::Quit) => { mini.take(); start_close(&webview, &mut closing); }
            _ => {}
        }
        if let Some(t) = closing {
            if t.elapsed() >= Duration::from_millis(400) { mini.take(); webview.take(); *control_flow = ControlFlow::Exit; }
            else { *control_flow = ControlFlow::WaitUntil(t + Duration::from_millis(400)); }
        }
    });
}
