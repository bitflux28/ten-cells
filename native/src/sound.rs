// Sound: the web version's slow space lo-fi soundtrack (same chords, melody and tempo) rendered once at start-up
// into a 49-second loop, plus short synth effects. Everything is synthesised; no sound files.
use crate::game::Set;
use macroquad::audio::{load_sound_from_bytes, play_sound, play_sound_once, set_sound_volume, PlaySoundParams, Sound};
use std::cell::RefCell;

const SR: f32 = 22050.0;
thread_local! { static SND: RefCell<Option<Bank>> = RefCell::new(None); }
struct Bank { music: Sound, fx: Vec<(&'static str, Sound)>, playing: bool }

fn mtof(m: f32) -> f32 { 440.0 * 2f32.powf((m - 69.0) / 12.0) }
fn wav(samples: &[f32]) -> Vec<u8> {
    let mut o = Vec::with_capacity(44 + samples.len() * 2); let n = samples.len() as u32 * 2;
    o.extend(b"RIFF"); o.extend((36 + n).to_le_bytes()); o.extend(b"WAVEfmt "); o.extend(16u32.to_le_bytes()); o.extend(1u16.to_le_bytes()); o.extend(1u16.to_le_bytes());
    o.extend((SR as u32).to_le_bytes()); o.extend((SR as u32 * 2).to_le_bytes()); o.extend(2u16.to_le_bytes()); o.extend(16u16.to_le_bytes()); o.extend(b"data"); o.extend(n.to_le_bytes());
    for s in samples { o.extend(((s.clamp(-1.0, 1.0) * 32000.0) as i16).to_le_bytes()) } o
}
struct Rng(u32); impl Rng { fn f(&mut self) -> f32 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 17; self.0 ^= self.0 << 5; (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0 } }
#[derive(Clone, Copy)] enum W { Sine, Tri, Saw }
/// add one note into buf: attack/decay/sustain/release envelope, optional pitch slide and one-pole lowpass
fn tone(buf: &mut [f32], at: f32, f: f32, f2: f32, w: W, a: f32, d: f32, sus: f32, rel: f32, v: f32, filt: f32) {
    let start = (at * SR) as usize; let len = ((a + d + sus + rel) * SR) as usize; let mut ph = 0.0f32; let mut lp = 0.0f32; let k = if filt > 0.0 { 1.0 - (-2.0 * std::f32::consts::PI * filt / SR).exp() } else { 1.0 };
    for i in 0..len {
        let t = i as f32 / SR; let idx = start + i; if idx >= buf.len() { break }
        let env = if t < a { t / a } else if t < a + d { 1.0 - (t - a) / d * 0.35 } else if t < a + d + sus { 0.65 } else { 0.65 * (1.0 - (t - a - d - sus) / rel).max(0.0) };
        let fr = if f2 > 0.0 { f2 + (f - f2) * (-t / 0.05).exp() } else { f }; ph = (ph + fr / SR).fract();
        let s = match w { W::Sine => (ph * std::f32::consts::TAU).sin(), W::Tri => 1.0 - 4.0 * (ph - 0.5).abs(), W::Saw => ph * 2.0 - 1.0 };
        lp += (s - lp) * k; buf[idx] += lp * env * v;
    }
}
fn noise(buf: &mut [f32], at: f32, d: f32, v: f32, hp: bool, r: &mut Rng) {
    let start = (at * SR) as usize; let len = (d * SR) as usize; let mut last = 0.0f32;
    for i in 0..len { let idx = start + i; if idx >= buf.len() { break } let n = r.f(); let s = if hp { n - last } else { (n + last) * 0.5 }; last = n; buf[idx] += s * v * (1.0 - i as f32 / len as f32).powi(2) }
}
fn reverb(buf: &mut Vec<f32>, mix: f32, wrap: bool) {
    let dry = buf.clone(); let n = buf.len();
    for (dl, g) in [(1557usize, 0.5f32), (1617, 0.48), (1491, 0.46), (1422, 0.44)] { let mut line = vec![0.0f32; dl]; let mut p = 0; for pass in 0..if wrap { 2 } else { 1 } { for i in 0..n { let y = line[p]; line[p] = dry[i] + y * g * 1.6; p = (p + 1) % dl; if pass == if wrap { 1 } else { 0 } { buf[i] += y * mix * 0.25 } } } }
}
fn music() -> Vec<f32> {
    const BPM: f32 = 78.0; let step = 60.0 / BPM / 4.0; let bar = step * 16.0;
    let prog_a = [(41, [57, 60, 64, 65]), (40, [55, 59, 62, 64]), (45, [55, 60, 64, 71]), (38, [53, 57, 60, 64])];
    let prog_b = [(36, [55, 59, 60, 64]), (43, [55, 59, 62, 64]), (41, [57, 59, 64, 65]), (40, [57, 59, 62, 64])];
    let pent = [69, 72, 74, 76, 79, 81, 84, 86];
    let mel: [[i8; 16]; 8] = [[-1, -1, 4, -2, -2, -1, 3, -1, 2, -2, -2, -2, -1, -1, -1, -1], [-1, -1, -1, -1, 3, -1, 4, -1, 5, -2, -2, -2, 4, -1, 3, -2], [2, -2, -2, -1, 1, -1, 2, -1, -1, -1, 0, -2, -2, -2, -1, -1], [-1; 16],
        [5, -2, -2, -1, 6, -1, 5, -2, 4, -2, -2, -1, 3, -1, -1, -1], [-1, -1, 2, -1, 3, -1, 4, -2, -2, -2, -1, -1, 3, -1, 2, -1], [1, -2, -2, -2, -1, -1, 0, -1, 1, -2, 2, -2, -2, -2, -1, -1], [-1; 16]];
    let bars = 16; let total = bar * bars as f32; let mut buf = vec![0.0f32; (total * SR) as usize + 1]; let mut r = Rng(0x2545F491);
    for st in 0..bars * 16 {
        let (pos, b) = (st % 16, st / 16); let sec = (b / 4) % 4; let (root, ch) = if sec == 2 { prog_b[b % 4] } else { prog_a[b % 4] }; let at = st as f32 * step;
        if pos == 0 { for n in ch { for dt in [1.0, 1.0045] { tone(&mut buf, at, mtof(n as f32) * dt, 0.0, W::Saw, 0.9, bar * 0.4, bar * 0.35, 1.4, 0.011, 750.0) } } }
        if pos == 0 { for (i, n) in ch.iter().enumerate() { tone(&mut buf, at + i as f32 * 0.018, mtof(*n as f32 + 12.0), 0.0, W::Tri, 0.006, 0.5, 0.0, 0.9, 0.028, 1900.0) } }
        if pos == 10 { for (i, n) in [ch[1], ch[3]].iter().enumerate() { tone(&mut buf, at + i as f32 * 0.02, mtof(*n as f32 + 12.0), 0.0, W::Tri, 0.006, 0.35, 0.0, 0.6, 0.02, 1700.0) } }
        if pos == 0 { tone(&mut buf, at, mtof(root as f32), 0.0, W::Sine, 0.02, step * 7.0, 0.0, 0.2, 0.17, 0.0) }
        if pos == 11 { tone(&mut buf, at, mtof(root as f32 + if b % 2 == 1 { 7.0 } else { 12.0 }), 0.0, W::Sine, 0.02, step * 3.0, 0.0, 0.15, 0.12, 0.0) }
        if b >= 4 {
            let sw = if pos % 2 == 1 { step * 0.2 } else { 0.0 };
            if pos == 0 || pos == 7 || pos == 10 { tone(&mut buf, at, 105.0, 44.0, W::Sine, 0.003, 0.16, 0.0, 0.05, if pos == 0 { 0.26 } else { 0.18 }, 0.0) }
            if pos == 4 || pos == 12 { noise(&mut buf, at, 0.14, 0.05, false, &mut r) }
            if pos % 2 == 0 { noise(&mut buf, at + sw, 0.03, if pos % 4 == 2 { 0.016 } else { 0.01 }, true, &mut r) }
        }
        let m = mel[b % 8][pos];
        if m >= 0 { let mut hold = 1; while pos + hold < 16 && mel[b % 8][pos + hold] == -2 { hold += 1 } let f = mtof(pent[m as usize] as f32); let d = step * hold as f32;
            tone(&mut buf, at, f, 0.0, W::Sine, 0.01, d, 0.0, 1.2, 0.05, 0.0); tone(&mut buf, at, f * 2.0, 0.0, W::Sine, 0.01, d * 0.5, 0.0, 0.6, 0.008, 0.0); tone(&mut buf, at + step * 3.0, f, 0.0, W::Sine, 0.01, d * 0.6, 0.0, 0.8, 0.018, 0.0) }
    }
    reverb(&mut buf, 0.6, true);
    let pk = buf.iter().fold(0.0f32, |m, s| m.max(s.abs())).max(0.01); for s in buf.iter_mut() { *s = *s / pk * 0.8 } buf
}
fn fx(name: &str) -> Vec<f32> {
    let mut b = vec![0.0f32; (SR * 1.2) as usize]; let mut r = Rng(7);
    match name {
        "tap" => tone(&mut b, 0.0, mtof(81.0), 0.0, W::Sine, 0.002, 0.07, 0.0, 0.08, 0.35, 0.0),
        "crit" => { tone(&mut b, 0.0, mtof(84.0), 0.0, W::Sine, 0.002, 0.08, 0.0, 0.1, 0.35, 0.0); tone(&mut b, 0.05, mtof(91.0), 0.0, W::Sine, 0.002, 0.1, 0.0, 0.2, 0.3, 0.0) }
        "buy" => { tone(&mut b, 0.0, mtof(76.0), 0.0, W::Tri, 0.003, 0.06, 0.0, 0.08, 0.35, 2500.0); tone(&mut b, 0.04, mtof(81.0), 0.0, W::Tri, 0.003, 0.06, 0.0, 0.1, 0.3, 2500.0) }
        "upg" => for (i, n) in [72.0, 76.0, 79.0, 84.0].iter().enumerate() { tone(&mut b, i as f32 * 0.05, mtof(*n), 0.0, W::Tri, 0.003, 0.08, 0.0, 0.2, 0.25, 3000.0) },
        "bad" => tone(&mut b, 0.0, 140.0, 90.0, W::Saw, 0.005, 0.12, 0.0, 0.08, 0.2, 900.0),
        "tab" => tone(&mut b, 0.0, mtof(88.0), 0.0, W::Sine, 0.001, 0.025, 0.0, 0.03, 0.15, 0.0),
        "event" => for (i, n) in [81.0, 88.0, 93.0].iter().enumerate() { tone(&mut b, i as f32 * 0.08, mtof(*n), 0.0, W::Sine, 0.003, 0.15, 0.0, 0.5, 0.25, 0.0) },
        "ab" => { tone(&mut b, 0.0, mtof(69.0), mtof(57.0), W::Saw, 0.01, 0.3, 0.0, 0.3, 0.2, 1400.0); noise(&mut b, 0.0, 0.3, 0.1, false, &mut r) }
        _ => { tone(&mut b, 0.0, 200.0, 60.0, W::Saw, 0.05, 0.8, 0.0, 0.3, 0.25, 1200.0); noise(&mut b, 0.0, 1.0, 0.25, false, &mut r); for (i, n) in [69.0, 76.0, 81.0, 88.0].iter().enumerate() { tone(&mut b, 0.3 + i as f32 * 0.07, mtof(*n), 0.0, W::Sine, 0.005, 0.3, 0.0, 0.6, 0.2, 0.0) } } // prestige
    }
    reverb(&mut b, 0.4, false); b
}
pub async fn init(set: &Set) {
    let m = load_sound_from_bytes(&wav(&music())).await; let mut fxs = vec![];
    for n in ["tap", "crit", "buy", "upg", "bad", "tab", "event", "ab", "prestige"] { if let Ok(s) = load_sound_from_bytes(&wav(&fx(n))).await { fxs.push((n, s)) } }
    if let Ok(m) = m { SND.with(|s| *s.borrow_mut() = Some(Bank { music: m, fx: fxs, playing: false })); sync(set) }
}
pub fn sync(set: &Set) {
    SND.with(|s| { let mut s = s.borrow_mut(); let Some(b) = s.as_mut() else { return };
        if set.mus > 0.0 { if !b.playing { play_sound(&b.music, PlaySoundParams { looped: true, volume: set.mv as f32 * 0.6 }); b.playing = true } set_sound_volume(&b.music, set.mv as f32 * 0.6) } else if b.playing { set_sound_volume(&b.music, 0.0) } });
}
pub fn sfx(name: &str, set: &Set) {
    if set.sfx <= 0.0 { return }
    SND.with(|s| { if let Some(b) = s.borrow().as_ref() { if let Some((_, snd)) = b.fx.iter().find(|f| f.0 == name) { if set.sv > 0.05 { play_sound(snd, PlaySoundParams { looped: false, volume: set.sv as f32 * 0.7 }) } else { play_sound_once(snd) } } } });
}
