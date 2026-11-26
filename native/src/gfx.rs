// Drawing: the built-in art (sprites, worlds, skies, 3D model loops), the pixel fonts, panels and buttons in the
// web version's style, and the animated atom reactor drawn pixel by pixel like the web canvas.
use crate::assets;
use macroquad::prelude::*;
use std::collections::HashMap;

pub fn hex(h: &str) -> Color {
    let h = h.trim_start_matches('#'); let p = |i: usize| u8::from_str_radix(h.get(i..i + 2).unwrap_or("ff"), 16).unwrap_or(255);
    if h.len() >= 8 { Color::from_rgba(p(0), p(2), p(4), p(6)) } else { Color::from_rgba(p(0), p(2), p(4), 255) }
}
pub fn alpha(c: Color, a: f32) -> Color { Color::new(c.r, c.g, c.b, c.a * a) }
pub fn mix(a: Color, b: Color, t: f32) -> Color { Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t) }
pub const INK: Color = Color::new(0.043, 0.039, 0.122, 1.0);
pub fn c(name: &str) -> Color {
    hex(match name { "bg" => "#12102e", "p1" => "#1b1845", "p2" => "#262056", "p3" => "#3d3780", "hi" => "#4f48a3", "vio" => "#8a63ff", "cyan" => "#45c8f5", "blue" => "#3466f0", "gold" => "#ffcc4d", "red" => "#ff4f73",
        "green" => "#4fe89a", "mag" => "#ff5fd0", "txt" => "#e6ebff", "mist" => "#a3a8d6", "faint" => "#6e72a8", "core" => "#c4f3ff", "dark" => "#110e30", "ink" => "#0b0a1f", _ => "#ffffff" })
}

/// swap characters the pixel fonts don't have for ones they do
pub fn clean(s: &str) -> std::borrow::Cow<'_, str> {
    if s.chars().all(|c| (c as u32) < 0x2070) { return std::borrow::Cow::Borrowed(s) }
    let mut o = String::with_capacity(s.len());
    for c in s.chars() { match c { '→' => o.push('›'), '←' => o.push('‹'), '★' => o.push('*'), '∞' => o.push_str("inf"), '⁴' => o.push_str("^4"), '₀' => o.push('0'), '₁' => o.push('1'), c if (c as u32) >= 0x2190 => {}, c => o.push(c) } }
    std::borrow::Cow::Owned(o)
}
pub struct Gfx {
    pub sprites: Texture2D, pub spr_idx: HashMap<String, usize>, pub fonts: [Font; 3], tex: HashMap<String, Texture2D>, pub atom: Image, pub atom_tex: Texture2D, pub out: Image,
    pub scale: f32,
}
fn load_png(name: &str) -> Option<Texture2D> { let b = assets::get(name)?; let t = Texture2D::from_file_with_format(b, Some(ImageFormat::Png)); t.set_filter(FilterMode::Nearest); Some(t) }
impl Gfx {
    pub fn new() -> Gfx {
        let spr_idx: HashMap<String, usize> = serde_json::from_slice(assets::get("sprites.json").unwrap()).unwrap();
        let font = |n: &str| { let mut f = load_ttf_font_from_bytes(assets::get(n).unwrap()).unwrap(); f.set_filter(FilterMode::Nearest); f };
        let atom = Image::gen_image_color(100, 100, BLANK); let atom_tex = Texture2D::from_image(&atom); atom_tex.set_filter(FilterMode::Nearest);
        Gfx { sprites: load_png("sprites.png").unwrap(), spr_idx, fonts: [font("PixelifySans_1.ttf"), font("PixelifySans_2.ttf"), font("PXNumbers_0.ttf")], tex: HashMap::new(), atom, atom_tex, out: Image::gen_image_color(100, 100, BLANK), scale: 1.0 }
    }
    pub fn tex(&mut self, name: &str) -> Option<Texture2D> {
        if let Some(t) = self.tex.get(name) { return Some(t.clone()) }
        let t = load_png(name)?; self.tex.insert(name.into(), t.clone()); Some(t)
    }
    /// a 12×12 sprite (optionally a swap variant like "gem@r3" or "ghost@boss0")
    pub fn spr(&self, name: &str, x: f32, y: f32, size: f32, tint: Color) {
        let i = *self.spr_idx.get(name).or_else(|| self.spr_idx.get(name.split('@').next().unwrap())).or_else(|| self.spr_idx.get("question")).unwrap_or(&0);
        draw_texture_ex(&self.sprites, x.round(), y.round(), tint, DrawTextureParams { dest_size: Some(vec2(size, size)), source: Some(Rect::new(i as f32 * 12.0, 0.0, 12.0, 12.0)), ..Default::default() });
    }
    /// frame of a 3D model loop (16 frames of 128px), e.g. "pet_spark_2", "boss_3", "char_vela" (+"_talk")
    pub fn model(&mut self, name: &str, x: f32, y: f32, size: f32, t: f64, tint: Color) {
        let Some(tex) = self.tex(&format!("{name}.png")) else { return }; let f = ((t * 8.0) as usize) % 16;
        draw_texture_ex(&tex, x.round(), y.round(), tint, DrawTextureParams { dest_size: Some(vec2(size, size)), source: Some(Rect::new(f as f32 * 128.0, 0.0, 128.0, 128.0)), ..Default::default() });
    }
    /// a world picture (96 frames of 50px), locked = greyed still
    pub fn world(&mut self, id: &str, x: f32, y: f32, size: f32, t: f64, locked: bool) {
        let Some(tex) = self.tex(&format!("world_{id}.png")) else { return }; let f = ((t * 15.0) as usize) % 96;
        let src = if locked { Rect::new(0.0, 50.0, 50.0, 50.0) } else { Rect::new(f as f32 * 50.0, 0.0, 50.0, 50.0) };
        draw_texture_ex(&tex, x.round(), y.round(), WHITE, DrawTextureParams { dest_size: Some(vec2(size, size)), source: Some(src), ..Default::default() });
    }
    // ---------------- text ----------------
    /// font: 0 regular, 1 bold, 2 numbers
    pub fn text(&self, s: &str, x: f32, y: f32, size: f32, col: Color, font: usize) -> f32 {
        let s = &*clean(s); let sz = (size * self.scale).round() as u16;
        let dims = measure_text(s, Some(&self.fonts[font]), sz, 1.0);
        draw_text_ex(s, (x * self.scale).round(), (y * self.scale + dims.offset_y).round(), TextParams { font: Some(&self.fonts[font]), font_size: sz, color: col, ..Default::default() });
        dims.width / self.scale
    }
    /// text with the web version's hard 2px drop shadow
    pub fn text_sh(&self, s: &str, x: f32, y: f32, size: f32, col: Color, font: usize) -> f32 { self.text(s, x + 2.0, y + 2.0, size, INK, font); self.text(s, x, y, size, col, font) }
    /// cut text to fit a width, ending in …
    pub fn fit(&self, s: &str, w: f32, size: f32, font: usize) -> String {
        if self.tw(s, size, font) <= w { return s.to_string() }
        let mut t: Vec<char> = s.chars().collect(); while !t.is_empty() && self.tw(&format!("{}…", t.iter().collect::<String>()), size, font) > w { t.pop(); }
        format!("{}…", t.iter().collect::<String>().trim_end())
    }
    pub fn tw(&self, s: &str, size: f32, font: usize) -> f32 { let s = &*clean(s); measure_text(s, Some(&self.fonts[font]), (size * self.scale).round() as u16, 1.0).width / self.scale }
    pub fn text_c(&self, s: &str, cx: f32, y: f32, size: f32, col: Color, font: usize) { let w = self.tw(s, size, font); self.text(s, cx - w / 2.0, y, size, col, font); }
    pub fn text_r(&self, s: &str, rx: f32, y: f32, size: f32, col: Color, font: usize) { let w = self.tw(s, size, font); self.text(s, rx - w, y, size, col, font); }
    /// word-wrapped text, returns the height used
    pub fn wrap(&self, s: &str, x: f32, y: f32, w: f32, size: f32, col: Color, font: usize) -> f32 {
        let lh = size * 1.35; let mut yy = y;
        for para in s.split('\n') {
            let mut line = String::new();
            for word in para.split(' ') {
                let t = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
                if self.tw(&t, size, font) > w && !line.is_empty() { self.text(&line, x, yy, size, col, font); yy += lh; line = word.to_string() } else { line = t }
            }
            self.text(&line, x, yy, size, col, font); yy += lh;
        }
        yy - y
    }
    pub fn wrap_h(&self, s: &str, w: f32, size: f32, font: usize) -> f32 {
        let lh = size * 1.35; let mut n = 0.0;
        for para in s.split('\n') { let mut line = String::new(); n += 1.0; for word in para.split(' ') { let t = if line.is_empty() { word.to_string() } else { format!("{line} {word}") }; if self.tw(&t, size, font) > w && !line.is_empty() { n += 1.0; line = word.to_string() } else { line = t } } }
        n * lh
    }
    // ---------------- shapes in logical px ----------------
    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32, col: Color) { let k = self.scale; draw_rectangle((x * k).round(), (y * k).round(), (w * k).round(), (h * k).round(), col) }
    /// the web .panel: flat fill with a 3px light top-left bevel and a dark bottom-right one
    pub fn panel(&self, x: f32, y: f32, w: f32, h: f32, fill: Color) {
        self.rect(x, y, w, h, fill); self.rect(x, y, w, 3.0, c("p3")); self.rect(x, y, 3.0, h, c("p3")); self.rect(x, y + h - 3.0, w, 3.0, c("dark")); self.rect(x + w - 3.0, y, 3.0, h, c("dark"));
    }
    /// the web .btn: ink border with clipped corners, bevel inside
    pub fn button(&self, x: f32, y: f32, w: f32, h: f32, fill: Color, lo: Color, hi: Color, pressed: bool) {
        let dy = if pressed { 2.0 } else { 0.0 };
        self.rect(x + 3.0, y + dy, w - 6.0, h, INK); self.rect(x, y + 3.0 + dy, w, h - 6.0, INK);
        self.rect(x + 3.0, y + 3.0 + dy, w - 6.0, h - 6.0, fill);
        if !pressed { self.rect(x + 3.0, y + 3.0, w - 6.0, 3.0, hi) }
        self.rect(x + 3.0, y + h - 7.0 + dy, w - 6.0, 4.0, lo);
    }
    pub fn bar(&self, x: f32, y: f32, w: f32, h: f32, p: f32, col: Color) { self.rect(x, y, w, h, INK); let iw = ((w - 4.0) * p.clamp(0.0, 1.0)).max(0.0); self.rect(x + 2.0, y + 2.0, iw, h - 4.0, col); if iw > 2.0 { self.rect(x + 2.0, y + 2.0, iw, 2.0, alpha(WHITE, 0.35)) } }
    pub fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32, w: f32, col: Color) { let k = self.scale; draw_line(x1 * k, y1 * k, x2 * k, y2 * k, w * k, col) }
}

// ---------------- the atom reactor, drawn into a 100×100 pixel buffer exactly like the web canvas ----------------
pub struct AtomIn<'a> { pub t: f64, pub a: f64, pub hit: f64, pub heat: f64, pub dead: bool, pub fx: bool, pub gens: &'a [f64], pub rings: &'a [(f64, bool)], pub c2: Color }
const OUTC: [u8; 4] = [11, 10, 31, 255];
fn put(img: &mut Image, x: i32, y: i32, w: i32, h: i32, col: Color, a: f32) {
    for yy in y.max(0)..(y + h).min(100) { for xx in x.max(0)..(x + w).min(100) {
        let i = (yy * 100 + xx) as usize * 4; let d = &mut img.bytes[i..i + 4]; let sa = col.a * a; let da = d[3] as f32 / 255.0; let oa = sa + da * (1.0 - sa);
        if oa <= 0.0 { continue }
        for k in 0..3 { let s = [col.r, col.g, col.b][k] * 255.0; d[k] = ((s * sa + d[k] as f32 * da * (1.0 - sa)) / oa).round() as u8 } d[3] = (oa * 255.0).round() as u8;
    } }
}
fn disc_shaded(img: &mut Image, cx: f64, cy: f64, r: f64, cols: [Color; 4]) {
    // a sphere lit from the top-left with 4 shades and Bayer dithering (the web wdisc)
    const BAYER: [[f64; 4]; 4] = [[0., 8., 2., 10.], [12., 4., 14., 6.], [3., 11., 1., 9.], [15., 7., 13., 5.]];
    let ri = r.ceil() as i32;
    for dy in -ri..=ri { for dx in -ri..=ri {
        let (fx, fy) = (dx as f64 / r, dy as f64 / r); let d2 = fx * fx + fy * fy; if d2 > 1.0 { continue }
        let nz = (1.0 - d2).sqrt(); let l = (-fx * 0.5 - fy * 0.6 + nz * 0.62).max(0.0); let v = (l * 3.6).min(3.99); let base = v.floor(); let frac = v - base;
        let (X, Y) = ((cx + dx as f64).round() as i32, (cy + dy as f64).round() as i32);
        let k = if frac * 16.0 > BAYER[(Y & 3) as usize][(X & 3) as usize] { (base as usize + 1).min(3) } else { base as usize };
        put(img, X, Y, 1, 1, cols[k], 1.0);
    } }
}
pub fn draw_atom(img: &mut Image, out: &mut Image, gfx_spr: &dyn Fn(&mut Image, usize, i32, i32, f32), a: &AtomIn) {
    for p in img.bytes.iter_mut() { *p = 0 }
    let (cx, cy) = (50.0, 50.0); let A = a.a; let mv = if a.fx { 1.0 } else { 0.0 }; let hit = a.hit;
    let ec = if a.dead { [hex("#8a8fbf"); 3] } else { [a.c2, hex("#ff5fd0"), hex("#ffcc4d")] };
    let mut back = vec![]; let mut front = vec![]; let mut elec = vec![];
    for (k, (r, sq, rot)) in [(45.0f64, 0.3f64, 0.3f64), (41.0, 0.5, -0.9), (37.0, 0.4, 1.4)].iter().enumerate() {
        let dir = if k % 2 == 1 { -1.0 } else { 1.0 };
        for j in 0..10 { let th = A * dir * (2.4 + 0.4 * k as f64) + k as f64 * 2.1 - j as f64 * 0.06 * dir; let (ex, ey) = (th.cos() * r, th.sin() * r * sq); elec.push((cx + ex * rot.cos() - ey * rot.sin(), cy + ex * rot.sin() + ey * rot.cos(), j, th.sin() > 0.0, ec[k])); }
        for i in 0..150 { let th = i as f64 / 150.0 * 6.283 + A * dir * (0.6 + 0.2 * k as f64); let (ex, ey) = (th.cos() * r, th.sin() * r * sq); let p = (cx + ex * rot.cos() - ey * rot.sin(), cy + ex * rot.sin() + ey * rot.cos(), ec[k]); if th.sin() > 0.0 { front.push(p) } else { back.push(p) } }
    }
    // energy makers orbit as little satellites
    let mut sats = vec![]; for i in 0..a.gens.len() { if a.gens[i] > 0.0 { let r = 30.0 + (i % 13) as f64 * 1.5; let an = A * if i % 2 == 1 { -0.7 } else { 0.7 } / (1.0 + i as f64 * 0.15) + i as f64 * 0.63; let z = an.sin(); sats.push((i, cx + an.cos() * r, cy + z * r * 0.34, z)); } }
    for (x, y, col) in &back { put(img, x.round() as i32, y.round() as i32, 1, 1, *col, 0.4) }
    for s in sats.iter().filter(|s| s.3 <= 0.0) { gfx_spr(img, s.0, (s.1 - 6.0).round() as i32, (s.2 - 6.0).round() as i32, 0.55) }
    elec.sort_by(|p, q| q.2.cmp(&p.2));
    let draw_e = |img: &mut Image, e: &(f64, f64, usize, bool, Color)| { let (x, y) = (e.0.round() as i32, e.1.round() as i32); if e.2 > 0 { let w = if e.2 < 4 { 2 } else { 1 }; put(img, x, y, w, w, e.4, 1.0 - e.2 as f32 / 10.0) } else { put(img, x - 1, y - 1, 4, 4, e.4, 1.0); put(img, x, y, 2, 2, WHITE, 1.0) } };
    for e in elec.iter().filter(|e| !e.3) { draw_e(img, e) }
    if mv > 0.0 && !a.dead {
        let n = 26 + (a.heat * 40.0) as i32;
        for k in 0..n { let an = crate::game::rnd() * 6.283; let rr = 14.0 + crate::game::rnd().powf(0.7) * 22.0; let col = if a.heat > 0.4 && k % 3 == 0 { hex("#ffcc4d") } else if k % 2 == 1 { a.c2 } else { hex("#3466f0") }; put(img, (cx + an.cos() * rr).round() as i32, (cy + an.sin() * rr * 0.85).round() as i32, 1, 1, col, 0.35 + crate::game::rnd() as f32 * 0.4) }
    }
    // the nucleus: 11 shaded spheres turning in 3D
    let nucl = [[0., 0., 0.], [1., 0., 0.], [-1., 0., 0.], [0., 1., 0.], [0., -1., 0.], [0., 0., 1.], [0., 0., -1.], [0.62, 0.62, 0.55], [-0.62, -0.55, 0.6], [0.6, -0.6, -0.6], [-0.6, 0.62, -0.6]];
    let (ry, rxa, sc) = (A * 0.7, A * 0.43, 7.5 * (1.0 - hit * 0.25));
    let mut nuc: Vec<(f64, f64, f64, usize)> = nucl.iter().enumerate().map(|(k, p)| { let x1 = p[0] * ry.cos() + p[2] * ry.sin(); let z1 = -p[0] * ry.sin() + p[2] * ry.cos(); let y1 = p[1] * rxa.cos() - z1 * rxa.sin(); let z2 = p[1] * rxa.sin() + z1 * rxa.cos(); let jig = if mv > 0.0 { (a.t * 9.0 + k as f64 * 2.1).sin() * 0.5 * (1.0 + a.heat * 2.0) } else { 0.0 }; (cx + x1 * sc + jig, cy + y1 * sc, z2, k) }).collect();
    nuc.sort_by(|p, q| p.2.partial_cmp(&q.2).unwrap());
    let hx = |s: [&str; 4]| [hex(s[0]), hex(s[1]), hex(s[2]), hex(s[3])];
    for n in &nuc { let cols = if a.dead { hx(["#262056", "#3d3780", "#8a8fbf", "#c9cde8"]) } else if n.3 % 2 == 1 { hx(["#8f1d3c", "#ff4f73", "#ffb3c3", "#ffffff"]) } else { hx(["#1d3492", "#3466f0", "#9ab8ff", "#ffffff"]) }; disc_shaded(img, n.0, n.1, 5.4 - hit, cols) }
    for (x, y, col) in &front { put(img, x.round() as i32, y.round() as i32, 1, 1, *col, 1.0) }
    for e in elec.iter().filter(|e| e.3) { draw_e(img, e) }
    for s in sats.iter().filter(|s| s.3 > 0.0) { gfx_spr(img, s.0, (s.1 - 6.0).round() as i32, (s.2 - 6.0).round() as i32, 1.0) }
    let hn = (a.heat * 40.0) as i32; for i in 0..hn { let an = -std::f64::consts::FRAC_PI_2 + i as f64 / 40.0 * 6.283; put(img, (cx + an.cos() * 48.0).round() as i32, (cy + an.sin() * 48.0).round() as i32, 2, 2, if i < 20 { hex("#ffcc4d") } else { hex("#ff4f73") }, 1.0) }
    for (r, crit) in a.rings { let col = if *crit { hex("#ffcc4d") } else { a.c2 }; let rr = r.round(); let n = (rr * 6.283).max(8.0) as i32; for k in 0..n { let an = k as f64 / n as f64 * 6.283; put(img, (cx + an.cos() * rr).round() as i32, (cy + an.sin() * rr).round() as i32, 1, 1, col, 1.0) } }
    // dark outline behind it all (2px round the nucleus) and a soft pulsing light behind that
    out.bytes.copy_from_slice(&img.bytes);
    let solid = |b: &[u8], x: i32, y: i32| x >= 0 && y >= 0 && x < 100 && y < 100 && b[((y * 100 + x) * 4 + 3) as usize] > 0;
    for y in 0..100 { for x in 0..100 {
        if solid(&out.bytes, x, y) { continue }
        let near = (x as f64 - 50.0).hypot(y as f64 - 50.0) < 17.0; let mut best = 0u8;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (2, 0), (-2, 0), (0, 2), (0, -2), (1, 1), (-1, 1), (1, -1), (-1, -1)] { if (dx as i32).abs() + (dy as i32).abs() == 2 && !near { continue } let (sx, sy) = (x - dx, y - dy); if solid(&out.bytes, sx, sy) { best = best.max(out.bytes[((sy * 100 + sx) * 4 + 3) as usize]) } }
        if best > 0 { let i = ((y * 100 + x) * 4) as usize; img.bytes[i..i + 4].copy_from_slice(&[OUTC[0], OUTC[1], OUTC[2], best]) }
    } }
    if !a.dead {
        let g = mix(a.c2, WHITE, 0.35); let p = if mv > 0.0 { (a.t * 2.6).sin() } else { 0.0 };
        for y in 0..100 { for x in 0..100 {
            let i = ((y * 100 + x) * 4) as usize; if img.bytes[i + 3] > 0 { continue }
            let d = (x as f64 - 50.0).hypot(y as f64 - 50.0); let al = if d <= 16.0 { 0.3 + 0.18 + 0.1 } else if d <= 20.0 + p { 0.18 + 0.1 } else if d <= 25.0 + p * 1.5 { 0.1 } else { 0.0 };
            if al > 0.0 { img.bytes[i..i + 4].copy_from_slice(&[(g.r * 255.0) as u8, (g.g * 255.0) as u8, (g.b * 255.0) as u8, (al * 255.0) as u8]) }
        } }
    }
}
