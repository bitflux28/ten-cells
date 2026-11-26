// The app shell: input, layout (a centred column like the web version), the header with energy and currencies,
// the bottom tabs, notifications, pop-up sheets, the sky, saving and the main loop.
use crate::big::{b, Big};
use crate::data::{data, NG};
use crate::fmt::{fmt, fmt_t};
use crate::game::{now_ms, Game};
use crate::gfx::{alpha, c, hex, Gfx, INK};
use macroquad::prelude::*;

#[derive(Clone)] pub struct Toast { pub text: String, pub spr: String, pub col: Color, pub t: f64 }
#[derive(Clone)] pub struct Float { pub x: f32, pub y: f32, pub text: String, pub crit: bool, pub t: f64 }
#[derive(Clone)] pub struct Flash { pub title: String, pub sub: String, pub col: Color, pub t: f64 }
#[derive(Clone, PartialEq)] pub enum Sheet { None, Confirm(String, String, String, String), Info(String, String), Gen(usize), Upg(String), Export(String), Import, Welcome(String), WhatsNew, Terms, Scene(usize, String, usize), Prestige(String), Reset, Keys, Shortcut }

pub struct App {
    pub g: Game, pub gfx: Gfx, pub spr_img: Image,
    pub tab: String, pub sub: std::collections::HashMap<String, String>, pub scroll: std::collections::HashMap<String, f32>, pub sheet: Sheet, pub sheet_scroll: f32,
    pub toasts: Vec<Toast>, pub floats: Vec<Float>, pub flash: Option<Flash>, pub log: Vec<(f64, String, String)>, pub hist: Vec<(f64, f64)>,
    pub buy: f64, pub cat: String, pub dim_sel: Option<usize>, pub rx_a: f64, pub rx_hit: f64, pub rings: Vec<(f64, bool)>, pub spin_b: f64,
    // input
    pub mx: f32, pub my: f32, pub down: bool, pub pressed: bool, pub click: bool, pub drag: f32, press_y: f32, press_scroll: f32, pub dragging: bool, pub clip: Rect, hold_t: f64,
    pub cw: f32, pub x0: f32, pub lw: f32, pub lh: f32, pub time: f64, pub input: String, pub input_key: String,
    save_t: f64, hist_t: f64, pub prestige_fx: f64, last_fx: f64, pub dlg: Option<(usize, String, usize)>, pub full: bool,
}
pub const NAV: [(&str, &str, &str); 6] = [("engine", "Engine", "core"), ("upgrades", "Upgrades", "tree"), ("prestige", "Prestige", "star"), ("worlds", "Worlds", "planet"), ("crew", "Crew", "person"), ("stats", "Stats", "bars")];

impl App {
    pub fn new(g: Game) -> App {
        let gfx = Gfx::new(); let spr_img = Image::from_file_with_format(crate::assets::get("sprites.png").unwrap(), Some(ImageFormat::Png)).unwrap();
        let mut sub = std::collections::HashMap::new(); for (k, v) in [("upgrades", "tree"), ("prestige", "reb"), ("worlds", "dims"), ("crew", "story"), ("stats", "stats")] { sub.insert(k.to_string(), v.to_string()); }
        let mut a = App { g, gfx, spr_img, tab: "engine".into(), sub, scroll: Default::default(), sheet: Sheet::None, sheet_scroll: 0.0, toasts: vec![], floats: vec![], flash: None, log: vec![], hist: vec![],
            buy: 1.0, cat: "gen".into(), dim_sel: None, rx_a: 0.0, rx_hit: 0.0, rings: vec![], spin_b: 0.0, mx: 0.0, my: 0.0, down: false, pressed: false, click: false, drag: 0.0, press_y: 0.0, press_scroll: 0.0, dragging: false,
            clip: Rect::new(0.0, 0.0, 9999.0, 9999.0), hold_t: 0.0, cw: 390.0, x0: 0.0, lw: 390.0, lh: 844.0, time: 0.0, input: String::new(), input_key: String::new(), save_t: 0.0, hist_t: 0.0, prestige_fx: 0.0, last_fx: 0.0, dlg: None, full: false };
        crate::fmt::set_notation(&a.g.s.set.not);
        if a.g.s.set.tos == 0.0 { a.sheet = Sheet::Terms }
        a
    }
    pub fn sub(&self, t: &str) -> String { self.sub.get(t).cloned().unwrap_or_default() }
    pub fn set_sub(&mut self, t: &str, v: &str) { self.sub.insert(t.into(), v.into()); }
    pub fn toast(&mut self, text: impl Into<String>, spr: &str, col: Color) { let text = text.into(); self.log.push((now_ms(), text.clone(), spr.into())); self.toasts.push(Toast { text, spr: spr.into(), col, t: 0.0 }); if self.toasts.len() > 4 { self.toasts.remove(0); } }
    pub fn hover(&self, r: Rect) -> bool { r.contains(vec2(self.mx, self.my)) && self.clip.contains(vec2(self.mx, self.my)) }
    /// a tap inside r (not a scroll drag)
    pub fn hit(&mut self, r: Rect) -> bool { if self.click && self.hover(r) { self.click = false; true } else { false } }
    // ---------------- widgets ----------------
    /// style: "" normal, "on" blue, "gold", "ghost", "danger", "green"; disabled when !ok
    pub fn btn(&mut self, x: f32, y: f32, w: f32, h: f32, label: &str, style: &str, ok: bool) -> bool {
        let r = Rect::new(x, y, w, h); let pressed = ok && self.down && self.hover(r);
        let (fill, lo, hi, tc) = if !ok { (c("p2"), c("p1"), c("p3"), c("faint")) } else { match style {
            "on" => (c("blue"), hex("#1d3492"), hex("#9ab8ff"), WHITE), "gold" => (hex("#ffcc4d"), hex("#a8661a"), hex("#fff1b0"), INK), "green" => (hex("#17543f"), hex("#0f3a2b"), hex("#2a8a66"), c("green")),
            "danger" => (c("red"), hex("#8f1d3c"), hex("#ffb3c3"), WHITE), "ghost" => (c("p2"), hex("#191545"), c("p3"), c("mist")), "vio" => (c("vio"), hex("#4a35a8"), hex("#b49bff"), WHITE), _ => (c("p3"), c("p2"), c("hi"), c("mist")) } };
        self.gfx.button(x, y, w, h, fill, lo, hi, pressed);
        let lines: Vec<&str> = label.split('\n').collect(); let n = lines.len() as f32; let sz = if h < 36.0 { 13.0 } else { 14.0 };
        for (i, l) in lines.iter().enumerate() { let yy = y + h / 2.0 - n * 9.0 + i as f32 * 18.0 + if pressed { 2.0 } else { 0.0 }; let f = if i == 0 { 1 } else { 0 }; if tc == WHITE { self.gfx.text_c(l, x + w / 2.0 + 1.0, yy + 1.0, sz, INK, f) } self.gfx.text_c(l, x + w / 2.0, yy, sz, tc, f) }
        ok && self.hit(r)
    }
    /// sub-tab chips; returns the picked key
    pub fn chips(&mut self, y: f32, list: &[(&str, String, bool)], cur: &str) -> (f32, Option<String>) {
        let (mut x, mut yy) = (self.x0 + 10.0, y); let mut pick = None;
        for (k, n, lock) in list {
            let w = self.gfx.tw(n, 14.0, 0) + 22.0 + if *lock { 16.0 } else { 0.0 };
            if x + w > self.x0 + self.cw - 10.0 { x = self.x0 + 10.0; yy += 36.0 }
            let sel = cur == *k; self.gfx.rect(x, yy, w, 30.0, if sel { c("vio") } else { c("p2") }); self.gfx.rect(x, yy + 27.0, w, 3.0, if sel { hex("#4a35a8") } else { hex("#191545") }); if sel { self.gfx.rect(x, yy, w, 3.0, hex("#b49bff")) }
            let mut tx = x + 11.0; if *lock { self.gfx.spr("lock", tx - 2.0, yy + 9.0, 12.0, WHITE); tx += 14.0 }
            if sel { self.gfx.text_sh(n, tx, yy + 7.0, 14.0, WHITE, 0); } else { self.gfx.text(n, tx, yy + 7.0, 14.0, c("mist"), 0); }
            if self.hit(Rect::new(x, yy, w, 30.0)) { pick = Some(k.to_string()) }
            x += w + 6.0;
        }
        (yy + 40.0 - y, pick)
    }
    pub fn toggle(&mut self, x: f32, y: f32, on: bool) -> bool {
        let r = Rect::new(x, y, 46.0, 24.0); self.gfx.rect(x, y, 46.0, 24.0, INK); self.gfx.rect(x + 2.0, y + 2.0, 42.0, 20.0, if on { hex("#17543f") } else { c("p2") });
        self.gfx.rect(if on { x + 24.0 } else { x + 4.0 }, y + 4.0, 18.0, 16.0, if on { c("green") } else { c("faint") }); self.hit(r)
    }
    pub fn panel(&self, y: f32, h: f32) { self.gfx.panel(self.x0 + 8.0, y, self.cw - 16.0, h, c("p1")) }
    pub fn input_field(&mut self, key: &str, x: f32, y: f32, w: f32, val: &str) -> Option<String> {
        let active = self.input_key == key; let r = Rect::new(x, y, w, 30.0);
        self.gfx.rect(x, y, w, 30.0, INK); self.gfx.rect(x + 2.0, y + 2.0, w - 4.0, 26.0, if active { c("p3") } else { c("p2") });
        let shown = if active { format!("{}{}", self.input, if (self.time * 2.0) as i64 % 2 == 0 { "_" } else { " " }) } else { val.to_string() };
        self.gfx.text(&shown, x + 8.0, y + 8.0, 14.0, WHITE, 2);
        if self.hit(r) { self.input_key = key.into(); self.input = val.into(); }
        if active && (is_key_pressed(KeyCode::Enter) || (self.click && !self.hover(r))) { self.input_key.clear(); return Some(self.input.clone()) }
        None
    }
    pub fn spr(&self, name: &str, x: f32, y: f32, size: f32) { let k = self.gfx.scale; self.gfx.spr(name, x * k, y * k, size * k, WHITE) }
    pub fn model(&mut self, name: &str, x: f32, y: f32, size: f32) { let k = self.gfx.scale; let t = self.time; self.gfx.model(name, x * k, y * k, size * k, t, WHITE) }
    pub fn world(&mut self, id: &str, x: f32, y: f32, size: f32, locked: bool) { let k = self.gfx.scale; let t = self.time; self.gfx.world(id, x * k, y * k, size * k, t, locked) }

    // ---------------- per-frame ----------------
    pub fn layout(&mut self) {
        let (w, h) = (screen_width(), screen_height());
        let auto = (w / 420.0).min(h / 860.0).clamp(1.0, 3.0);
        let k = match self.g.s.set.uiz.as_f64() { Some(z) => (z / 100.0) as f32, None => auto }; self.gfx.scale = k.max(0.75);
        self.lw = w / self.gfx.scale; self.lh = h / self.gfx.scale; self.cw = self.lw.min(560.0); self.x0 = ((self.lw - self.cw) / 2.0).floor();
    }
    pub fn read_input(&mut self) {
        let (mx, my) = mouse_position(); self.mx = mx / self.gfx.scale; self.my = my / self.gfx.scale;
        let touch = touches(); let now_down = is_mouse_button_down(MouseButton::Left) || !touch.is_empty();
        self.pressed = now_down && !self.down; self.click = false;
        if self.pressed { self.press_y = self.my; self.press_scroll = *self.scroll.get(&self.scroll_key()).unwrap_or(&0.0); self.dragging = false; self.drag = 0.0; self.hold_t = 0.0 }
        if now_down { self.drag = self.my - self.press_y; if self.drag.abs() > 8.0 { self.dragging = true } self.hold_t += get_frame_time() as f64 }
        if !now_down && self.down && !self.dragging { self.click = true }
        self.down = now_down;
        // typing into a text field
        if !self.input_key.is_empty() { while let Some(ch) = get_char_pressed() { if !ch.is_control() && self.input.len() < 4000 { self.input.push(ch) } } if is_key_pressed(KeyCode::Backspace) { self.input.pop(); } if is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::V) { if let Some(t) = miniquad::window::clipboard_get() { self.input.push_str(t.trim()) } } }
    }
    pub fn scroll_key(&self) -> String { if self.sheet != Sheet::None { "sheet".into() } else { format!("{}:{}", self.tab, self.sub(&self.tab)) } }

    pub fn frame(&mut self) {
        let dt = get_frame_time().min(0.25) as f64; self.time += dt;
        self.layout(); self.read_input();
        // ---- game time
        self.g.tick(dt, false);
        self.rx_a += 0.016 * dt * 60.0 * (1.0 + (crate::game::oom(self.g.c.eps) / 25.0).min(4.0)) * (1.0 + self.spin_b) * if self.g.s.set.fx > 0.0 { 1.0 } else { 0.0 };
        self.rx_hit *= 0.8f64.powf(dt * 60.0); self.spin_b *= 0.9f64.powf(dt * 60.0);
        for r in self.rings.iter_mut() { r.0 += 1.6 * dt * 60.0 } self.rings.retain(|r| r.0 < 50.0);
        self.drain_notes();
        for t in self.g.toasts.drain(..).collect::<Vec<_>>() { self.toast(t.0, &t.1, hex(&t.2)) }
        self.save_t += dt; if self.save_t > 20.0 { self.save_t = 0.0; self.save(); }
        self.hist_t += dt; if self.hist_t > 60.0 || self.hist.is_empty() { self.hist_t = 0.0; self.hist.push((now_ms(), self.g.c.eps.log10().max(0.0))); if self.hist.len() > 120 { self.hist.remove(0); } }
        if self.g.s.set.not != self.last_not() { crate::fmt::set_notation(&self.g.s.set.not) }
        self.keys();
        // ---- draw
        clear_background(c("bg"));
        self.draw_sky();
        let head_h = self.header();
        let nav_h = 64.0; let top = head_h; let bot = self.lh - nav_h;
        let key = self.scroll_key();
        let sc = *self.scroll.get(&key).unwrap_or(&0.0);
        self.clip = Rect::new(0.0, top, self.lw, bot - top);
        let k = self.gfx.scale;
        unsafe { get_internal_gl().quad_gl.scissor(Some(((0.0) as i32, (top * k) as i32, (self.lw * k) as i32, ((bot - top) * k) as i32))); }
        let content_h = crate::screens::draw(self, top + 6.0 - sc);
        unsafe { get_internal_gl().quad_gl.scissor(None); }
        self.clip = Rect::new(0.0, 0.0, self.lw, self.lh);
        // scrolling: wheel, drag
        if self.sheet == Sheet::None {
            let max = (content_h + 20.0 - (bot - top)).max(0.0); let mut s = sc;
            let (_, wy) = mouse_wheel(); if wy != 0.0 { s -= wy.signum() * 60.0 }
            if self.down && self.dragging && self.press_y > top && self.press_y < bot { s = self.press_scroll - self.drag }
            self.scroll.insert(key, s.clamp(0.0, max));
        }
        self.nav(bot);
        self.draw_floats(dt);
        self.draw_toasts(dt, top);
        if let Some(f) = self.flash.as_mut() { f.t += dt; } self.draw_flash();
        if self.prestige_fx > 0.0 { self.prestige_fx -= dt; let a = (self.prestige_fx / 1.2).clamp(0.0, 1.0) as f32; self.gfx.rect(0.0, 0.0, self.lw, self.lh, alpha(WHITE, a * a)); }
        if self.sheet != Sheet::None { crate::screens::sheet(self) }
        if let Some(_) = self.dlg { crate::screens::dialogue(self) }
    }
    fn last_not(&self) -> String { match crate::fmt::NOTATION.load(std::sync::atomic::Ordering::Relaxed) { 1 => "sci".into(), 2 => "eng".into(), _ => "suffix".into() } }
    pub fn save(&mut self) { self.g.s.last = now_ms(); if let Err(e) = crate::save::save(&self.g.s) { self.toast(format!("Saving failed: {e}"), "cross", c("red")) } }

    fn keys(&mut self) {
        if !self.input_key.is_empty() { return }
        if is_key_pressed(KeyCode::Space) { self.tap(self.x0 + self.cw / 2.0, 300.0) }
        for (i, kc) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5, KeyCode::Key6].iter().enumerate() { if is_key_pressed(*kc) { self.tab = NAV[i].0.into(); self.sheet = Sheet::None } }
        if is_key_pressed(KeyCode::G) { let n = self.g.buy_everything(); if n > 0.0 { self.toast(format!("Bought {} energy makers", crate::fmt::group(n as i64)), "gear", c("cyan")) } }
        if is_key_pressed(KeyCode::U) { let n = self.g.buy_all_upg(); if n > 0 { self.toast(format!("Bought {n} upgrade{}", if n > 1 { "s" } else { "" }), "arrowup", c("cyan")) } }
        if is_key_pressed(KeyCode::A) { for id in self.g.fx.ab.clone() { self.g.use_ab(&id); } }
        if is_key_pressed(KeyCode::Escape) { self.sheet = Sheet::None; self.dlg = None }
        if is_key_pressed(KeyCode::F11) || is_key_pressed(KeyCode::F) { self.full = !self.full; set_fullscreen(self.full) }
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) { self.zoom(10.0) } if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) { self.zoom(-10.0) }
    }
    pub fn zoom(&mut self, d: f64) { let cur = self.g.s.set.uiz.as_f64().unwrap_or((self.gfx.scale * 100.0) as f64); let n = (cur + d).clamp(75.0, 300.0); self.g.s.set.uiz = serde_json::json!(n); self.toast(format!("Screen size {}%", n), "arrows", c("cyan")) }
    pub fn tap(&mut self, x: f32, y: f32) {
        if let Some((v, crit)) = self.g.do_tap() {
            self.rx_hit = 1.0; self.rings.push((24.0, crit));
            self.floats.push(Float { x: x + (crate::game::rnd() as f32 - 0.5) * 30.0, y, text: format!("+{}", fmt(v)), crit, t: 0.0 }); if self.floats.len() > 24 { self.floats.remove(0); }
        }
    }
    fn drain_notes(&mut self) {
        let notes: Vec<_> = self.g.q.drain(..).collect();
        for n in notes {
            let (spr, col) = match n.kind.as_str() { "prestige" => ("star", c("cyan")), "event" => (data().events.iter().find(|e| e.k == n.extra).map(|_| "bolt").unwrap_or("bolt"), c("cyan")), "ach" => ("trophy", c("gold")), "badge" => ("trophy", c("gold")), "boss" => ("swords", c("red")),
                "art" => ("gem", c("vio")), "egg" => ("egg", c("mag")), "pet" => ("heart", c("mag")), "story" => ("scroll", c("gold")), "chal" => ("swords", c("green")), "bad" => ("cross", c("red")), "beat" => ("crown", c("gold")), _ => ("sparkle", c("cyan")) };
            if n.kind == "prestige" && !n.auto { self.prestige_fx = 1.2; let k = n.extra.clone(); let nm = crate::game::LAYERS.iter().find(|l| l.0 == k).map(|l| l.1).unwrap_or(""); self.flash = Some(Flash { title: nm.to_string(), sub: n.text.split('+').nth(1).map(|s| format!("+{s}")).unwrap_or_default(), col: c("cyan"), t: 0.0 }); }
            if matches!(n.kind.as_str(), "boss" | "story" | "beat") { self.flash = Some(Flash { title: n.text.split('!').next().unwrap_or("").split(':').next().unwrap_or("").to_string(), sub: n.text.clone(), col: col, t: 0.0 }) }
            self.toast(n.text, spr, col);
        }
    }
    // ---------------- the sky ----------------
    fn draw_sky(&mut self) {
        let id = self.g.top_dim().map(|i| data().dims[i].id.clone()).unwrap_or_else(|| "default".into());
        let k = self.gfx.scale; let px = 3.0 * k; let (w, h) = (screen_width(), screen_height());
        if let Some(t) = self.gfx.tex(&format!("sky_{id}.png")) {
            // cover the window with the 240×320 sky, art pixels 3px (scaled) like the web version
            let (sw, sh) = (240.0 * px, 320.0 * px); let s = (w / sw).max(h / sh).max(1.0);
            let drift = if self.g.s.set.fx > 0.0 { (self.time * 0.03).sin() as f32 * 4.0 * px } else { 0.0 };
            draw_texture_ex(&t, ((w - sw * s) / 2.0 + drift).round(), ((h - sh * s) / 2.0).round(), WHITE, DrawTextureParams { dest_size: Some(vec2(sw * s, sh * s)), ..Default::default() });
        }
        // twinkling stars on top
        if self.g.s.set.fx > 0.0 { for i in 0..70 { let hx = ((i as f64 * 127.1).sin() * 43758.5).fract().abs(); let hy = ((i as f64 * 311.7).sin() * 24634.6).fract().abs(); let ph = (self.time * (1.0 + (i % 5) as f64 * 0.4) + i as f64).sin(); if ph > 0.4 { let a = ((ph - 0.4) / 0.6) as f32; draw_rectangle((hx as f32 * w / px).floor() * px, (hy as f32 * h / px).floor() * px, px, px, alpha(if i % 4 == 0 { hex("#c4f3ff") } else { WHITE }, a)) } } }
    }
    // ---------------- header: energy, per second, currencies ----------------
    fn header(&mut self) -> f32 {
        let (x, w) = (self.x0 + 8.0, self.cw - 16.0); let g = &self.g; let s = &g.s;
        let mut curs: Vec<(&str, Big, &str)> = vec![];
        if s.frag_life.gtf(0.0) || s.cnt.reb > 0.0 { curs.push(("diamond", s.frag, "Fragments")) } if s.cnt.sing > 0.0 { curs.push(("hole", s.sing, "Singularities")) } if s.cnt.asc > 0.0 { curs.push(("tri", s.cores, "Ascension Cores")) }
        if s.cnt.tra > 0.0 { curs.push(("moon", s.ve, "Void Essence")) } if s.cnt.inf > 0.0 { curs.push(("inf", s.ip, "Infinity Points")) } if s.cnt.ult > 0.0 || s.up_total.gtf(0.0) { curs.push(("crown", s.up, "Ultra Points")) } if s.cnt.reb > 0.0 { curs.push(("flask", s.rp, "Research Points")) }
        let h = if curs.is_empty() { 96.0 } else { 136.0 };
        self.gfx.rect(x - 3.0, 6.0, w + 6.0, h + 3.0, INK); self.gfx.panel(x, 6.0, w, h, c("p1"));
        self.gfx.text("Ten Cells", x + 14.0, 16.0, 14.0, c("mist"), 0);
        let e = fmt(s.energy); self.gfx.text_sh(&e, x + 14.0, 34.0, 38.0, WHITE, 2);
        let eps = format!("{}", fmt(g.c.eps.mulf(g.c.sp))); let ew = self.gfx.text(&eps, x + 14.0, 76.0, 14.0, c("cyan"), 2);
        self.gfx.text(" energy per second", x + 14.0 + ew, 76.0, 14.0, c("mist"), 0);
        if g.capped() { self.gfx.text_r("INFINITY REACHED", x + w - 14.0, 76.0, 12.0, c("gold"), 1) }
        // mute
        let mr = Rect::new(x + w - 44.0, 12.0, 32.0, 28.0); let snd = s.set.sfx > 0.0 || s.set.mus > 0.0; self.gfx.spr(if snd { "speaker" } else { "mute" }, (mr.x + 4.0) * self.gfx.scale, (mr.y + 2.0) * self.gfx.scale, 24.0 * self.gfx.scale, WHITE);
        if self.hit(mr) { let on = !snd; self.g.s.set.sfx = if on { 1.0 } else { 0.0 }; self.g.s.set.mus = self.g.s.set.sfx; crate::sound::sync(&self.g.s.set); self.toast(if on { "Sound on" } else { "Sound off" }, if on { "speaker" } else { "mute" }, c("cyan")) }
        // currencies
        let mut cx = x + 12.0; let cy = 100.0;
        for (spr, v, n) in curs {
            let t = fmt(v); let cwid = self.gfx.tw(&t, 14.0, 2) + 34.0; if cx + cwid > x + w - 8.0 { break }
            self.gfx.rect(cx, cy, cwid, 30.0, INK); self.gfx.rect(cx + 2.0, cy + 2.0, cwid - 4.0, 26.0, c("p2")); self.spr(spr, cx + 5.0, cy + 6.0, 18.0); self.gfx.text(&t, cx + 27.0, cy + 9.0, 14.0, WHITE, 2);
            if self.hit(Rect::new(cx, cy, cwid, 30.0)) { self.toast(format!("{}: {}", n, fmt(v)), spr, c("cyan")) }
            cx += cwid + 6.0;
        }
        // active buffs line
        let mut by = h + 12.0;
        let buffs: Vec<_> = self.g.s.ev.buffs.clone();
        if !buffs.is_empty() || self.g.s.boss.f.is_some() {
            let mut bx = x;
            for bf in &buffs { let nm = match bf.k.as_str() { "prod" => "Production", "tap" => "Taps", "cost" => "Costs", _ => "Speed" }; let t = format!("{} ×{} · {}s", nm, if bf.m >= 10.0 { format!("{}", bf.m.round()) } else { format!("{}", (bf.m * 100.0).round() / 100.0) }, bf.tt.min(bf.t).ceil()); let bw = self.gfx.tw(&t, 12.0, 2) + 14.0; let col = match bf.k.as_str() { "prod" => c("cyan"), "tap" => c("gold"), "cost" => c("vio"), _ => c("mag") }; self.gfx.rect(bx, by, bw, 20.0, INK); self.gfx.rect(bx + 2.0, by + 2.0, (bw - 4.0) * (bf.t / bf.tt) as f32, 16.0, alpha(col, 0.35)); self.gfx.text(&t, bx + 7.0, by + 5.0, 12.0, col, 2); bx += bw + 4.0; if bx > x + w - 60.0 { break } }
            if let Some(f) = &self.g.s.boss.f { let t = format!("{} · {}s", data().bosses[f.i].n, f.t.ceil()); let bw = self.gfx.tw(&t, 12.0, 0) + 14.0; self.gfx.rect(bx, by, bw, 20.0, INK); self.gfx.text(&t, bx + 7.0, by + 4.0, 12.0, c("red"), 0); }
            by += 24.0;
        }
        by
    }
    fn nav(&mut self, y: f32) {
        let (x, w) = (self.x0, self.cw); self.gfx.rect(0.0, y, self.lw, self.lh - y, c("p1")); self.gfx.rect(0.0, y, self.lw, 3.0, INK); self.gfx.rect(0.0, y + 3.0, self.lw, 3.0, c("p3"));
        let bw = (w - 8.0) / NAV.len() as f32;
        for (i, (k, n, spr)) in NAV.iter().enumerate() {
            let bx = x + 4.0 + i as f32 * bw; let sel = self.tab == *k;
            if sel { self.gfx.rect(bx + 2.0, y + 8.0, bw - 4.0, 52.0, c("p3")); self.gfx.rect(bx + 2.0, y + 57.0, bw - 4.0, 3.0, c("cyan")) }
            self.spr(spr, bx + bw / 2.0 - 12.0, y + 12.0, 24.0);
            if sel { self.gfx.text_c(n, bx + bw / 2.0 + 1.0, y + 40.0, 12.0, INK, 0); self.gfx.text_c(n, bx + bw / 2.0, y + 39.0, 12.0, WHITE, 0) } else { self.gfx.text_c(n, bx + bw / 2.0, y + 39.0, 12.0, c("faint"), 0) }
            // a dot when something is waiting there
            let dot = match *k { "engine" => self.g.daily_ready() || self.g.s.ev.cur.is_some(), "crew" => (!self.g.story_done() && self.g.missions_done(self.g.s.story.ch as usize)) || self.g.s.pets.eggs.iter().any(|e| e.t <= 0.0), "prestige" => self.g.can_prestige("reb") && self.g.s.cnt.reb == 0.0, _ => false };
            if dot { self.gfx.rect(bx + bw / 2.0 + 10.0, y + 10.0, 7.0, 7.0, INK); self.gfx.rect(bx + bw / 2.0 + 11.0, y + 11.0, 5.0, 5.0, c("gold")) }
            if self.hit(Rect::new(bx, y + 4.0, bw, 58.0)) { if self.tab == *k { self.scroll.insert(self.scroll_key(), 0.0); } self.tab = k.to_string(); crate::sound::sfx("tab", &self.g.s.set) }
        }
    }
    fn draw_floats(&mut self, dt: f64) {
        for f in self.floats.iter_mut() { f.t += dt }
        self.floats.retain(|f| f.t < 0.9);
        for f in &self.floats { let p = (f.t / 0.9) as f32; let y = f.y - 78.0 * (1.0 - (1.0 - p).powi(3)); let a = if p > 0.4 { 1.0 - (p - 0.4) / 0.6 } else { 1.0 }; let sz = if f.crit { 24.0 } else { 18.0 }; let col = if f.crit { c("gold") } else { WHITE }; self.gfx.text_c(&f.text, f.x + 2.0, y + 2.0, sz, alpha(if f.crit { c("red") } else { c("blue") }, a), 1); self.gfx.text_c(&f.text, f.x, y, sz, alpha(col, a), 1) }
    }
    fn draw_toasts(&mut self, dt: f64, top: f32) {
        for t in self.toasts.iter_mut() { t.t += dt } self.toasts.retain(|t| t.t < 3.2);
        let mut y = top + 4.0;
        for t in self.toasts.clone() {
            let a = if t.t < 0.2 { (t.t / 0.2) as f32 } else if t.t > 2.8 { ((3.2 - t.t) / 0.4) as f32 } else { 1.0 };
            let maxw = self.cw - 40.0; let tw = self.gfx.tw(&t.text, 14.0, 0).min(maxw - 40.0); let h = self.gfx.wrap_h(&t.text, maxw - 40.0, 14.0, 0) + 12.0; let w = tw + 44.0; let x = self.x0 + (self.cw - w) / 2.0;
            self.gfx.rect(x - 3.0, y - 3.0, w + 6.0, h + 6.0, alpha(INK, a)); self.gfx.rect(x, y, w, h, alpha(c("p1"), a)); self.gfx.rect(x, y, w, 3.0, alpha(c("p3"), a)); self.gfx.rect(x, y + h - 3.0, w, 3.0, alpha(t.col, a));
            let k = self.gfx.scale; self.gfx.spr(&t.spr, (x + 8.0) * k, (y + h / 2.0 - 10.0) * k, 20.0 * k, alpha(WHITE, a));
            self.gfx.wrap(&t.text, x + 34.0, y + 6.0, maxw - 40.0, 14.0, alpha(c("txt"), a), 0);
            if self.hit(Rect::new(x, y, w, h)) { if let Some(tt) = self.toasts.iter_mut().find(|q| q.text == t.text) { tt.t = 3.1 } }
            y += h + 8.0;
        }
    }
    fn draw_flash(&mut self) {
        let Some(f) = self.flash.clone() else { return }; if f.t > 2.6 { self.flash = None; return }
        let a = if f.t < 0.25 { (f.t / 0.25) as f32 } else if f.t > 2.1 { ((2.6 - f.t) / 0.5) as f32 } else { 1.0 };
        let y = self.lh * 0.38; let w = self.cw - 30.0; let x = self.x0 + 15.0;
        self.gfx.rect(x - 3.0, y - 3.0, w + 6.0, 86.0, alpha(INK, a)); self.gfx.rect(x, y, w, 80.0, alpha(c("p1"), a)); self.gfx.rect(x, y, w, 4.0, alpha(f.col, a)); self.gfx.rect(x, y + 76.0, w, 4.0, alpha(f.col, a));
        self.gfx.text_c(&f.title, self.x0 + self.cw / 2.0 + 2.0, y + 14.0, 26.0, alpha(INK, a), 1); self.gfx.text_c(&f.title, self.x0 + self.cw / 2.0, y + 12.0, 26.0, alpha(f.col, a), 1);
        let sub: String = f.sub.chars().take(60).collect(); self.gfx.text_c(&sub, self.x0 + self.cw / 2.0, y + 50.0, 14.0, alpha(c("txt"), a), 0);
    }
    pub fn hold_secs(&self) -> f64 { if self.down { self.hold_t } else { 0.0 } }
    pub fn gen_hue(i: usize) -> Color { let h = (i as f32 * 37.0 + 190.0) % 360.0 / 360.0; macroquad::color::hsl_to_rgb(h, 0.8, 0.65) }
    pub fn atom_blit(&self) -> impl Fn(&mut Image, usize, i32, i32, f32) + '_ {
        move |img: &mut Image, gi: usize, x: i32, y: i32, a: f32| {
            let name = &data().gen_spr[gi.min(NG - 1)]; let idx = *self.gfx.spr_idx.get(name).unwrap_or(&0) as i32;
            for yy in 0..12 { for xx in 0..12 { let si = ((yy * self.spr_img.width as i32) + idx * 12 + xx) as usize * 4; let p = &self.spr_img.bytes[si..si + 4]; if p[3] == 0 { continue } let (tx, ty) = (x + xx, y + yy); if tx < 0 || ty < 0 || tx >= 100 || ty >= 100 { continue } let di = ((ty * 100 + tx) * 4) as usize; let d = &mut img.bytes[di..di + 4]; for k in 0..3 { d[k] = (p[k] as f32 * a + d[k] as f32 * (1.0 - a)) as u8 } d[3] = d[3].max((255.0 * a) as u8); } }
        }
    }
    #[allow(dead_code)] fn unused(&self) -> Big { b(0.0) }
    #[allow(dead_code)] fn fmt_time(x: f64) -> String { fmt_t(x) }
}
