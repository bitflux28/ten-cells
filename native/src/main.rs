// Ten Cells: the native build. No browser inside: the game logic, drawing and sound are all Rust.
#![cfg_attr(windows, windows_subsystem = "windows")]
mod assets; mod big; mod data; mod fmt; mod game; mod gfx; mod save; mod screens; mod sound; mod ui;
use macroquad::prelude::*;

fn conf() -> Conf {
    Conf { window_title: "Ten Cells".into(), window_width: 460, window_height: 820, high_dpi: true, window_resizable: true, sample_count: 1,
        icon: Some(miniquad::conf::Icon { small: icon(16), medium: icon(32), big: icon(64) }), ..Default::default() }
}
/// the app icon, from the 512px web icon
fn icon<const N: usize>(n: usize) -> [u8; N] {
    let img = image_from(crate::assets::get("icon512.png").unwrap()); let mut o = [0u8; N];
    for y in 0..n { for x in 0..n { let (sx, sy) = (x * img.0 / n, y * img.0 / n); let i = (sy * img.0 + sx) * 4; o[(y * n + x) * 4..(y * n + x) * 4 + 4].copy_from_slice(&img.1[i..i + 4]) } } o
}
fn image_from(b: &[u8]) -> (usize, Vec<u8>) { let i = Image::from_file_with_format(b, Some(ImageFormat::Png)).unwrap(); (i.width as usize, i.bytes) }

#[macroquad::main(conf)]
async fn main() {
    game::seed_rng(game::now_ms() as u64);
    let args: Vec<String> = std::env::args().collect();
    // test hooks: TC_IMPORT=<file with save code>, TC_SHOT=<png> TC_TAB=<tab[:sub]> TC_FRAMES=<n>
    let (state, bak, corrupt) = match std::env::var("TC_IMPORT") { Ok(f) => (save::import(&std::fs::read_to_string(f).unwrap()).ok(), false, false), Err(_) => save::load() };
    let mut g = game::Game::new(state.unwrap_or_default());
    let away = g.offline();
    let mut app = ui::App::new(g);
    if let Some((away, used, cap, gained)) = away { if away > 60.0 { app.sheet = ui::Sheet::Welcome(format!("You were away for {}. The Engine kept running for {}{} and made {} energy.", fmt::fmt_t(away), fmt::fmt_t(used), if away > cap { format!(" (your offline limit is {})", fmt::fmt_t(cap)) } else { String::new() }, fmt::fmt(gained))) } }
    if bak { app.toast("Loaded your save from the backup or the older Windows app", "check", gfx::c("gold")) } else if corrupt { app.toast("A damaged save was found and skipped", "cross", gfx::c("red")) }
    let shot = std::env::var("TC_SHOT").ok(); let frames: usize = std::env::var("TC_FRAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(40);
    if let Ok(t) = std::env::var("TC_TAB") { let (t, s) = t.split_once(':').map(|(a, b)| (a.to_string(), Some(b.to_string()))).unwrap_or((t, None)); app.tab = t.clone(); if let Some(s) = s { app.set_sub(&t, &s) } if shot.is_some() { app.sheet = ui::Sheet::None } }
    if shot.is_none() { sound::init(&app.g.s.set).await; }
    let _ = args;
    let mut n = 0;
    loop {
        app.frame();
        n += 1;
        if let Some(p) = &shot { if n == frames { get_screen_data().export_png(p); break } }
        if is_quit_requested() { app.save(); break }
        next_frame().await;
    }
    app.save();
}
