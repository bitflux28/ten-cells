// Every screen of the game: Engine, Upgrades, Prestige, Worlds, Crew, Stats, plus the pop-up sheets and story scenes.
use crate::big::{b, Big};
use crate::data::{data, NG};
use crate::fmt::{fmt, fmt_t, fmtf, group};
use crate::game::{now_ms, oom, Game, LAYERS};
use crate::gfx::{alpha, c, hex, INK};
use crate::ui::{App, Sheet};
use macroquad::prelude::*;

fn sfx(a: &App, n: &str) { crate::sound::sfx(n, &a.g.s.set) }
fn eta(g: &Game, cost: Big) -> String {
    if cost.lte(g.s.energy) { return String::new() } let r = g.c.eps.mulf(g.c.sp); if !r.gtf(0.0) { return String::new() }
    let s = cost.sub(g.s.energy).div(r).to_f64(); if s.is_finite() && s < 86400.0 * 365.0 { format!("in {}", fmt_t(s.ceil())) } else { String::new() }
}
/// a shop row: icon, name, description, a buy button on the right. Returns (height, clicked)
fn item(a: &mut App, y: f32, spr: &str, name: &str, desc: &str, btn: &str, ok: bool, owned: bool, extra: &str) -> (f32, bool) {
    let (x, w) = (a.x0 + 8.0, a.cw - 16.0); let bw = 104.0; let tw = w - bw - 70.0;
    let dh = a.gfx.wrap_h(desc, tw, 13.0, 0) + if extra.is_empty() { 0.0 } else { 18.0 }; let h = (dh + 34.0).max(62.0);
    a.gfx.panel(x, y, w, h, if owned { hex("#15233a") } else { c("p1") });
    a.spr(spr, x + 12.0, y + 14.0, 32.0);
    let nm = a.gfx.fit(name, tw + if owned { 30.0 } else { 0.0 }, 15.0, 1); a.gfx.text(&nm, x + 56.0, y + 10.0, 15.0, if owned { c("green") } else { WHITE }, 1);
    a.gfx.wrap(desc, x + 56.0, y + 30.0, tw, 13.0, c("mist"), 0);
    if !extra.is_empty() { let ex = a.gfx.fit(extra, tw, 12.0, 0); a.gfx.text(&ex, x + 56.0, y + 30.0 + dh - 18.0, 12.0, c("faint"), 0); }
    let clicked = if owned { a.gfx.text_r("Owned", x + w - 16.0, y + h / 2.0 - 7.0, 14.0, c("green"), 1); false } else { a.btn(x + w - bw - 10.0, y + (h - 44.0) / 2.0, bw, 44.0, btn, if ok { "on" } else { "" }, ok) };
    (h + 8.0, clicked)
}
fn title(a: &App, y: f32, t: &str) -> f32 { let w = a.gfx.tw(t, 18.0, 1); a.gfx.rect(a.x0 + 8.0, y - 4.0, w + 16.0, 28.0, alpha(INK, 0.72)); a.gfx.text_sh(t, a.x0 + 14.0, y, 18.0, WHITE, 1); 30.0 }
fn muted(a: &App, y: f32, t: &str) -> f32 { let h = a.gfx.wrap_h(t, a.cw - 28.0, 14.0, 0); a.gfx.rect(a.x0 + 8.0, y - 4.0, a.cw - 16.0, h + 6.0, alpha(INK, 0.72)); a.gfx.wrap(t, a.x0 + 14.0, y, a.cw - 28.0, 14.0, c("mist"), 0) + 8.0 }

pub fn draw(a: &mut App, y: f32) -> f32 {
    let y0 = y; let tab = a.tab.clone();
    let h = match tab.as_str() { "engine" => engine(a, y), "upgrades" => upgrades(a, y), "prestige" => prestige(a, y), "worlds" => worlds(a, y), "crew" => crew(a, y), _ => stats(a, y) };
    h.max(y - y0)
}

// ============================ ENGINE ============================
fn engine(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let (x, w) = (a.x0 + 8.0, a.cw - 16.0);
    // random event to claim
    if let Some(cur) = a.g.s.ev.cur.clone() {
        let e = data().events.iter().find(|e| e.k == cur.k).unwrap().clone(); let col = hex(&e.col);
        a.gfx.panel(x, y, w, 70.0, c("p1")); a.gfx.rect(x, y, 4.0, 70.0, col);
        let bob = (a.time * 3.0).sin() as f32 * 3.0; let spr = match cur.k.as_str() { "storm" => "bolt", "collapse" => "atom", "golden" => "star", "surge" => "comet", "leak" => "drop", _ => "glitch" }; a.spr(spr, x + 14.0, y + 17.0 + bob, 36.0);
        a.gfx.text(&e.n, x + 62.0, y + 12.0, 16.0, col, 1); a.gfx.text(&e.d, x + 62.0, y + 34.0, 13.0, c("mist"), 0); a.gfx.bar(x + 62.0, y + 54.0, w - 190.0, 8.0, (cur.t / 15.0) as f32, col);
        if a.btn(x + w - 112.0, y + 13.0, 100.0, 44.0, "Claim", "on", true) { a.g.claim_event(); sfx(a, "event") }
        y += 78.0;
    }
    // daily reward
    if a.g.daily_ready() {
        let k = (a.g.s.daily.day as usize) % 7; let r = data().rew[k].clone();
        a.gfx.panel(x, y, w, 76.0, hex("#2a2252")); a.model(&format!("rew_{k}"), x + 6.0, y + 4.0, 68.0);
        a.gfx.text(&format!("Daily reward · Day {}", k + 1), x + 80.0, y + 12.0, 15.0, c("gold"), 1); a.gfx.text(&r.n, x + 80.0, y + 32.0, 14.0, WHITE, 0); a.gfx.text(&r.d, x + 80.0, y + 52.0, 12.0, c("mist"), 0);
        if a.btn(x + w - 100.0, y + 16.0, 88.0, 44.0, "Claim", "gold", true) { if let Some((_, msg)) = a.g.claim_daily() { a.toast(format!("{}: {}", r.n, msg), &r.spr, c("gold")); sfx(a, "upg"); a.save(); } }
        y += 84.0;
    }
    // permanent upgrades bar
    if a.g.s.set.perm_bar > 0.0 { if let Some(u) = data().perm.iter().find(|u| !a.g.shop_owned("pm", &u.id)).cloned() {
        let own = data().perm.iter().filter(|u| a.g.shop_owned("pm", &u.id)).count(); let ok = a.g.s.energy.gte(u.c);
        a.gfx.panel(x, y, w, 56.0, hex("#2b2340")); a.gfx.rect(x, y, 4.0, 56.0, c("gold")); a.spr("crown", x + 12.0, y + 14.0, 28.0);
        a.gfx.text(&u.n, x + 48.0, y + 10.0, 14.0, c("gold"), 1); a.gfx.text(&format!("Permanent {}/{} · {}", own, data().perm.len(), u.d), x + 48.0, y + 31.0, 12.0, c("mist"), 0);
        let lab = format!("{}", fmt(u.c)); if a.btn(x + w - 154.0, y + 8.0, 96.0, 40.0, &lab, if ok { "gold" } else { "" }, ok) { if a.g.buy_perm(&u.id) { a.toast(format!("Permanent: {}", u.n), "star", c("gold")); sfx(a, "upg") } }
        if a.btn(x + w - 54.0, y + 8.0, 44.0, 40.0, "×", "ghost", true) { a.g.s.set.perm_bar = 0.0; a.toast("Permanent upgrades are in Upgrades › Permanent", "crown", c("gold")) }
        y += 64.0;
    } }
    // challenge banner
    if let Some(ch) = a.g.s.chal.a.clone() {
        let goal = data().chal_goal[ch.l as usize - 1]; let n = data().chal.iter().find(|c| c.id == ch.id).map(|c| c.n.clone()).unwrap_or_default();
        a.gfx.panel(x, y, w, 64.0, hex("#3a1a2e")); a.gfx.text(&format!("Challenge: {} {}", n, "I".repeat(ch.l as usize)), x + 12.0, y + 10.0, 15.0, c("red"), 1);
        a.gfx.bar(x + 12.0, y + 34.0, w - 120.0, 14.0, (oom(a.g.s.run_e) / goal.log10()) as f32, c("red")); a.gfx.text(&format!("{} / {}", fmt(a.g.s.run_e), fmt(goal)), x + 12.0, y + 50.0, 11.0, c("mist"), 2);
        if a.btn(x + w - 100.0, y + 12.0, 88.0, 40.0, "Leave", "danger", true) { a.sheet = Sheet::Confirm("Leave the challenge?".into(), "Your current run restarts without a reward.".into(), "Leave".into(), "chalx".into()) }
        y += 72.0;
    }
    // boss fight
    if let Some(f) = a.g.s.boss.f.clone() {
        let bo = data().bosses[f.i].clone(); a.gfx.panel(x, y, w, 84.0, hex("#2a1530"));
        let hurt = a.time % 0.4 < 0.2 && a.rx_hit > 0.3; a.model(&format!("boss_{}", f.i), x + 4.0, y + 2.0, 80.0); if hurt { a.gfx.rect(x + 4.0, y + 2.0, 80.0, 80.0, alpha(c("red"), 0.15)) }
        a.gfx.text(&bo.n, x + 92.0, y + 10.0, 16.0, c("red"), 1); let hp = (1.0 - f.dmg.div(bo.hp).to_f64()).max(0.0); a.gfx.bar(x + 92.0, y + 34.0, w - 104.0, 16.0, hp as f32, c("red"));
        a.gfx.text(&format!("{} HP · {}s · {} dmg/s", fmt(bo.hp.sub(f.dmg).maxf(0.0)), f.t.ceil(), fmt(a.g.c.eps.mul(a.g.boss_dmg()))), x + 92.0, y + 56.0, 12.0, c("mist"), 2);
        y += 92.0;
    }
    // abilities
    let abs: Vec<_> = data().abil.iter().filter(|ab| a.g.fx.ab.contains(&ab.0)).cloned().collect();
    if !abs.is_empty() {
        let bw = (w - (abs.len() as f32 - 1.0) * 6.0) / abs.len() as f32;
        for (i, ab) in abs.iter().enumerate() { let cd = a.g.s.ab.get(&ab.0).copied().unwrap_or(0.0); let ok = cd <= 0.0; let lab = if ok { ab.1.clone() } else { format!("{}\n{}", ab.1, fmt_t(cd.ceil())) };
            if a.btn(x + i as f32 * (bw + 6.0), y, bw, 48.0, &lab, if ok { "vio" } else { "" }, ok) { a.g.use_ab(&ab.0); sfx(a, "ab"); a.toast(format!("{}: {}", ab.1, ab.4), "bolt", c("vio")) } }
        y += 56.0;
    }
    // the reactor
    let cx = a.x0 + a.cw / 2.0; let rs = 300.0f32.min(a.cw - 30.0); let ry = y + 10.0;
    let top = a.g.top_dim(); let dead = a.g.in_c("notap") || a.g.law("haste");
    if let Some(ti) = top {
        let id = data().dims[ti].id.clone(); let s = rs * 0.8 * (1.0 + a.rx_hit as f32 * 0.08); a.world(&id, cx - s / 2.0, ry + (rs - s) / 2.0, s, false);
        let col = hex(&data().dims[ti].col); for k in 0..16 { let an = a.rx_a * 0.6 + k as f64 / 16.0 * 6.283; let (px, py) = (cx + (an.cos() as f32) * rs * 0.47, ry + rs / 2.0 + (an.sin() as f32) * rs * 0.47); a.gfx.rect(px - 3.0, py - 3.0, 6.0, 6.0, if k % 4 > 0 { alpha(col, 0.55) } else { WHITE }) }
    } else {
        let rings = a.rings.clone(); let gens = a.g.s.gens.clone(); let (t, ra, hit, heat, fx) = (a.time, a.rx_a, a.rx_hit, a.g.heat, a.g.s.set.fx > 0.0);
        let mut img = std::mem::replace(&mut a.gfx.atom, Image::empty()); let mut out = std::mem::replace(&mut a.gfx.out, Image::empty());
        { let blit = a.atom_blit(); crate::gfx::draw_atom(&mut img, &mut out, &blit, &crate::gfx::AtomIn { t, a: ra, hit, heat, dead, fx, gens: &gens, rings: &rings, c2: c("cyan") }); }
        a.gfx.atom_tex.update(&img); a.gfx.atom = img; a.gfx.out = out;
        let k = a.gfx.scale; draw_texture_ex(&a.gfx.atom_tex, ((cx - rs / 2.0) * k).round(), (ry * k).round(), WHITE, DrawTextureParams { dest_size: Some(vec2(rs * k, rs * k)), ..Default::default() });
    }
    // the active pet sits next to the reactor
    if let Some(p) = a.g.s.pets.active.clone() { if let Some(o) = a.g.s.pets.own.get(&p) { let st = Game::pet_stage(o.lvl); a.model(&format!("pet_{p}_{st}"), x + w - 78.0, ry + 4.0, 72.0); } }
    let rr = Rect::new(cx - rs / 2.0, ry, rs, rs);
    if a.pressed && a.hover(rr) && !dead { let (mx, my) = (a.mx, a.my); a.tap(mx, my); sfx(a, "tap"); }
    if a.click && a.hover(rr) { a.click = false }
    y = ry + rs + 4.0;
    let name = match top { Some(ti) => format!("Your engine: {}", data().scale_name.iter().find(|s| s.0 == data().dims[ti].id).map(|s| s.1.replace("A ", "a ").replace("The ", "the ")).unwrap_or_default()), None => "Your engine: an atom".into() };
    a.gfx.text_c(&name, cx + 1.0, y + 1.0, 13.0, INK, 0); a.gfx.text_c(&name, cx, y, 13.0, c("gold"), 0); y += 20.0;
    let tapl = if dead { "Tapping is disabled here".to_string() } else { format!("Tap the reactor · {}", fmt(a.g.c.tap.add(a.g.c.tap_e))) };
    a.gfx.text_c(&tapl, cx + 2.0, y + 2.0, 14.0, INK, 0); a.gfx.text_c(&tapl, cx, y, 14.0, WHITE, 0);
    if a.g.heat > 0.05 { a.gfx.text_c(&format!("Overdrive ×{:.2}", 1.0 + a.g.heat), cx, y + 18.0, 13.0, c("gold"), 0) }
    y += 40.0;
    // next story goal
    if !a.g.story_done() {
        let ch = a.g.s.story.ch as usize; if let Some(m) = data().missions.get(ch).and_then(|ms| ms.iter().find(|m| { let (v, n) = a.g.mission_val(m); v < n })).cloned() {
            let (v, n) = a.g.mission_val(&m); a.gfx.panel(x, y, w, 50.0, c("p1")); a.spr("scroll", x + 10.0, y + 12.0, 24.0);
            a.gfx.text(&format!("{} · {} / {}", m.t, group(v.floor() as i64), group(n as i64)), x + 44.0, y + 10.0, 13.0, c("txt"), 0); a.gfx.bar(x + 44.0, y + 30.0, w - 56.0, 10.0, (v / n) as f32, c("gold"));
            if a.hit(Rect::new(x, y, w, 50.0)) { a.tab = "crew".into(); a.set_sub("crew", "story") }
            y += 58.0;
        } else if a.g.missions_done(ch) { if a.btn(x, y, w, 44.0, "Chapter ready: tap to complete it in Crew", "gold", true) { a.tab = "crew".into(); a.set_sub("crew", "story") } y += 52.0; }
    }
    // buy mode + everything
    let modes = [(1.0, "×1"), (10.0, "×10"), (25.0, "×25"), (0.0, "MAX")]; let bw = (w - 18.0) / 4.0;
    for (i, (m, l)) in modes.iter().enumerate() { if a.btn(x + i as f32 * (bw + 6.0), y, bw, 40.0, l, if a.buy == *m { "vio" } else { "ghost" }, true) { a.buy = *m } }
    y += 46.0;
    if a.btn(x, y, w, 40.0, "Buy everything (G)", "", true) { let n = a.g.buy_everything(); if n > 0.0 { a.toast(format!("Bought {} energy makers", group(n as i64)), "gear", c("cyan")); sfx(a, "buy") } else { a.toast("Nothing affordable yet", "cross", c("red")); sfx(a, "bad") } }
    y += 48.0;
    // energy makers
    let mut shown_next = false;
    for i in 0..NG {
        let vis = a.g.gen_visible(i);
        if !vis { if !shown_next { shown_next = true; a.gfx.panel(x, y, w, 66.0, c("p1")); a.spr("question", x + 12.0, y + 18.0, 28.0); a.gfx.text("Unknown signal", x + 52.0, y + 9.0, 15.0, c("faint"), 1);
            let c0 = a.g.gen_cost_n(i - 1, 1.0); let e = eta(&a.g, c0); let l1 = a.gfx.fit(&format!("Own one {} to reveal the next maker", data().gens[i - 1].n), w - 64.0, 12.0, 0); a.gfx.text(&l1, x + 52.0, y + 29.0, 12.0, c("mist"), 0);
            a.gfx.text(&if e.is_empty() { "You can afford one now".to_string() } else { format!("Affordable {e}") }, x + 52.0, y + 45.0, 12.0, c("cyan"), 0); y += 74.0; } continue }
        let g = &a.g; let gen = &data().gens[i]; let own = g.s.gens[i]; let n = if a.buy == 0.0 { g.gen_max_n(i).max(1.0) } else { a.buy }; let cost = g.gen_cost_n(i, n); let ok = cost.lte(g.s.energy) && g.gen_allowed(i);
        let prod = g.c.prod[i].mulf(g.c.sp); let next = g.next_mile(own); let prev = if own >= 25.0 { [0.0, 25.0, 50.0, 100.0, 150.0, 200.0, 250.0, 300.0].iter().rev().find(|t| **t <= own).copied().unwrap_or(0.0) } else { 0.0 };
        let e = if ok { String::new() } else { eta(g, cost) }; let hue = App::gen_hue(i);
        a.gfx.panel(x, y, w, 74.0, c("p1")); a.spr(&data().gen_spr[i], x + 10.0, y + 14.0, 40.0);
        a.gfx.text(&gen.n, x + 60.0, y + 9.0, 15.0, WHITE, 1); let ow = format!("×{}", group(own as i64)); a.gfx.text(&ow, x + 64.0 + a.gfx.tw(&gen.n, 15.0, 1), y + 10.0, 14.0, hue, 2);
        a.gfx.text(&format!("{}/s", fmt(prod)), x + 60.0, y + 30.0, 13.0, c("cyan"), 2);
        a.gfx.bar(x + 60.0, y + 50.0, w - 190.0, 10.0, ((own - prev) / (next - prev)) as f32, hue); a.gfx.text(&format!("{}", next), x + w - 124.0, y + 50.0, 10.0, c("faint"), 2);
        let lab = format!("{}{}\n{}", if a.buy == 0.0 { format!("+{} ", group(n as i64)) } else if n > 1.0 { format!("+{} ", n) } else { String::new() }, fmt(cost), if e.is_empty() { "Buy".into() } else { e });
        if a.btn(x + w - 116.0, y + 13.0, 106.0, 48.0, &lab, if ok { "on" } else { "" }, ok) { if a.g.buy_gen(i, a.buy) > 0.0 { a.g.calc(); sfx(a, "buy") } }
        if a.down && !a.dragging && a.hold_secs() > 0.35 && a.hover(Rect::new(x + w - 116.0, y + 13.0, 106.0, 48.0)) && ok && (a.time * 10.0) as i64 % 2 == 0 { a.g.buy_gen(i, a.buy); }
        y += 82.0;
    }
    y - y0
}

// ============================ UPGRADES ============================
fn upgrades(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let sub = a.sub("upgrades");
    let own = data().perm.iter().filter(|u| a.g.shop_owned("pm", &u.id)).count();
    let (h, pick) = a.chips(y, &[("perm", format!("Permanent {}/{}", own, data().perm.len()), false), ("tree", "Upgrades".into(), false), ("research", "Research".into(), !a.g.tier_open(0)), ("auto", "Automation".into(), false)], &sub);
    if let Some(p) = pick { if p != "research" || a.g.tier_open(0) { a.set_sub("upgrades", &p) } else { a.toast("Research opens after your first Rebirth", "lock", c("faint")) } }
    y += h;
    match sub.as_str() {
        "perm" => {
            y += muted(a, y, "Permanent upgrades cost energy and are never reset, not even by Ultra Prestige.");
            if a.btn(a.x0 + 8.0, y, a.cw - 16.0, 38.0, if a.g.s.set.perm_bar > 0.0 { "Hide the gold bar on the Engine screen" } else { "Show the gold bar on the Engine screen" }, "ghost", true) { a.g.s.set.perm_bar = if a.g.s.set.perm_bar > 0.0 { 0.0 } else { 1.0 } } y += 46.0;
            for u in data().perm.clone() { let owned = a.g.shop_owned("pm", &u.id); let ok = a.g.s.energy.gte(u.c); let (h, cl) = item(a, y, "crown", &u.n, &u.d, &fmt(u.c), ok, owned, ""); if cl { a.g.buy_perm(&u.id); sfx(a, "upg") } y += h; }
        }
        "research" => {
            y += muted(a, y, &format!("{} Research Points · +{}/s. Research is never reset by Rebirth.", fmt(a.g.s.rp), fmtf(a.g.rp_rate())));
            for (i, r) in data().res.clone().iter().enumerate() {
                let owned = a.g.s.res.contains_key(&r.id); let avail = a.g.res_avail(i); let ok = avail && a.g.s.rp.gte(r.c);
                let ex = if !a.g.tier_open(r.tier) { ["Opens after your first Rebirth", "Opens after your first Singularity", "Opens after your first Ascension", "Opens after your first Transcendence"][r.tier].to_string() } else if !owned && !avail { "Needs the research before it".into() } else { String::new() };
                let (h, cl) = item(a, y, "flask", &r.n, &r.d, &format!("{} RP", fmt(r.c)), ok, owned, &ex); if cl { a.g.buy_res(i); sfx(a, "upg") } y += h;
            }
        }
        "auto" => y += auto_page(a, y),
        _ => {
            let cats: Vec<(&str, String, bool)> = data().ucats.iter().map(|(k, n)| (k.as_str(), n.clone(), false)).collect(); let cur = a.cat.clone();
            let (h, pick) = a.chips(y, &cats, &cur); if let Some(p) = pick { a.cat = p } y += h;
            let owned = data().upg.iter().filter(|u| a.g.has_u(&u.id)).count();
            a.gfx.text(&format!("{}/{} owned", owned, data().upg.len()), a.x0 + 14.0, y + 12.0, 14.0, c("mist"), 0);
            if a.btn(a.x0 + a.cw - 200.0, y, 190.0, 40.0, "Buy all affordable (U)", "on", true) { let n = a.g.buy_all_upg(); if n > 0 { a.toast(format!("Bought {n} upgrade{}", if n > 1 { "s" } else { "" }), "arrowup", c("cyan")); sfx(a, "upg") } else { sfx(a, "bad") } }
            y += 48.0;
            if a.g.in_c("noupg") { y += muted(a, y, "Upgrades are disabled in this challenge."); }
            let mut list: Vec<_> = data().upg.iter().filter(|u| u.cat == a.cat).cloned().collect(); list.sort_by(|p, q| (p.row, p.col).partial_cmp(&(q.row, q.col)).unwrap());
            // available first, then locked, then owned
            let rank = |g: &Game, u: &crate::data::Upg| if g.has_u(&u.id) { 2 } else if g.upg_avail(u) { 0 } else { 1 };
            list.sort_by_key(|u| rank(&a.g, u));
            let hub = match a.cat.as_str() { "gen" => "gear", "prod" => "sparkle", "eff" => "arrows", "tap" => "hand", "auto" => "cycle", _ => "bolt" };
            for u in list.iter().take(60) {
                let owned = a.g.has_u(&u.id); let avail = a.g.upg_avail(u); let cost = a.g.upg_cost(u); let ok = avail && cost.lte(a.g.s.energy) && !a.g.in_c("noupg");
                let ex = if owned || avail { String::new() } else if !u.req.iter().all(|r| a.g.has_u(r)) { format!("Needs {}", u.req.iter().filter(|r| !a.g.has_u(r)).filter_map(|r| data().upg.iter().find(|x| &x.id == r)).map(|x| x.n.clone()).collect::<Vec<_>>().join(", ")) } else { u.cond.as_ref().map(Game::cond_text).unwrap_or_default() };
                let lab = if ok || avail { format!("{}\n{}", fmt(cost), eta(&a.g, cost)) } else { fmt(cost) };
                let (h, cl) = item(a, y, hub, &u.n, &u.d, lab.trim_end(), ok, owned, &ex); if cl { if a.g.buy_upg(&u.id) { sfx(a, "upg") } } y += h;
            }
        }
    }
    y - y0
}
fn auto_page(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let (x, w) = (a.x0 + 8.0, a.cw - 16.0); let keys = a.g.auto_keys();
    let pf = ["autoReb", "autoSing", "autoAsc", "autoTra", "autoInf"].iter().any(|f| a.g.fx.f(f));
    if keys.is_empty() && !pf && !a.g.bh_on() { return muted(a, y, "Automation upgrades in the Automation branch of the upgrade tree add auto-tap, autobuyers and more. They show up here.") }
    let row = |a: &mut App, y: f32, label: &str, on: bool| -> (f32, bool) { a.gfx.panel(x, y, w, 44.0, c("p1")); a.gfx.text(label, x + 12.0, y + 14.0, 14.0, WHITE, 0); let t = a.toggle(x + w - 60.0, y + 10.0, on); (52.0, t) };
    for k in keys {
        let on = if let Some(i) = k.strip_prefix('g').and_then(|s| s.parse::<usize>().ok()) { a.g.s.auto.gens[i] > 0.0 } else { a.g.s.auto.on(&k) };
        let (h, t) = row(a, y, &Game::auto_name(&k), on); if t { if let Some(i) = k.strip_prefix('g').and_then(|s| s.parse::<usize>().ok()) { a.g.s.auto.gens[i] = if on { 0.0 } else { 1.0 } } else { a.g.s.auto.set_on(&k, !on) } } y += h;
    }
    if a.g.bh_on() {
        a.gfx.panel(x, y, w, 76.0, c("p1")); a.gfx.text(&format!("Feed the Black Hole {}% of your energy per second", a.g.s.auto.feed), x + 12.0, y + 12.0, 13.0, WHITE, 0);
        for (i, p) in [0.0, 1.0, 5.0, 10.0, 25.0].iter().enumerate() { if a.btn(x + 12.0 + i as f32 * 62.0, y + 34.0, 56.0, 32.0, &format!("{p}%"), if a.g.s.auto.feed == *p { "vio" } else { "ghost" }, true) { a.g.s.auto.feed = *p } } y += 84.0;
    }
    for (k, fl) in [("reb", "autoReb"), ("sing", "autoSing"), ("asc", "autoAsc"), ("tra", "autoTra"), ("inf", "autoInf")] {
        if !a.g.fx.f(fl) { continue }
        let nm = LAYERS.iter().find(|l| l.0 == k).unwrap().1; let on = a.g.s.auto.on(k); let m = a.g.s.auto.rule(k, "M"); let m = if m.is_empty() { "gain".to_string() } else { m };
        a.gfx.panel(x, y, w, 118.0, c("p1")); a.gfx.text(&format!("Auto-{}", nm), x + 12.0, y + 12.0, 15.0, WHITE, 1); if a.toggle(x + w - 60.0, y + 10.0, on) { a.g.s.auto.set_on(k, !on) }
        for (i, (mk, ml)) in [("gain", "On gain"), ("time", "On a timer"), ("mult", "Multiple")].iter().enumerate() { if a.btn(x + 12.0 + i as f32 * 104.0, y + 40.0, 98.0, 30.0, ml, if m == *mk { "vio" } else { "ghost" }, true) { a.g.s.auto.set_rule(k, "M", mk) } }
        let (lab, val, key) = match m.as_str() { "time" => ("Every N minutes of the run", a.g.s.auto.rule(k, "T"), "T"), "mult" => ("When the gain is N× what you have", a.g.s.auto.rule(k, "X"), "X"), _ => ("When the gain reaches at least", a.g.s.auto.at(k).to_string(), "At") };
        a.gfx.text(lab, x + 12.0, y + 86.0, 12.0, c("mist"), 0);
        if let Some(v) = a.input_field(&format!("auto{k}{key}"), x + w - 150.0, y + 78.0, 138.0, &val) { if key == "At" { *a.g.s.auto.at_mut(k) = v } else { a.g.s.auto.set_rule(k, key, &v) } }
        y += 126.0;
    }
    y - y0
}

// ============================ PRESTIGE ============================
fn prestige(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let sub = a.sub("prestige"); let (x, w) = (a.x0 + 8.0, a.cw - 16.0);
    let seen = |a: &App, k: &str| match k { "reb" => true, "sing" => a.g.s.cnt.reb > 0.0, "asc" => a.g.s.cnt.sing > 0.0, "tra" => a.g.s.cnt.asc > 0.0, "inf" => a.g.s.cnt.tra > 0.0, "ult" => a.g.s.cnt.inf > 0.0, "chal" => a.g.chal_open(), _ => false };
    let list: Vec<(&str, String, bool)> = [("reb", "Rebirth"), ("sing", "Singularity"), ("asc", "Ascension"), ("tra", "Transcendence"), ("inf", "Infinity"), ("ult", "Ultra"), ("chal", "Challenges")].iter().map(|(k, n)| (*k, n.to_string(), !seen(a, k))).collect();
    let (h, pick) = a.chips(y, &list, &sub); if let Some(p) = pick { if seen(a, &p) { a.set_sub("prestige", &p) } else { a.toast("Reach the layer before it first", "lock", c("faint")) } } y += h;
    if sub == "chal" { return y - y0 + chal_page(a, y) }
    let k = sub.as_str(); let (_, n, cur, _) = *LAYERS.iter().find(|l| l.0 == k).unwrap();
    let (amount, spr, eff) = { let s = &a.g.s; match k { "reb" => (s.frag, "diamond", format!("production ×{}", fmt(s.frag.mulf(0.05 + 0.02 * a.g.perk_lv("reso")).addf(1.0)))), "sing" => (s.sing, "hole", format!("production ×{}", fmt(s.sing_total.addf(1.0).pow(1.5)))),
        "asc" => (s.cores, "arrowup", format!("production ×{}", fmt(s.cores_total.addf(1.0).pow(1.5)))), "tra" => (s.ve, "spiral", format!("production ×{}", fmt(s.ve_total.addf(1.0).pow(2.0)))), "inf" => (s.ip, "inf", format!("production ×{}", fmt(s.ip_total.addf(1.0).pow(2.0)))), _ => (s.up, "crown", format!("production ×{}", fmt(s.up_total.addf(1.0).pow(4.0)))) } };
    // the portal: currency in a glowing ring
    let cx = a.x0 + a.cw / 2.0; let ring = 92.0; let cy = y + ring + 10.0; let gain = a.g.layer_gain(k); let can = a.g.can_prestige(k);
    let prog = match k { "reb" => (oom(a.g.s.run_e) / a.g.reb_req().log10()).min(1.0), "sing" => (oom(a.g.s.run_e) / 30.0).min(1.0), "asc" => (oom(a.g.s.run_e) / 75.0).min(1.0), "tra" => (oom(a.g.s.run_e) / 125.0).min(1.0), "inf" => (oom(a.g.s.run_e) / 308.25).min(1.0), _ => (a.g.s.ip_total.maxf(1.0).log10() / data().ult_ip.log10()).min(1.0) };
    let col = match k { "reb" => c("cyan"), "sing" => hex("#ff8c3f"), "asc" => c("gold"), "tra" => c("mag"), "inf" => c("vio"), _ => c("gold") };
    for i in 0..72 { let an = i as f64 / 72.0 * 6.283 - 1.5708 + a.time * 0.15; let lit = (i as f64) < prog * 72.0; let (px, py) = (cx + an.cos() as f32 * ring, cy + an.sin() as f32 * ring); a.gfx.rect(px - 4.0, py - 4.0, 8.0, 8.0, INK); a.gfx.rect(px - 3.0, py - 3.0, 6.0, 6.0, if lit { col } else { c("p3") }) }
    a.spr(spr, cx - 24.0, cy - 64.0, 48.0); let t = fmt(amount); a.gfx.text_c(&t, cx + 2.0, cy - 6.0, 32.0, INK, 2); a.gfx.text_c(&t, cx, cy - 8.0, 32.0, WHITE, 2);
    a.gfx.text_c(cur, cx, cy + 30.0, 13.0, c("mist"), 0); a.gfx.text_c(&eff, cx, cy + 48.0, 12.0, col, 0);
    y = cy + ring + 18.0;
    // the prestige button and its description
    let desc = data().ldesc[k].clone(); let d1 = desc[1].as_str().unwrap_or("").to_string(); let d2 = desc[2].as_str().unwrap_or("").to_string();
    let bh = 46.0 + a.gfx.wrap_h(&d1, w - 28.0, 13.0, 0) + a.gfx.wrap_h(&d2, w - 28.0, 13.0, 0) + 70.0; a.gfx.panel(x, y, w, bh, c("p1"));
    let lab = if can { format!("{} for\n{} {}", n, fmt(gain), cur) } else if a.g.s.chal.a.is_some() { "Leave the challenge first".into() } else { format!("Needs {}", a.g.layer_req(k)) };
    if a.btn(x + 12.0, y + 12.0, w - 24.0, 58.0, &lab, if can { "on" } else { "" }, can) { if k == "ult" || a.g.s.cnt.reb < 3.0 && k == "reb" { a.sheet = Sheet::Prestige(k.into()) } else { do_prestige(a, k) } }
    let mut yy = y + 78.0;
    let info = format!("{} / {} energy this run", fmt(a.g.s.run_e), match k { "reb" => fmt(a.g.reb_req()), "sing" => "1e30".into(), "asc" => "1e75".into(), "tra" => "1e125".into(), "inf" => "1.79e308".into(), _ => fmt(a.g.s.ip_total) });
    a.gfx.text_c(&info, cx, yy, 12.0, c("mist"), 2); yy += 20.0;
    yy += a.gfx.wrap(&d1, x + 14.0, yy, w - 28.0, 13.0, c("txt"), 0) + 4.0; a.gfx.wrap(&d2, x + 14.0, yy, w - 28.0, 13.0, c("faint"), 0);
    y += bh + 10.0;
    // the shop of this layer
    let (shop, sname) = match k { "reb" => ("perk", "Fragment perks"), "sing" => ("su", "Singularity upgrades"), "asc" => ("au", "Ascension upgrades"), "tra" => ("vu", "Void upgrades"), "inf" => ("inf", "The Infinity tree"), _ => ("uu", "Ultra upgrades") };
    if k == "sing" { y += bh_panel(a, y) }
    if k == "tra" { y += laws_panel(a, y) }
    y += title(a, y, sname) - 6.0;
    if a.btn(x + w - 104.0, y - 30.0, 100.0, 34.0, "Buy max", "on", true) { let n = a.g.shop_max(shop); if n > 0 { a.toast(format!("Bought {n} upgrade{}", if n > 1 { "s" } else { "" }), "check", c("green")); sfx(a, "upg") } else { a.toast("Nothing affordable yet", "cross", c("red")); sfx(a, "bad") } }
    y += 8.0;
    if shop == "perk" {
        for p in data().perks.clone() { let l = a.g.perk_lv(&p.id); let maxed = l >= p.max as f64; let cost = b(Game::perk_cost(&p.id, l)); let ok = !maxed && a.g.s.frag.gte(cost);
            let (h, cl) = item(a, y, "diamond", &format!("{}  {}/{}", p.n, l, p.max), &p.d, &format!("{} frag", fmt(cost)), ok, maxed, ""); if cl { a.g.buy_perk(&p.id); sfx(a, "upg") } y += h; }
    } else {
        let list = Game::shop_list(shop).clone(); let cur_v = a.g.shop_cur(shop); let cn = match shop { "su" => "sing", "au" => "cores", "vu" => "VE", "inf" => "IP", _ => "UP" };
        for u in list { let owned = a.g.shop_owned(shop, &u.id); let avail = shop != "inf" || a.g.inf_avail(&u.id) || owned; let ok = !owned && avail && cur_v.gte(u.c);
            let ex = if shop == "inf" && !avail { format!("Needs {}", u.req.iter().filter_map(|r| data().inf.iter().find(|x| &x.id == r)).map(|x| x.n.clone()).collect::<Vec<_>>().join(", ")) } else { String::new() };
            let sp = if u.id == "omega" { "crown" } else { spr };
            let (h, cl) = item(a, y, sp, &u.n, &u.d, &format!("{} {}", fmt(u.c), cn), ok, owned, &ex);
            if cl { if u.id == "omega" { a.sheet = Sheet::Confirm("Build the Omega Engine?".into(), "This beats the game. You can keep playing afterwards.".into(), "Build it".into(), "omega".into()) } else if a.g.shop_buy(shop, &u.id) { sfx(a, "upg") } }
            y += h; }
    }
    y - y0
}
pub fn do_prestige(a: &mut App, k: &str) { if a.g.prestige(k, false).is_some() { sfx(a, "prestige"); a.save() } }
fn bh_panel(a: &mut App, y: f32) -> f32 {
    if !a.g.bh_on() { return 0.0 } let (x, w) = (a.x0 + 8.0, a.cw - 16.0);
    a.gfx.panel(x, y, w, 118.0, hex("#221838")); a.spr("hole", x + 12.0, y + 14.0, 36.0);
    a.gfx.text("Black Hole", x + 58.0, y + 12.0, 16.0, hex("#ff8c3f"), 1); a.gfx.text(&format!("Mass {} · production ×{} (power {:.2})", fmt(a.g.s.bh), fmt(b(1.0 + oom(a.g.s.bh)).pow(a.g.bh_pow())), a.g.bh_pow()), x + 58.0, y + 34.0, 12.0, c("mist"), 0);
    a.gfx.text("Feed it energy:", x + 12.0, y + 62.0, 13.0, c("txt"), 0);
    for (i, p) in [0.1, 0.5, 1.0].iter().enumerate() { if a.btn(x + 130.0 + i as f32 * 80.0, y + 54.0, 74.0, 34.0, &format!("{}%", (p * 100.0) as i32), "vio", true) { a.g.feed_bh(*p); sfx(a, "ab") } }
    a.gfx.text("Automation can feed it a share every second (Upgrades › Automation).", x + 12.0, y + 96.0, 11.0, c("faint"), 0);
    126.0
}
fn laws_panel(a: &mut App, mut y: f32) -> f32 {
    if !a.g.laws_open() { return 0.0 } let y0 = y; let (x, w) = (a.x0 + 8.0, a.cw - 16.0);
    y += title(a, y, &format!("Reality Laws {}/{}", a.g.s.laws.len(), a.g.fx.law_slots)) - 4.0;
    y += muted(a, y, &if a.g.s.law_cd > 0.0 { format!("Reality settles in {}s", a.g.s.law_cd.ceil()) } else { "Tap a law to enact or repeal it. Each one is a boon with a cost.".into() });
    for l in data().laws.clone() { let on = a.g.law(&l.id); a.gfx.panel(x, y, w, 62.0, if on { hex("#2a1e4a") } else { c("p1") }); a.gfx.text(&l.n, x + 12.0, y + 8.0, 14.0, if on { c("mag") } else { WHITE }, 1); let (lb, ld) = (a.gfx.fit(&format!("+ {}", l.b), w - 84.0, 12.0, 0), a.gfx.fit(&format!("− {}", l.d), w - 84.0, 12.0, 0)); a.gfx.text(&lb, x + 12.0, y + 28.0, 12.0, c("green"), 0); a.gfx.text(&ld, x + 12.0, y + 44.0, 12.0, c("red"), 0);
        if a.toggle(x + w - 60.0, y + 18.0, on) { if a.g.toggle_law(&l.id) { sfx(a, "ab") } else { sfx(a, "bad") } } y += 70.0; }
    y - y0 + 6.0
}
fn chal_page(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let (x, w) = (a.x0 + 8.0, a.cw - 16.0);
    y += muted(a, y, "Challenges restart your run under a harsh rule. Reach the goal to earn a permanent reward that grows with each level (I, II, III).");
    let goals: Vec<String> = data().chal_goal.iter().map(|g| fmt(*g)).collect(); y += muted(a, y, &format!("Goals: I {} · II {} · III {} energy in the run.", goals[0], goals[1], goals[2]));
    for ch in data().chal.clone() {
        let done = a.g.s.chal.done.get(&ch.id).copied().unwrap_or(0.0); let active = a.g.s.chal.a.as_ref().map(|z| z.id == ch.id).unwrap_or(false);
        a.gfx.panel(x, y, w, 104.0, if active { hex("#3a1a2e") } else { c("p1") }); a.spr("swords", x + 12.0, y + 12.0, 28.0);
        a.gfx.text(&format!("{} {}", ch.n, "★".repeat(0)), x + 50.0, y + 10.0, 15.0, WHITE, 1); a.gfx.text(&format!("Level {}/3", done), x + w - 90.0, y + 12.0, 12.0, c("gold"), 2);
        let l1 = a.gfx.fit(&ch.d, w - 60.0, 12.0, 0); a.gfx.text(&l1, x + 50.0, y + 32.0, 12.0, c("mist"), 0); let l2 = a.gfx.fit(&format!("Reward: {}", ch.rw), w - 60.0, 12.0, 0); a.gfx.text(&l2, x + 50.0, y + 50.0, 12.0, c("green"), 0);
        for l in 1..=3u32 { let ok = a.g.s.chal.a.is_none() && done >= l as f64 - 1.0; if a.btn(x + 50.0 + (l - 1) as f32 * 82.0, y + 68.0, 76.0, 30.0, &format!("Start {}", "I".repeat(l as usize)), if done >= l as f64 { "green" } else if ok { "on" } else { "" }, ok) { a.sheet = Sheet::Confirm(format!("Start {} {}?", ch.n, "I".repeat(l as usize)), "This restarts your run (like a Rebirth without a reward) until you reach the goal or leave.".into(), "Start".into(), format!("chal:{}:{}", ch.id, l)) } }
        y += 112.0;
    }
    y - y0
}

// ============================ WORLDS ============================
fn worlds(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let sub = a.sub("worlds");
    let (h, pick) = a.chips(y, &[("dims", "Dimensions".into(), a.g.s.cnt.asc < 1.0), ("bosses", "Bosses".into(), a.g.s.cnt.reb < 1.0), ("arts", "Artifacts".into(), false)], &sub);
    if let Some(p) = pick { let lock = (p == "dims" && a.g.s.cnt.asc < 1.0) || (p == "bosses" && a.g.s.cnt.reb < 1.0); if lock { a.toast(if p == "dims" { "Dimensions open after your first Ascension" } else { "Bosses appear after your first Rebirth" }, "lock", c("faint")) } else { a.set_sub("worlds", &p) } }
    y += h;
    match sub.as_str() { "bosses" => y += bosses(a, y), "arts" => y += arts(a, y), _ => y += dims(a, y) }
    y - y0
}
fn dims(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let (x, w) = (a.x0 + 8.0, a.cw - 16.0); let ds = data().dims.clone();
    if a.g.s.cnt.asc < 1.0 { a.gfx.panel(x, y, w, 230.0, c("p1")); a.world("material", a.x0 + a.cw / 2.0 - 64.0, y + 12.0, 128.0, true); a.gfx.text_c("Twelve dimensions await", a.x0 + a.cw / 2.0, y + 150.0, 18.0, WHITE, 1); a.gfx.wrap("Your Engine runs on an atom. Reach 1e75 energy in one run and Ascend to earn Ascension Cores, then come back here.", x + 14.0, y + 176.0, w - 28.0, 13.0, c("mist"), 0); return 240.0 }
    if let Some(i) = a.dim_sel { return dim_detail(a, y, i) }
    // how to unlock the next one
    if let Some(nx) = (0..ds.len()).find(|&i| !a.g.dim_on(&ds[i].id)) {
        let needs = a.g.dim_needs(nx); let h = 40.0 + needs.len() as f32 * 22.0 + 44.0; a.gfx.panel(x, y, w, h, c("p1"));
        a.gfx.text(&format!("Next: {}", ds[nx].n), x + 12.0, y + 10.0, 15.0, WHITE, 1); let mut yy = y + 36.0;
        for (t, ok, _, v) in &needs { a.spr(if *ok { "check" } else { "lock" }, x + 12.0, yy - 1.0, 14.0); a.gfx.text(&format!("{}{}", t, if !ok && !v.is_empty() { format!(" ({})", v) } else { String::new() }), x + 32.0, yy, 13.0, if *ok { c("green") } else { c("txt") }, 0); yy += 22.0 }
        let ok = a.g.dim_unlock_ok(nx) && a.g.s.cores.gtef(ds[nx].cost); if a.btn(x + 12.0, yy + 2.0, w - 24.0, 38.0, &format!("Unlock {} ({} cores)", ds[nx].n, ds[nx].cost), if ok { "on" } else { "" }, ok) { if a.g.unlock_dim(nx) { a.flash = Some(crate::ui::Flash { title: "Your engine grew".into(), sub: data().scale_name.iter().find(|s| s.0 == ds[nx].id).map(|s| s.1.clone()).unwrap_or_default(), col: hex(&ds[nx].col), t: 0.0 }); sfx(a, "prestige") } }
        y += h + 8.0;
    }
    for (i, d) in ds.iter().enumerate() {
        let on = a.g.dim_on(&d.id); a.gfx.panel(x, y, w, 84.0, c("p1")); a.world(&d.id, x + 10.0, y + 10.0, 64.0, !on);
        a.gfx.text(&d.n, x + 84.0, y + 10.0, 16.0, if on { hex(&d.col) } else { c("faint") }, 1);
        let tw = w - 96.0; if on { let l1 = a.gfx.fit(&format!("{} {} · +{}/s", fmt(a.g.s.dims[&d.id].r), d.res, fmt(a.g.dim_rate(&d.id))), tw, 12.0, 2); a.gfx.text(&l1, x + 84.0, y + 34.0, 12.0, c("txt"), 2); let l2 = a.gfx.fit(&d.eff, tw, 12.0, 0); a.gfx.text(&l2, x + 84.0, y + 54.0, 12.0, c("mist"), 0); }
        else { let l = a.gfx.fit(&d.mech, tw, 12.0, 0); a.gfx.text(&l, x + 84.0, y + 34.0, 12.0, c("faint"), 0); a.gfx.text("Locked", x + 84.0, y + 54.0, 12.0, c("faint"), 0); }
        if on && a.hit(Rect::new(x, y, w, 84.0)) { a.dim_sel = Some(i); a.scroll.insert(a.scroll_key(), 0.0); }
        y += 92.0;
    }
    y - y0
}
fn dim_detail(a: &mut App, mut y: f32, i: usize) -> f32 {
    let y0 = y; let (x, w) = (a.x0 + 8.0, a.cw - 16.0); let d = data().dims[i].clone(); let id = d.id.clone();
    if a.btn(x, y, 100.0, 36.0, "‹ Back", "ghost", true) { a.dim_sel = None } y += 44.0;
    a.gfx.panel(x, y, w, 150.0, c("p1")); a.world(&id, x + 10.0, y + 10.0, 128.0, false);
    a.gfx.text(&d.n, x + 150.0, y + 12.0, 20.0, hex(&d.col), 1); let dd = a.g.s.dims[&id].clone();
    a.gfx.text(&format!("{} {}", fmt(dd.r), d.res), x + 150.0, y + 42.0, 16.0, WHITE, 2); a.gfx.text(&format!("+{}/s · ×{}", fmt(a.g.dim_rate(&id)), fmt(a.g.dim_mult(&id))), x + 150.0, y + 66.0, 12.0, c("cyan"), 2);
    a.gfx.wrap(&d.mech, x + 150.0, y + 86.0, w - 160.0, 12.0, c("mist"), 0); a.gfx.wrap(&d.eff, x + 150.0, y + 118.0, w - 160.0, 12.0, c("green"), 0);
    y += 158.0;
    if id == "quantum" { let ok = a.g.s.qr.cd <= 0.0; if a.btn(x, y, w, 40.0, &if ok { format!("Observe (lock ×{:.2} for 60s)", a.g.s.qr.m) } else { format!("Observe ({}s)", a.g.s.qr.cd.ceil()) }, if ok { "vio" } else { "" }, ok) { if a.g.observe() { a.toast(format!("Observed: fluctuation locked at ×{:.2}", a.g.s.qr.m), "eye", c("vio")) } } y += 48.0; }
    if id == "temporal" { let ok = a.g.s.tskip_cd <= 0.0 && dd.r.gtef(10.0); if a.btn(x, y, w, 40.0, &if a.g.s.tskip_cd > 0.0 { format!("Time Skip ({}s)", a.g.s.tskip_cd.ceil()) } else { "Time Skip: spend half your Time for energy".into() }, if ok { "vio" } else { "" }, ok) { a.g.time_skip(); } y += 48.0; }
    for k in 0..3 { let g = dd.g[k].clone(); let cost = Game::dim_gen_cost(&dd, k); let ok = dd.r.gte(cost);
        a.gfx.panel(x, y, w, 62.0, c("p1")); a.gfx.text(&format!("{} ×{}", data().dgen_n[k], g.b), x + 12.0, y + 10.0, 15.0, WHITE, 1); a.gfx.text(&format!("amount {} · ×2 every 10 bought", fmt(Game::dim_eff(&dd, k))), x + 12.0, y + 34.0, 12.0, c("mist"), 2);
        if a.btn(x + w - 190.0, y + 10.0, 104.0, 42.0, &fmt(cost), if ok { "on" } else { "" }, ok) { a.g.buy_dim_gen(&id, k, false); sfx(a, "buy") }
        if a.btn(x + w - 80.0, y + 10.0, 70.0, 42.0, "Max", if ok { "on" } else { "" }, ok) { a.g.buy_dim_gen(&id, k, true); sfx(a, "buy") }
        y += 70.0; }
    for (k, n, desc) in [("amp", "Amplify", "Dimension production ×2"), ("reso", "Resonance", "Dimension effect strength ×1.25"), ("link", "Link", "Production ×(1 + OoM of the previous world's resource)")] {
        let dd = a.g.s.dims[&id].clone(); let lvl = match k { "amp" => dd.amp, "reso" => dd.reso, _ => dd.link }; let maxed = lvl >= Game::dupg_max(k); let cost = Game::dupg_cost(&dd, k); let ok = !maxed && dd.r.gte(cost);
        let (h, cl) = item(a, y, "arrowup", &format!("{} ({})", n, lvl), desc, &format!("{} {}", fmt(cost), d.res), ok, maxed, ""); if cl { a.g.buy_dim_upg(&id, k); sfx(a, "upg") } y += h; }
    y - y0
}
fn bosses(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let (x, w) = (a.x0 + 8.0, a.cw - 16.0);
    y += muted(a, y, &format!("Bosses take your production as damage, ×{} (tapping hits 10× harder). Each one beaten gives ×3 production forever.", fmt(a.g.boss_dmg())));
    for bo in data().bosses.clone() {
        let i = bo.i; let unl = a.g.boss_unlocked(i); let killed = a.g.s.boss.k.get(&i.to_string()).copied().unwrap_or(0.0); let cd = a.g.s.boss.cd.get(&i.to_string()).copied().unwrap_or(0.0); let fighting = a.g.s.boss.f.as_ref().map(|f| f.i == i).unwrap_or(false);
        if !unl && i > 0 && !a.g.boss_unlocked(i - 1) && killed == 0.0 { a.gfx.panel(x, y, w, 50.0, c("p1")); a.spr("lock", x + 14.0, y + 13.0, 24.0); a.gfx.text(if i >= 10 && !a.g.fx.f("broken") { "Past Infinity: Break Infinity first" } else { "Beat the boss before it" }, x + 50.0, y + 17.0, 13.0, c("faint"), 0); y += 58.0; continue }
        a.gfx.panel(x, y, w, 96.0, if fighting { hex("#2a1530") } else { c("p1") }); a.model(&format!("boss_{i}"), x + 4.0, y + 4.0, 88.0);
        a.gfx.text(&bo.n, x + 98.0, y + 10.0, 16.0, if killed > 0.0 { c("green") } else { WHITE }, 1); a.gfx.text(&format!("{} HP{}", fmt(bo.hp), if killed > 0.0 { format!(" · beaten {}×", killed) } else { String::new() }), x + 98.0, y + 32.0, 12.0, c("mist"), 2);
        let secs = bo.hp.div(a.g.c.eps.mul(a.g.boss_dmg()).mulf(a.g.c.sp).maxf(1e-300)).to_f64(); a.gfx.text(&if secs < 60.0 { format!("You'd win in about {}", fmt_t(secs.ceil())) } else { "Too strong for now: you need 60s or less".into() }, x + 98.0, y + 50.0, 12.0, if secs < 60.0 { c("green") } else { c("red") }, 0);
        let ok = a.g.boss_ready(i); let lab = if fighting { "Fighting…".to_string() } else if cd > 0.0 { format!("Ready in {}", fmt_t(cd.ceil())) } else { "Fight".into() };
        if a.btn(x + 98.0, y + 66.0, 130.0, 26.0, &lab, if ok { "danger" } else { "" }, ok) { if a.g.start_boss(i) { a.tab = "engine".into(); sfx(a, "ab") } }
        y += 104.0;
    }
    y - y0
}
fn arts(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let (x, w) = (a.x0 + 8.0, a.cw - 16.0);
    y += muted(a, y, &format!("{}/{} found. Artifacts drop from bosses and random events, and stack up to level 5.", a.g.s.arts.len(), data().arts.len()));
    for ar in data().arts.clone() { let l = a.g.s.arts.get(&ar.id).copied().unwrap_or(0.0); let spr = data().art_spr[&ar.id].as_str().unwrap_or("gem").to_string();
        if l > 0.0 || !ar.secret { a.gfx.panel(x, y, w, 58.0, c("p1")); if l > 0.0 { a.spr(&format!("{spr}@r{}", ar.r), x + 12.0, y + 13.0, 32.0); } else { a.spr("question", x + 12.0, y + 13.0, 32.0); }
            let rc = [c("mist"), c("cyan"), c("vio"), c("gold"), c("mag")][ar.r]; a.gfx.text(if l > 0.0 { &ar.n } else { "Undiscovered" }, x + 56.0, y + 9.0, 14.0, if l > 0.0 { WHITE } else { c("faint") }, 1);
            a.gfx.text(&format!("{}{}", data().rar[ar.r], if l > 0.0 { format!(" · Lv {}", l) } else { String::new() }), x + w - 130.0, y + 10.0, 12.0, rc, 0);
            if l > 0.0 { let d = a.gfx.fit(&ar.d, w - 66.0, 12.0, 0); a.gfx.text(&d, x + 56.0, y + 31.0, 12.0, c("mist"), 0); } y += 66.0 } }
    y - y0
}

// ============================ CREW ============================
fn crew(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let sub = a.sub("crew");
    let (h, pick) = a.chips(y, &[("story", "Story".into(), false), ("pets", "Pets".into(), false), ("cast", "Cast".into(), false)], &sub); if let Some(p) = pick { a.set_sub("crew", &p) } y += h;
    match sub.as_str() { "pets" => y += pets(a, y), "cast" => y += cast(a, y), _ => y += story(a, y) }
    y - y0
}
fn story(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let (x, w) = (a.x0 + 8.0, a.cw - 16.0); let chs = &data().chapters;
    if a.g.story_done() { y += title(a, y, "The whole story is done"); y += muted(a, y, "Every chapter is complete. Rewatch any of them below."); }
    else {
        let ci = a.g.chapter_i(); let ch = chs[ci].clone(); let ms = data().missions[ci].clone();
        let h = 150.0 + ms.len() as f32 * 44.0 + 60.0; a.gfx.panel(x, y, w, h, c("p1"));
        a.model(&format!("char_{}", ch.who), x + 6.0, y + 6.0, 112.0);
        a.gfx.text(&format!("Chapter {}", ci + 1), x + 124.0, y + 14.0, 13.0, c("gold"), 0); a.gfx.text(&ch.n, x + 124.0, y + 32.0, 18.0, WHITE, 1); a.gfx.wrap(&ch.syn, x + 124.0, y + 58.0, w - 134.0, 12.0, c("mist"), 0);
        let mut yy = y + 128.0;
        for m in &ms { let (v, n) = a.g.mission_val(m); let ok = v >= n; a.spr(if ok { "check" } else { "scroll" }, x + 12.0, yy, 18.0); a.gfx.text(&m.t, x + 38.0, yy + 2.0, 13.0, if ok { c("green") } else { c("txt") }, 0); a.gfx.text_r(&format!("{} / {}", group(v.min(n).floor() as i64), group(n as i64)), x + w - 14.0, yy + 2.0, 12.0, c("mist"), 2); a.gfx.bar(x + 38.0, yy + 22.0, w - 52.0, 10.0, (v / n) as f32, if ok { c("green") } else { c("gold") }); yy += 44.0 }
        let bw = (w - 36.0) / 3.0;
        if a.btn(x + 12.0, yy + 4.0, bw, 40.0, "Opening", "ghost", true) { a.dlg = Some((ci, "intro".into(), 0)) }
        let half = a.g.missions_half(ci) && !ch.mid.is_empty(); if a.btn(x + 18.0 + bw, yy + 4.0, bw, 40.0, "Middle", "ghost", half) { a.dlg = Some((ci, "mid".into(), 0)) }
        let done = a.g.missions_done(ci); if a.btn(x + 24.0 + bw * 2.0, yy + 4.0, bw, 40.0, "Complete", if done { "gold" } else { "" }, done) { if a.g.claim_chapter() { a.dlg = Some((ci, "outro".into(), 0)); sfx(a, "upg"); a.save() } }
        y += h + 10.0;
        if !a.g.s.story.seen.contains_key(&format!("{ci}i")) && a.dlg.is_none() { a.g.s.story.seen.insert(format!("{ci}i"), 1.0); a.dlg = Some((ci, "intro".into(), 0)) }
    }
    let done_n = (a.g.s.story.ch as usize).min(chs.len());
    if done_n > 0 { y += title(a, y, "Completed chapters"); }
    for i in (0..done_n).rev() { let ch = chs[i].clone(); a.gfx.panel(x, y, w, 50.0, c("p1")); a.spr("scroll", x + 12.0, y + 13.0, 24.0); a.gfx.text(&format!("{}. {}", i + 1, ch.n), x + 46.0, y + 16.0, 14.0, WHITE, 0);
        if a.btn(x + w - 92.0, y + 8.0, 82.0, 34.0, "Watch", "ghost", true) { a.dlg = Some((i, "intro".into(), 0)) } y += 58.0; }
    y - y0
}
pub fn dialogue(a: &mut App) {
    let Some((ci, part, li)) = a.dlg.clone() else { return }; let ch = &data().chapters[ci];
    let lines = match part.as_str() { "mid" => &ch.mid, "outro" => &ch.outro, _ => &ch.intro };
    if li >= lines.len() { a.dlg = if part == "intro" && !ch.mid.is_empty() && false { Some((ci, "mid".into(), 0)) } else { None }; return }
    let (who, text) = lines[li].clone();
    a.gfx.rect(0.0, 0.0, a.lw, a.lh, alpha(INK, 0.55));
    let (x, w) = (a.x0 + 10.0, a.cw - 20.0); let th = a.gfx.wrap_h(&text, w - 28.0, 15.0, 0); let h = th + 80.0; let y = a.lh - 80.0 - h;
    a.model(&format!("char_{who}_talk"), x + w / 2.0 - 80.0, y - 166.0, 160.0);
    a.gfx.rect(x - 3.0, y - 3.0, w + 6.0, h + 6.0, INK); a.gfx.panel(x, y, w, h, c("p1"));
    let name = data().chars.iter().find(|c| c.0 == who).map(|c| c.1.clone()).unwrap_or(who.clone());
    a.gfx.text_sh(&name, x + 14.0, y + 12.0, 16.0, c("gold"), 1); a.gfx.wrap(&text, x + 14.0, y + 40.0, w - 28.0, 15.0, c("txt"), 0);
    a.gfx.text_r(&format!("{}/{} · tap to continue", li + 1, lines.len()), x + w - 12.0, y + h - 22.0, 11.0, c("faint"), 0);
    if a.btn(x + w - 70.0, y + 8.0, 60.0, 30.0, "Skip", "ghost", true) { a.dlg = None; return }
    if a.click || is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) { a.click = false; a.dlg = Some((ci, part, li + 1)) }
    a.click = false;
}
fn pets(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let (x, w) = (a.x0 + 8.0, a.cw - 16.0);
    if let Some(id) = a.g.s.pets.active.clone() { if let Some(o) = a.g.s.pets.own.get(&id).cloned() {
        let p = data().pets.iter().find(|p| p.id == id).unwrap().clone(); let st = Game::pet_stage(o.lvl);
        a.gfx.panel(x, y, w, 168.0, c("p1")); a.model(&format!("pet_{id}_{st}"), x + 6.0, y + 6.0, 140.0);
        a.gfx.text(&p.n, x + 154.0, y + 14.0, 20.0, hex(&p.col), 1); a.gfx.text(&format!("Lv {} · {}", o.lvl, data().pet_stage[st]), x + 154.0, y + 42.0, 13.0, c("gold"), 0);
        a.gfx.wrap(&p.d, x + 154.0, y + 62.0, w - 164.0, 12.0, c("mist"), 0); a.gfx.text(&p.bd, x + 154.0, y + 102.0, 12.0, c("green"), 0);
        let fc = a.g.feed_cost(&id); let ok = o.lvl < 30.0 && a.g.s.energy.gte(fc);
        if a.btn(x + 154.0, y + 120.0, w - 166.0, 40.0, &if o.lvl >= 30.0 { "Max level".to_string() } else { format!("Feed · {}", fmt(fc)) }, if ok { "on" } else { "" }, ok) { a.g.feed_pet(&id); sfx(a, "upg") }
        y += 176.0;
    } }
    y += title(a, y, &format!("Eggs {}/3", a.g.s.pets.eggs.len()));
    for (i, e) in a.g.s.pets.eggs.clone().iter().enumerate() {
        a.gfx.panel(x, y, w, 80.0, c("p1")); a.model(&format!("egg_{}", e.r), x + 6.0, y + 4.0, 72.0); a.gfx.text(&format!("{} egg", data().rar[e.r]), x + 86.0, y + 14.0, 15.0, WHITE, 1);
        let ready = e.t <= 0.0; a.gfx.text(&if ready { "Ready to hatch!".to_string() } else { format!("Hatches in {}", fmt_t(e.t.ceil())) }, x + 86.0, y + 38.0, 13.0, if ready { c("green") } else { c("mist") }, 0);
        if a.btn(x + w - 110.0, y + 18.0, 100.0, 44.0, "Hatch", if ready { "gold" } else { "" }, ready) { if let Some(pid) = a.g.hatch(i) { let n = data().pets.iter().find(|p| p.id == pid).unwrap().n.clone(); a.flash = Some(crate::ui::Flash { title: "Hatched!".into(), sub: n, col: c("mag"), t: 0.0 }); sfx(a, "event") } }
        y += 88.0;
    }
    let ec = a.g.egg_cost(); let ok = a.g.s.pets.eggs.len() < 3 && a.g.s.energy.gte(ec);
    if a.btn(x, y, w, 42.0, &format!("Buy an egg · {}", fmt(ec)), if ok { "on" } else { "" }, ok) { a.g.buy_egg(); sfx(a, "buy") } y += 50.0;
    y += title(a, y, &format!("Your pets {}/{}", a.g.s.pets.own.len(), data().pets.len()));
    let cols = 4; let cw = (w - (cols as f32 - 1.0) * 6.0) / cols as f32;
    for (k, p) in data().pets.clone().iter().enumerate() {
        let (cx, cy) = (x + (k % cols) as f32 * (cw + 6.0), y + (k / cols) as f32 * (cw + 36.0)); let own = a.g.s.pets.own.get(&p.id).cloned(); let act = a.g.s.pets.active.as_deref() == Some(&p.id);
        a.gfx.panel(cx, cy, cw, cw + 30.0, if act { hex("#2a2252") } else { c("p1") });
        if let Some(o) = &own { a.model(&format!("pet_{}_{}", p.id, Game::pet_stage(o.lvl)), cx + 4.0, cy + 4.0, cw - 8.0); a.gfx.text_c(&format!("{} {}", p.n, o.lvl), cx + cw / 2.0, cy + cw + 6.0, 12.0, if act { c("gold") } else { WHITE }, 0); if a.hit(Rect::new(cx, cy, cw, cw + 30.0)) { a.g.set_active_pet(&p.id); a.toast(format!("{} is your active pet", p.n), "heart", c("mag")) } }
        else { a.spr("question", cx + cw / 2.0 - 16.0, cy + cw / 2.0 - 16.0, 32.0); a.gfx.text_c(&data().rar[p.r], cx + cw / 2.0, cy + cw + 6.0, 12.0, c("faint"), 0); }
    }
    y += ((data().pets.len() + cols - 1) / cols) as f32 * (cw + 36.0);
    y - y0
}
fn cast(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let (x, w) = (a.x0 + 8.0, a.cw - 16.0); let reached = (a.g.s.story.ch as usize + 1).min(data().chapters.len());
    let met: Vec<String> = data().chapters[..reached].iter().flat_map(|c| c.intro.iter().chain(c.mid.iter()).chain(c.outro.iter()).map(|l| l.0.clone()).chain(std::iter::once(c.who.clone()))).collect();
    for (id, n, role) in data().chars.clone() { let seen = met.contains(&id);
        a.gfx.panel(x, y, w, 104.0, c("p1")); if seen { a.model(&format!("char_{id}"), x + 4.0, y + 4.0, 96.0); a.gfx.text(&n, x + 108.0, y + 26.0, 16.0, WHITE, 1); a.gfx.wrap(&role, x + 108.0, y + 50.0, w - 118.0, 13.0, c("mist"), 0); } else { a.spr("question", x + 30.0, y + 34.0, 36.0); a.gfx.text("Not met yet", x + 108.0, y + 40.0, 14.0, c("faint"), 0); }
        y += 112.0; }
    y - y0
}

// ============================ STATS ============================
fn stats(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let sub = a.sub("stats");
    let (h, pick) = a.chips(y, &[("stats", "Statistics".into(), false), ("ach", "Achievements".into(), false), ("badges", "Badges".into(), false), ("log", "Log".into(), false), ("set", "Settings".into(), false)], &sub); if let Some(p) = pick { a.set_sub("stats", &p) } y += h;
    match sub.as_str() { "ach" => y += achs(a, y), "badges" => y += badges(a, y), "log" => y += log_page(a, y), "set" => y += settings(a, y), _ => y += stat_page(a, y) }
    y - y0
}
fn date(ms: f64) -> String { chrono::DateTime::from_timestamp_millis(ms as i64).map(|d| d.with_timezone(&chrono::Local).format("%-d %B %Y").to_string()).unwrap_or_default() }
fn stat_page(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let (x, w) = (a.x0 + 8.0, a.cw - 16.0); let g = &a.g; let s = &g.s;
    let rows: Vec<(String, String)> = vec![("Started on".into(), date(s.started)), ("Days since start".into(), group(((now_ms() - s.started) / 864e5) as i64)), ("Beat the game".into(), if s.st.beat > 0.0 { date(s.st.beat) } else { "Not yet".into() }), ("Ultra Prestiges".into(), format!("{}", s.cnt.ult)),
        ("Time played".into(), fmt_t(s.st.play)), ("This run".into(), fmt_t(s.st.run)), ("Energy this run".into(), fmt(s.run_e)), ("Lifetime energy".into(), fmt(s.life_e)), ("Best energy/s".into(), fmt(s.st.max_eps)), ("Game speed".into(), format!("×{:.2}", g.c.sp)),
        ("Reactor taps".into(), group(s.st.taps as i64)), ("Critical taps".into(), group(s.st.crits as i64)), ("Upgrades bought".into(), format!("{}", s.st.upg)), ("Events claimed".into(), format!("{}", s.st.ev)), ("Abilities used".into(), format!("{}", s.st.ab)), ("Bosses defeated".into(), format!("{}", s.st.kills)),
        ("Rebirths".into(), format!("{}", s.cnt.reb)), ("Fastest Rebirth".into(), if s.cnt.reb > 0.0 && s.st.best_reb < 1e11 { fmt_t(s.st.best_reb) } else { "—".into() }), ("Fragments earned".into(), fmt(s.frag_life)), ("Singularities".into(), format!("{}", s.cnt.sing)), ("Ascensions".into(), format!("{}", s.cnt.asc)),
        ("Transcendences".into(), format!("{}", s.cnt.tra)), ("Infinities".into(), format!("{}", s.cnt.inf)), ("Challenges completed".into(), format!("{}", s.st.chal)), ("Dimensions unlocked".into(), format!("{}/{}", g.dims_unlocked(), data().dims.len())),
        ("Research done".into(), format!("{}/32", s.res.len())), ("Artifacts found".into(), format!("{}/{}", s.arts.len(), data().arts.len())), ("Achievements".into(), format!("{}/{}", g.ach_n, data().ach.len())), ("Offline cap".into(), format!("{}h", g.off_h())), ("Milestone multiplier".into(), format!("×{:.3}", g.c.mp))];
    let h = rows.len() as f32 * 24.0 + 20.0; a.gfx.panel(x, y, w, h, c("p1")); let mut yy = y + 12.0;
    for (k, v) in &rows { a.gfx.text(k, x + 14.0, yy, 13.0, c("mist"), 0); a.gfx.text_r(v, x + w - 14.0, yy, 13.0, WHITE, 2); yy += 24.0 } y += h + 10.0;
    // energy per second over the last two hours
    y += title(a, y, "Energy per second, last 2 hours"); a.gfx.panel(x, y, w, 120.0, c("p1"));
    let hist = a.hist.clone(); if hist.len() > 1 { let mx = hist.iter().map(|h| h.1).fold(0.0, f64::max).max(1.0); let mn = hist.iter().map(|h| h.1).fold(mx, f64::min);
        for (i, p) in hist.windows(2).enumerate() { let n = hist.len() as f32 - 1.0; let (x1, x2) = (x + 12.0 + i as f32 / n * (w - 24.0), x + 12.0 + (i + 1) as f32 / n * (w - 24.0)); let yy = |v: f64| y + 108.0 - ((v - mn) / (mx - mn).max(1e-9)) as f32 * 92.0; a.gfx.line(x1, yy(p[0].1), x2, yy(p[1].1), 2.0, c("cyan")) }
        a.gfx.text(&format!("1e{:.0}", mx), x + 12.0, y + 8.0, 11.0, c("faint"), 2); } else { a.gfx.text("Collecting…", x + 14.0, y + 50.0, 13.0, c("faint"), 0); }
    y += 128.0;
    // where production comes from
    y += title(a, y, "Where your production comes from"); let parts = a.g.mult_parts(); let h = parts.len() as f32 * 22.0 + 20.0; a.gfx.panel(x, y, w, h, c("p1")); let mut yy = y + 10.0;
    for (n, v) in parts { a.gfx.text(&n, x + 14.0, yy, 13.0, c("mist"), 0); a.gfx.text_r(&format!("×{}", fmt(v)), x + w - 14.0, yy, 13.0, if v.gtf(1.0) { c("green") } else if v.ltf(1.0) { c("red") } else { c("faint") }, 2); yy += 22.0 }
    y += h + 8.0;
    y - y0
}
fn achs(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let (x, w) = (a.x0 + 8.0, a.cw - 16.0);
    y += muted(a, y, &format!("{}/{} unlocked · each one +2% production", a.g.ach_n, data().ach.len()));
    for ac in data().ach.clone() { let done = a.g.s.ach.contains_key(&ac.id); let hid = ac.h && !done;
        a.gfx.panel(x, y, w, 50.0, if done { hex("#1b2a3a") } else { c("p1") }); a.spr(if done { "trophy" } else { "lock" }, x + 12.0, y + 13.0, 24.0);
        a.gfx.text(if hid { "???" } else { &ac.n }, x + 46.0, y + 8.0, 14.0, if done { c("gold") } else { c("faint") }, 1); let d = a.gfx.fit(if hid { "A hidden achievement" } else { &ac.d }, w - 56.0, 12.0, 0); a.gfx.text(&d, x + 46.0, y + 28.0, 12.0, c("mist"), 0); y += 56.0 }
    y - y0
}
fn badges(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let (x, w) = (a.x0 + 8.0, a.cw - 16.0);
    y += muted(a, y, &format!("{} badges · each tier +2% production", a.g.badge_n()));
    for bd in data().badges.clone() { let tier = a.g.s.badge.get(&bd.id).copied().unwrap_or(0.0) as usize; let v = a.g.badge_val(&bd.id);
        a.gfx.panel(x, y, w, 58.0, c("p1")); a.spr(&bd.spr, x + 12.0, y + 15.0, 28.0);
        a.gfx.text(&bd.n, x + 50.0, y + 8.0, 14.0, WHITE, 1); if tier > 0 { let (tn, tc) = &data().btier[tier - 1]; a.gfx.text(tn, x + 54.0 + a.gfx.tw(&bd.n, 14.0, 1), y + 9.0, 12.0, hex(tc), 0); }
        let next = bd.th.get(tier).copied(); a.gfx.text(&match next { Some(n) => format!("{} / {} {}", group(v.floor() as i64), group(n as i64), bd.what), None => format!("Maxed · {} {}", group(v.floor() as i64), bd.what) }, x + 50.0, y + 30.0, 12.0, c("mist"), 0);
        for t in 0..5 { let col = if t < tier { hex(&data().btier[t].1) } else { c("p3") }; a.gfx.rect(x + w - 80.0 + t as f32 * 14.0, y + 38.0, 10.0, 10.0, col) }
        y += 64.0 }
    y - y0
}
fn log_page(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let (x, w) = (a.x0 + 8.0, a.cw - 16.0);
    if a.log.is_empty() { return muted(a, y, "Notifications show up here with their time, so nothing gets missed.") }
    for (t, msg, spr) in a.log.clone().iter().rev().take(150) {
        let h = a.gfx.wrap_h(msg, w - 110.0, 13.0, 0) + 14.0; a.gfx.panel(x, y, w, h, c("p1")); a.spr(spr, x + 10.0, y + (h - 20.0) / 2.0, 20.0); a.gfx.wrap(msg, x + 40.0, y + 7.0, w - 110.0, 13.0, c("txt"), 0);
        let ago = (now_ms() - t) / 1000.0; a.gfx.text_r(&if ago < 60.0 { "just now".to_string() } else { format!("{} ago", fmt_t(ago)) }, x + w - 10.0, y + 7.0, 11.0, c("faint"), 0); y += h + 4.0 }
    y - y0
}
fn settings(a: &mut App, mut y: f32) -> f32 {
    let y0 = y; let (x, w) = (a.x0 + 8.0, a.cw - 16.0);
    y += title(a, y, "Numbers");
    let nots = [("suffix", "1.23M"), ("sci", "1.23e6"), ("eng", "123.4e3")]; let bw = (w - 12.0) / 3.0;
    for (i, (k, l)) in nots.iter().enumerate() { if a.btn(x + i as f32 * (bw + 6.0), y, bw, 40.0, l, if a.g.s.set.not == *k { "vio" } else { "ghost" }, true) { a.g.s.set.not = k.to_string(); crate::fmt::set_notation(k); a.g.s.sec.insert("notation".into(), serde_json::json!(1)); } }
    y += 52.0; y += title(a, y, "Sound");
    for (lab, key) in [("Sound effects", "sfx"), ("Music", "mus")] { let on = if key == "sfx" { a.g.s.set.sfx > 0.0 } else { a.g.s.set.mus > 0.0 }; a.gfx.panel(x, y, w, 44.0, c("p1")); a.gfx.text(lab, x + 12.0, y + 14.0, 14.0, WHITE, 0);
        let vol = if key == "sfx" { a.g.s.set.sv } else { a.g.s.set.mv }; for (j, v) in [0.25, 0.5, 0.75, 1.0].iter().enumerate() { if a.btn(x + w - 260.0 + j as f32 * 46.0, y + 8.0, 42.0, 28.0, &format!("{}", (v * 100.0) as i32), if (vol - v).abs() < 0.13 { "vio" } else { "ghost" }, true) { if key == "sfx" { a.g.s.set.sv = *v } else { a.g.s.set.mv = *v } crate::sound::sync(&a.g.s.set) } }
        if a.toggle(x + w - 60.0, y + 10.0, on) { if key == "sfx" { a.g.s.set.sfx = if on { 0.0 } else { 1.0 } } else { a.g.s.set.mus = if on { 0.0 } else { 1.0 } } crate::sound::sync(&a.g.s.set) } y += 52.0 }
    y += 8.0; y += title(a, y, "Graphics");
    a.gfx.panel(x, y, w, 44.0, c("p1")); a.gfx.text("Animations and effects", x + 12.0, y + 14.0, 14.0, WHITE, 0); let on = a.g.s.set.fx > 0.0; if a.toggle(x + w - 60.0, y + 10.0, on) { a.g.s.set.fx = if on { 0.0 } else { 1.0 } } y += 52.0;
    a.gfx.panel(x, y, w, 44.0, c("p1")); a.gfx.text(&format!("Screen size: {}", a.g.s.set.uiz.as_f64().map(|z| format!("{z}%")).unwrap_or("Auto".into())), x + 12.0, y + 14.0, 14.0, WHITE, 0);
    if a.btn(x + w - 186.0, y + 7.0, 56.0, 30.0, "Auto", "ghost", true) { a.g.s.set.uiz = serde_json::json!("auto") } if a.btn(x + w - 124.0, y + 7.0, 56.0, 30.0, "−", "ghost", true) { a.zoom(-10.0) } if a.btn(x + w - 62.0, y + 7.0, 56.0, 30.0, "+", "ghost", true) { a.zoom(10.0) } y += 52.0;
    if a.btn(x, y, w, 40.0, if a.full { "Leave full screen (F11)" } else { "Full screen (F11)" }, "ghost", true) { a.full = !a.full; set_fullscreen(a.full) } y += 50.0;
    y += title(a, y, "Your save");
    y += muted(a, y, &format!("Saved automatically every 20 seconds to {}. Export codes work in the web version too.", crate::save::dir().join("save.txt").display()));
    let bw = (w - 6.0) / 2.0;
    if a.btn(x, y, bw, 42.0, "Save now", "on", true) { a.save(); a.toast("Game saved", "check", c("green")) }
    if a.btn(x + bw + 6.0, y, bw, 42.0, "Open save folder", "ghost", true) { let d = crate::save::dir(); let _ = std::fs::create_dir_all(&d); #[cfg(windows)] { let _ = std::process::Command::new("explorer").arg(&d).spawn(); } #[cfg(not(windows))] { let _ = std::process::Command::new("xdg-open").arg(&d).spawn(); } } y += 50.0;
    if a.btn(x, y, bw, 42.0, "Export", "ghost", true) { a.g.s.sec.insert("export".into(), serde_json::json!(1)); let code = crate::save::export(&a.g.s); miniquad::window::clipboard_set(&code); a.sheet = Sheet::Export(code) }
    if a.btn(x + bw + 6.0, y, bw, 42.0, "Import", "ghost", true) { a.input.clear(); a.input_key = "import".into(); a.sheet = Sheet::Import } y += 50.0;
    if a.btn(x, y, w, 42.0, "Erase everything", "danger", true) { a.sheet = Sheet::Reset } y += 56.0;
    y += title(a, y, "About");
    if a.btn(x, y, bw, 40.0, "What's new", "ghost", true) { a.sheet = Sheet::WhatsNew } if a.btn(x + bw + 6.0, y, bw, 40.0, "Keyboard", "ghost", true) { a.sheet = Sheet::Keys } y += 48.0;
    if a.btn(x, y, w, 40.0, "Terms", "ghost", true) { a.sheet = Sheet::Terms } y += 48.0;
    a.gfx.text_c(&format!("Ten Cells {} · native Windows build", env!("CARGO_PKG_VERSION")), a.x0 + a.cw / 2.0, y, 12.0, c("faint"), 0); y += 24.0;
    y - y0
}

// ============================ SHEETS ============================
pub fn sheet(a: &mut App) {
    let sh = a.sheet.clone(); if a.last_sheet != sh { a.last_sheet = sh.clone(); a.sheet_scroll = 0.0 } a.gfx.rect(0.0, 0.0, a.lw, a.lh, alpha(INK, 0.6));
    let (x, w) = (a.x0 + 10.0, a.cw - 20.0);
    let body: (String, String) = match &sh {
        Sheet::Confirm(t, d, _, _) | Sheet::Info(t, d) => (t.clone(), d.clone()),
        Sheet::Export(code) => ("Export save".into(), format!("This code is on your clipboard. Keep it somewhere safe; Import brings it back here or in the web version.\n\n{}…", &code[..code.len().min(120)])),
        Sheet::Import => ("Import save".into(), "Paste a save code from this game or the web version (Ctrl+V), then press Load. This replaces your current game.".into()),
        Sheet::Welcome(t) => ("Welcome back".into(), t.clone()),
        Sheet::Terms => ("Terms".into(), "It's free. No accounts, no ads, no purchases. Play as much as you like.\n\nYour save lives on this computer, in Documents › Ten Cells. Use Export in Stats › Settings to keep a backup somewhere else.\n\nNo data collected. The game never connects to the internet. No tracking, no analytics, nothing leaves your computer.\n\nFlashing lights and sound. There are flashes and music. If they bother you, turn off Animations and effects or the sound in Stats › Settings, and take breaks.\n\nAs is. This is a hobby game provided as is, without any warranty. Updates may change balance or features, and in rare cases a save could be lost.\n\nPlay fair. Editing your save or cheating only spoils your own fun. Go for it if you want, just don't expect it to be supported.".into()),
        Sheet::Reset => ("Erase everything?".into(), "All progress, prestige layers, artifacts and achievements will be deleted. This can't be undone.".into()),
        Sheet::Keys => ("Keyboard".into(), "Space: tap the reactor\n1 to 6: switch tabs\nG: buy every energy maker you can\nU: buy all affordable upgrades\nA: fire every ready ability\n+ and −: screen size\nF or F11: full screen\nEsc: close this".into()),
        Sheet::Prestige(k) => { let (_, n, cur, _) = *LAYERS.iter().find(|l| l.0 == k).unwrap(); (format!("{n}?"), format!("You get {} {}.\n\n{}", fmt(a.g.layer_gain(k)), cur, data().ldesc[k.as_str()][2].as_str().unwrap_or(""))) }
        Sheet::WhatsNew => ("What's new".into(), data().changelog.iter().take(4).map(|c| format!("{} · {}\n{}", c.v, c.d, c.n.iter().map(|n| format!("• {n}")).collect::<Vec<_>>().join("\n"))).collect::<Vec<_>>().join("\n\n")),
        _ => (String::new(), String::new()),
    };
    let max_h = a.lh - 140.0; let th = a.gfx.wrap_h(&body.1, w - 28.0, 14.0, 0); let content = th + 70.0 + if sh == Sheet::Import { 40.0 } else { 0.0 }; let h = content.min(max_h) + 60.0; let y = a.lh - 70.0 - h;
    a.gfx.rect(x - 3.0, y - 3.0, w + 6.0, h + 6.0, INK); a.gfx.panel(x, y, w, h, c("p1"));
    a.gfx.text_sh(&body.0, x + 14.0, y + 14.0, 20.0, WHITE, 1);
    let k = a.gfx.scale; let clip_h = h - 110.0;
    unsafe { get_internal_gl().quad_gl.scissor(Some(((x * k) as i32, ((y + 46.0) * k) as i32, (w * k) as i32, (clip_h * k) as i32))); }
    let max_s = (th - clip_h + 10.0).max(0.0); let (_, wy) = mouse_wheel(); a.sheet_scroll = (a.sheet_scroll - wy.signum() * 40.0).clamp(0.0, max_s); if a.down && a.dragging { a.sheet_scroll = (a.press_scroll - a.drag).clamp(0.0, max_s) }
    a.gfx.wrap(&body.1, x + 14.0, y + 50.0 - a.sheet_scroll, w - 28.0, 14.0, c("txt"), 0);
    unsafe { get_internal_gl().quad_gl.scissor(None); }
    let by = y + h - 56.0; let bw = (w - 34.0) / 2.0;
    match sh {
        Sheet::Confirm(_, _, ok, act) => { if a.btn(x + 12.0, by, bw, 44.0, "Cancel", "ghost", true) { a.sheet = Sheet::None } if a.btn(x + 22.0 + bw, by, bw, 44.0, &ok, if act == "chalx" { "danger" } else { "on" }, true) { a.sheet = Sheet::None; confirm(a, &act) } }
        Sheet::Prestige(k) => { if a.btn(x + 12.0, by, bw, 44.0, "Cancel", "ghost", true) { a.sheet = Sheet::None } if a.btn(x + 22.0 + bw, by, bw, 44.0, "Do it", "on", true) { a.sheet = Sheet::None; do_prestige(a, &k) } }
        Sheet::Reset => { if a.btn(x + 12.0, by, bw, 44.0, "Cancel", "ghost", true) { a.sheet = Sheet::None } if a.btn(x + 22.0 + bw, by, bw, 44.0, "Erase", "danger", true) { a.sheet = Sheet::None; let set = a.g.s.set.clone(); a.g = Game::new(crate::game::S::default()); a.g.s.set = set; a.save(); a.toast("Started over", "check", c("green")) } }
        Sheet::Import => {
            let iy = by - 44.0; let shown: String = a.input.chars().take(40).collect(); a.gfx.rect(x + 12.0, iy, w - 24.0, 36.0, INK); a.gfx.text(&if shown.is_empty() { "Paste here (Ctrl+V)".to_string() } else { format!("{shown}…") }, x + 20.0, iy + 11.0, 13.0, if shown.is_empty() { c("faint") } else { WHITE }, 2);
            if a.btn(x + 12.0, by, bw, 44.0, "Cancel", "ghost", true) { a.sheet = Sheet::None; a.input_key.clear() }
            if a.btn(x + 22.0 + bw, by, bw, 44.0, "Load", "on", !a.input.is_empty()) { match crate::save::import(&a.input) { Ok(mut s) => { s.sec.insert("import".into(), serde_json::json!(1)); s.last = now_ms(); a.g = Game::new(s); a.save(); a.sheet = Sheet::None; a.input_key.clear(); a.toast("Save loaded", "check", c("green")) } Err(e) => a.toast(format!("That code didn't load: {e}"), "cross", c("red")) } }
        }
        Sheet::Terms => { if a.btn(x + 12.0, by, w - 24.0, 44.0, "I agree", "on", true) { a.g.s.set.tos = 1.0; a.sheet = Sheet::None } }
        _ => { if a.btn(x + 12.0, by, w - 24.0, 44.0, "Close", "on", true) { a.sheet = Sheet::None; a.sheet_scroll = 0.0 } }
    }
    a.click = false;
}
fn confirm(a: &mut App, act: &str) {
    if act == "chalx" { a.g.exit_chal() }
    else if act == "omega" { if a.g.buy_uu("omega") { a.flash = Some(crate::ui::Flash { title: "The Omega Engine".into(), sub: "You beat Ten Cells!".into(), col: c("gold"), t: 0.0 }); sfx(a, "prestige") } }
    else if let Some(r) = act.strip_prefix("chal:") { let (id, l) = r.split_once(':').unwrap(); if a.g.start_chal(id, l.parse().unwrap_or(1)) { a.tab = "engine".into() } }
}
#[allow(dead_code)] fn strip_html(s: &str) -> String { let mut o = String::new(); let mut tag = false; for ch in s.replace("</p>", "\n\n").replace("<br>", "\n").replace("<li>", "• ").replace("</li>", "\n").chars() { if ch == '<' { tag = true } else if ch == '>' { tag = false } else if !tag { o.push(ch) } } o.trim().to_string() }
