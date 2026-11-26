// The whole game state and its rules: a line-by-line port of the web version's logic (logic.js plus the later patches),
// so balance, prestige layers, events, bosses, worlds, pets, story and achievements behave the same.
use crate::big::{b, bs, Big};
use crate::data::{data, NG};
use crate::fmt::{fmt, fmt_t};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashMap};

type M<T> = BTreeMap<String, T>;
const PACE: f64 = 0.5;
const EVT_CAP: f64 = 3.0;
const MILES: [f64; 7] = [25.0, 50.0, 100.0, 150.0, 200.0, 250.0, 300.0];
const DIM_PROD: [(&str, f64); 4] = [("fractal", 2.0), ("mirror", 2.5), ("dream", 3.0), ("origin", 4.0)];
pub fn limit() -> Big { bs("1.79e308") }

// ---------------- tiny RNG and clock ----------------
static mut SEED: u64 = 0x9E3779B97F4A7C15;
pub fn rnd() -> f64 { unsafe { SEED ^= SEED << 13; SEED ^= SEED >> 7; SEED ^= SEED << 17; (SEED >> 11) as f64 / (1u64 << 53) as f64 } }
pub fn seed_rng(s: u64) { unsafe { SEED = s | 1 } }
pub fn now_ms() -> f64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as f64).unwrap_or(0.0) }
/// JavaScript's Date#toDateString in local time ("Thu Nov 26 2026"), so web saves and this build agree on days
pub fn today() -> String { chrono::Local::now().format("%a %b %d %Y").to_string() }
pub fn local_hour() -> u32 { use chrono::Timelike; chrono::Local::now().hour() }

// ---------------- the save: same field names as the web version ----------------
fn d0() -> Big { Big::ZERO } fn d1() -> Big { Big::ONE } fn d10() -> Big { b(10.0) }
#[derive(Serialize, Deserialize, Clone, Default)] #[serde(default)] pub struct DG { pub b: f64, pub a: Big }
#[derive(Serialize, Deserialize, Clone)] #[serde(default)] pub struct DimS { pub u: f64, pub r: Big, pub g: Vec<DG>, pub amp: f64, pub reso: f64, pub link: f64 }
impl Default for DimS { fn default() -> Self { DimS { u: 0.0, r: Big::ZERO, g: vec![DG::default(), DG::default(), DG::default()], amp: 0.0, reso: 0.0, link: 0.0 } } }
#[derive(Serialize, Deserialize, Clone)] #[serde(default)] pub struct Qr { pub m: f64, pub t: f64, pub lock: f64, pub cd: f64 }
impl Default for Qr { fn default() -> Self { Qr { m: 1.0, t: 0.0, lock: 0.0, cd: 0.0 } } }
#[derive(Serialize, Deserialize, Clone)] pub struct Fight { pub i: usize, pub t: f64, pub dmg: Big }
#[derive(Serialize, Deserialize, Clone, Default)] #[serde(default)] pub struct BossS { pub k: M<f64>, pub cd: M<f64>, pub f: Option<Fight> }
#[derive(Serialize, Deserialize, Clone)] pub struct ChalA { pub id: String, pub l: u32 }
#[derive(Serialize, Deserialize, Clone, Default)] #[serde(default)] pub struct ChalS { pub done: M<f64>, pub a: Option<ChalA> }
#[derive(Serialize, Deserialize, Clone)] pub struct Cur { pub k: String, pub t: f64 }
#[derive(Serialize, Deserialize, Clone)] pub struct Buff { pub k: String, pub m: f64, pub t: f64, #[serde(rename = "T")] pub tt: f64, #[serde(default)] pub src: String }
#[derive(Serialize, Deserialize, Clone)] #[serde(default)] pub struct Ev { pub next: f64, pub cur: Option<Cur>, pub buffs: Vec<Buff> }
impl Default for Ev { fn default() -> Self { Ev { next: 90.0, cur: None, buffs: vec![] } } }
#[derive(Serialize, Deserialize, Clone, Default)] #[serde(default)] pub struct Cnt { pub reb: f64, pub sing: f64, pub asc: f64, pub tra: f64, pub inf: f64, pub ult: f64 }
#[derive(Serialize, Deserialize, Clone)] #[serde(default, rename_all = "camelCase")]
pub struct St { pub taps: f64, pub max_eps: Big, pub upg: f64, pub ev: f64, #[serde(rename = "evT")] pub ev_t: M<f64>, pub ab: f64, pub kills: f64, pub crits: f64, pub play: f64, pub run: f64, pub best_reb: f64, pub last_buy: f64, pub glitch: f64, pub chal: f64, pub act: f64, pub day_n: f64, pub last_day: String, pub beat: f64, #[serde(flatten)] pub extra: Map<String, Value> }
impl Default for St { fn default() -> Self { St { taps: 0.0, max_eps: Big::ZERO, upg: 0.0, ev: 0.0, ev_t: M::new(), ab: 0.0, kills: 0.0, crits: 0.0, play: 0.0, run: 0.0, best_reb: 1e12, last_buy: 0.0, glitch: 0.0, chal: 0.0, act: 0.0, day_n: 1.0, last_day: String::new(), beat: 0.0, extra: Map::new() } } }
#[derive(Serialize, Deserialize, Clone)] pub struct Egg { pub r: usize, pub t: f64 }
#[derive(Serialize, Deserialize, Clone)] pub struct Own { pub lvl: f64 }
#[derive(Serialize, Deserialize, Clone, Default)] #[serde(default)] pub struct Pets { pub eggs: Vec<Egg>, pub own: M<Own>, pub active: Option<String>, pub bought: f64, pub hatched: f64 }
#[derive(Serialize, Deserialize, Clone)] #[serde(default)] pub struct Story { pub ch: f64, pub seen: M<f64>, pub claimed: f64, pub v: f64, #[serde(flatten)] pub extra: Map<String, Value> }
impl Default for Story { fn default() -> Self { Story { ch: 0.0, seen: M::new(), claimed: 0.0, v: 3.0, extra: Map::new() } } }
#[derive(Serialize, Deserialize, Clone)] #[serde(default)] pub struct Daily { pub day: f64, pub last: String, pub n: f64, pub bm: f64, pub bt: f64 }
impl Default for Daily { fn default() -> Self { Daily { day: 0.0, last: String::new(), n: 0.0, bm: 1.0, bt: 0.0 } } }
#[derive(Serialize, Deserialize, Clone)] #[serde(default, rename_all = "camelCase")]
pub struct Set { pub not: String, pub hap: f64, pub skin: String, pub fx: f64, pub sfx: f64, pub mus: f64, pub sv: f64, pub mv: f64, pub tos: f64, pub badge: Value, pub seen_v: String, pub file_sync: f64, pub dsk_v: String, pub uiz: Value, pub fps: f64, pub px: f64, pub perm_bar: f64, pub auto_seen: Option<M<f64>>, #[serde(flatten)] pub extra: Map<String, Value> }
impl Default for Set { fn default() -> Self { Set { not: "suffix".into(), hap: 1.0, skin: "default".into(), fx: 1.0, sfx: 1.0, mus: 1.0, sv: 0.7, mv: 0.5, tos: 0.0, badge: Value::Null, seen_v: String::new(), file_sync: 0.0, dsk_v: String::new(), uiz: json!("auto"), fps: 60.0, px: 3.0, perm_bar: 1.0, auto_seen: None, extra: Map::new() } } }
#[derive(Serialize, Deserialize, Clone)] #[serde(default)]
pub struct Auto { pub tap: f64, pub gens: Vec<f64>, pub upg: f64, pub ab: f64, pub feed: f64, pub reb: f64, #[serde(rename = "rebAt")] pub reb_at: String, pub sing: f64, #[serde(rename = "singAt")] pub sing_at: String, pub asc: f64, #[serde(rename = "ascAt")] pub asc_at: String, pub tra: f64, #[serde(rename = "traAt")] pub tra_at: String, pub inf: f64, #[serde(rename = "infAt")] pub inf_at: String, #[serde(flatten)] pub rules: Map<String, Value> }
impl Default for Auto {
    fn default() -> Self {
        let mut rules = Map::new();
        for (k, t) in [("reb", "10"), ("sing", "20"), ("asc", "30"), ("tra", "45"), ("inf", "60")] { rules.insert(format!("{k}M"), json!("gain")); rules.insert(format!("{k}T"), json!(t)); rules.insert(format!("{k}X"), json!("2")); }
        Auto { tap: 0.0, gens: vec![0.0; NG], upg: 0.0, ab: 0.0, feed: 0.0, reb: 0.0, reb_at: "1".into(), sing: 0.0, sing_at: "1".into(), asc: 0.0, asc_at: "1".into(), tra: 0.0, tra_at: "1".into(), inf: 0.0, inf_at: "1".into(), rules }
    }
}
impl Auto {
    pub fn on(&self, k: &str) -> bool { match k { "tap" => self.tap > 0.0, "upg" => self.upg > 0.0, "ab" => self.ab > 0.0, "reb" => self.reb > 0.0, "sing" => self.sing > 0.0, "asc" => self.asc > 0.0, "tra" => self.tra > 0.0, "inf" => self.inf > 0.0, _ => false } }
    pub fn set_on(&mut self, k: &str, v: bool) { let x = if v { 1.0 } else { 0.0 }; match k { "tap" => self.tap = x, "upg" => self.upg = x, "ab" => self.ab = x, "reb" => self.reb = x, "sing" => self.sing = x, "asc" => self.asc = x, "tra" => self.tra = x, "inf" => self.inf = x, _ => {} } }
    pub fn at(&self, k: &str) -> &str { match k { "reb" => &self.reb_at, "sing" => &self.sing_at, "asc" => &self.asc_at, "tra" => &self.tra_at, _ => &self.inf_at } }
    pub fn at_mut(&mut self, k: &str) -> &mut String { match k { "reb" => &mut self.reb_at, "sing" => &mut self.sing_at, "asc" => &mut self.asc_at, "tra" => &mut self.tra_at, _ => &mut self.inf_at } }
    pub fn rule(&self, k: &str, s: &str) -> String { self.rules.get(&format!("{k}{s}")).map(|v| v.as_str().map(|x| x.to_string()).unwrap_or_else(|| v.to_string())).unwrap_or_default() }
    pub fn set_rule(&mut self, k: &str, s: &str, v: &str) { self.rules.insert(format!("{k}{s}"), json!(v)); }
}

#[derive(Serialize, Deserialize, Clone)] #[serde(default, rename_all = "camelCase")]
pub struct S {
    pub v: f64, pub energy: Big, pub run_e: Big, pub life_e: Big, pub gens: Vec<f64>, pub upg: M<f64>,
    pub frag: Big, pub frag_life: Big, pub perks: M<f64>,
    pub sing: Big, pub sing_total: Big, pub supg: M<f64>, pub bh: Big,
    pub cores: Big, pub cores_total: Big, pub aupg: M<f64>, pub dims: M<DimS>, pub qr: Qr, pub tskip_cd: f64,
    pub ve: Big, pub ve_total: Big, pub vupg: M<f64>, pub laws: Vec<String>, pub law_cd: f64,
    pub ip: Big, pub ip_total: Big, pub inf: M<f64>, pub up: Big, pub up_total: Big, pub uupg: M<f64>,
    pub rp: Big, pub res: M<f64>, pub arts: M<f64>,
    pub boss: BossS, pub chal: ChalS, pub ev: Ev, pub ab: M<f64>, pub auto: Auto, pub cnt: Cnt, pub st: St,
    pub pets: Pets, pub story: Story, pub badge: M<f64>, pub perm: M<f64>, pub daily: Daily, pub ach: M<f64>, pub sec: M<Value>, pub set: Set,
    pub last: f64, pub idle: f64, pub mode: f64, pub started: f64,
    #[serde(flatten)] pub extra: Map<String, Value>,
}
impl Default for S {
    fn default() -> Self {
        let now = now_ms();
        S { v: 1.0, energy: d10(), run_e: d0(), life_e: d0(), gens: vec![0.0; NG], upg: M::new(), frag: d0(), frag_life: d0(), perks: M::new(), sing: d0(), sing_total: d0(), supg: M::new(), bh: d0(),
            cores: d0(), cores_total: d0(), aupg: M::new(), dims: data().dims.iter().map(|d| (d.id.clone(), DimS::default())).collect(), qr: Qr::default(), tskip_cd: 0.0,
            ve: d0(), ve_total: d0(), vupg: M::new(), laws: vec![], law_cd: 0.0, ip: d0(), ip_total: d0(), inf: M::new(), up: d0(), up_total: d0(), uupg: M::new(), rp: d0(), res: M::new(), arts: M::new(),
            boss: BossS::default(), chal: ChalS::default(), ev: Ev::default(), ab: M::new(), auto: Auto::default(), cnt: Cnt::default(), st: St::default(), pets: Pets::default(), story: Story::default(), badge: M::new(),
            perm: M::new(), daily: Daily::default(), ach: M::new(), sec: M::new(), set: Set::default(), last: now, idle: 0.0, mode: 1.0, started: now, extra: Map::new() }
    }
}
#[allow(dead_code)] fn _unused() -> Big { d1() }

// ---------------- derived effects (the web version's FX) ----------------
#[derive(Clone)]
pub struct Fx {
    pub gmult: Big, pub gen: Vec<Big>, pub cost: f64, pub upg_cost: f64, pub mile: f64, pub syn: f64, pub tap_flat: f64, pub tap_mult: Big, pub tap_eps: f64, pub crit_c: f64, pub crit_m: f64,
    pub autotap: f64, pub ag: Vec<bool>, pub auto_int: f64, pub auto_upg: f64, pub auto_max: f64, pub auto_ab: f64, pub ab: Vec<String>, pub ab_cd: f64, pub ab_dur: f64, pub ab_pow: f64, pub evt_freq: f64, pub evt_dur: f64,
    pub offline_h: f64, pub frag_gain: f64, pub sing_gain: f64, pub asc_gain: f64, pub void_gain: f64, pub ip_gain: f64, pub rp: f64, pub bh_pow: f64, pub bh_passive: f64, pub dim_mult: Big, pub speed: f64,
    pub boss_dmg: Big, pub art_drop: f64, pub ach_mult: f64, pub ult_gain: f64, pub growth_red: f64, pub law_slots: f64, pub law_pow: f64, pub expo: f64, pub dyn_: Vec<String>, pub flag: std::collections::HashSet<String>,
}
impl Fx {
    fn new() -> Fx {
        Fx { gmult: b(1.0), gen: vec![b(1.0); NG], cost: 1.0, upg_cost: 1.0, mile: 0.0, syn: 0.0, tap_flat: 0.0, tap_mult: b(1.0), tap_eps: 0.0, crit_c: 0.02, crit_m: 3.0,
            autotap: 0.0, ag: vec![false; NG], auto_int: 1.0, auto_upg: 0.0, auto_max: 0.0, auto_ab: 0.0, ab: vec![], ab_cd: 1.0, ab_dur: 1.0, ab_pow: 1.0, evt_freq: 1.0, evt_dur: 1.0,
            offline_h: 24.0, frag_gain: 1.0, sing_gain: 1.0, asc_gain: 1.0, void_gain: 1.0, ip_gain: 1.0, rp: 1.0, bh_pow: 1.5, bh_passive: 0.0, dim_mult: b(1.0), speed: 1.0,
            boss_dmg: b(1.0), art_drop: 1.0, ach_mult: 1.0, ult_gain: 1.0, growth_red: 0.0, law_slots: 2.0, law_pow: 1.0, expo: 1.0, dyn_: vec![], flag: Default::default() }
    }
    pub fn f(&self, k: &str) -> bool { self.flag.contains(k) }
    /// apply one effect [key, value] l times (the web version's ap())
    fn ap(&mut self, k: &str, v: &Value, l: f64) {
        let num = v.as_f64().unwrap_or(0.0);
        if let Some(i) = k.strip_prefix("gen").and_then(|s| s.parse::<usize>().ok()) { if i < NG { self.gen[i] = self.gen[i].mul(b(num).pow(l)); } return; }
        if let Some(i) = k.strip_prefix("ag").and_then(|s| s.parse::<usize>().ok()) { if i < NG { self.ag[i] = true; } return; }
        match k {
            "ab" => { let s = v.as_str().unwrap_or("").to_string(); if !self.ab.contains(&s) { self.ab.push(s) } }
            "dyn" => { let s = v.as_str().unwrap_or("").to_string(); if !self.dyn_.contains(&s) { self.dyn_.push(s) } }
            "flag" => { self.flag.insert(v.as_str().unwrap_or("").to_string()); }
            "gmult" => self.gmult = self.gmult.mul(b(num).pow(l)), "tapMult" => self.tap_mult = self.tap_mult.mul(b(num).pow(l)),
            "dimMult" => self.dim_mult = self.dim_mult.mul(b(num).pow(l)), "bossDmg" => self.boss_dmg = self.boss_dmg.mul(b(num).pow(l)),
            // multiplicative keys
            "cost" => self.cost *= num.powf(l), "upgCost" => self.upg_cost *= num.powf(l), "evtFreq" => self.evt_freq *= num.powf(l), "evtDur" => self.evt_dur *= num.powf(l),
            "abCd" => self.ab_cd *= num.powf(l), "abDur" => self.ab_dur *= num.powf(l), "abPow" => self.ab_pow *= num.powf(l), "fragGain" => self.frag_gain *= num.powf(l),
            "singGain" => self.sing_gain *= num.powf(l), "ascGain" => self.asc_gain *= num.powf(l), "voidGain" => self.void_gain *= num.powf(l), "ipGain" => self.ip_gain *= num.powf(l),
            "rp" => self.rp *= num.powf(l), "speed" => self.speed *= num.powf(l), "artDrop" => self.art_drop *= num.powf(l), "achMult" => self.ach_mult *= num.powf(l),
            "autoInt" => self.auto_int *= num.powf(l), "expo" => self.expo *= num.powf(l), "ultGain" => self.ult_gain *= num.powf(l),
            // additive keys
            "mile" => self.mile += num * l, "syn" => self.syn += num * l, "tapFlat" => self.tap_flat += num * l, "tapEps" => self.tap_eps += num * l, "critC" => self.crit_c += num * l, "critM" => self.crit_m += num * l,
            "autotap" => self.autotap += num * l, "autoUpg" => self.auto_upg += num * l, "autoMax" => self.auto_max += num * l, "autoAb" => self.auto_ab += num * l, "offlineH" => self.offline_h += num * l,
            "bhPow" => self.bh_pow += num * l, "bhPassive" => self.bh_passive += num * l, "growthRed" => self.growth_red += num * l, "lawSlots" => self.law_slots += num * l, "lawPow" => self.law_pow += num * l,
            _ => {}
        }
    }
}
#[derive(Clone, Default)] pub struct Calc { pub g: Big, pub eps: Big, pub prod: Vec<Big>, pub tap: Big, pub tap_e: Big, pub sp: f64, pub mp: f64 }
#[derive(Clone)] pub struct Note { pub kind: String, pub text: String, pub extra: String, pub auto: bool }

pub struct Game { pub s: S, pub fx: Fx, pub c: Calc, fx_dirty: bool, pub ach_n: usize, pub hid_n: usize, pub heat: f64, pub q: Vec<Note>, auto_t: f64, ach_t: f64, pub session: f64, pub toasts: Vec<(String, String, String)> }

fn lv(m: &M<f64>, id: &str) -> f64 { *m.get(id).unwrap_or(&0.0) }
fn has(m: &M<f64>, id: &str) -> bool { m.get(id).map(|v| *v != 0.0).unwrap_or(false) }
pub fn oom(x: Big) -> f64 { x.addf(1.0).log10() }

impl Game {
    pub fn new(s: S) -> Game { let mut g = Game { s, fx: Fx::new(), c: Calc::default(), fx_dirty: true, ach_n: 0, hid_n: 0, heat: 0.0, q: vec![], auto_t: 0.0, ach_t: 0.0, session: 0.0, toasts: vec![] }; g.fx_check(); g.calc(); g.count_ach(); g }
    pub fn notify(&mut self, kind: &str, text: String, extra: String) { self.q.push(Note { kind: kind.into(), text, extra, auto: false }) }
    pub fn toast(&mut self, text: String, spr: &str, col: &str) { self.toasts.push((text, spr.into(), col.into())) }
    pub fn dirty(&mut self) { self.fx_dirty = true }
    pub fn fx_check(&mut self) { if self.fx_dirty { self.fx = self.fx_build(); self.fx_dirty = false; } }
    pub fn free_auto(&self) -> bool { has(&self.s.aupg, "auto") || has(&self.s.inf, "a1") }
    // Muscle Memory (Ultra) also counts: the web version forgot to check its flag
    pub fn has_u(&self, id: &str) -> bool { has(&self.s.upg, id) || ((self.free_auto() || self.fx.f("freeAuto")) && data().upg.iter().any(|u| u.id == id && u.cat == "auto")) }
    fn fx_build(&self) -> Fx {
        let d = data(); let s = &self.s; let mut f = Fx::new();
        let add = |f: &mut Fx, e: &Vec<(String, Value)>, l: f64| for (k, v) in e { f.ap(k, v, l) };
        for u in &d.aupg { if has(&s.aupg, &u.id) { add(&mut f, &u.e, 1.0) } }
        for u in &d.inf { if has(&s.inf, &u.id) { add(&mut f, &u.e, 1.0) } }
        for u in &d.uupg { if has(&s.uupg, &u.id) { add(&mut f, &u.e, 1.0) } }
        for u in &d.supg { if has(&s.supg, &u.id) { add(&mut f, &u.e, 1.0) } }
        for u in &d.vupg { if has(&s.vupg, &u.id) { add(&mut f, &u.e, 1.0) } }
        for r in &d.res { if has(&s.res, &r.id) { add(&mut f, &r.e, 1.0) } }
        for a in &d.arts { let l = lv(&s.arts, &a.id); if l > 0.0 { add(&mut f, &a.e, l) } }
        for c in &d.chal { let l = lv(&s.chal.done, &c.id); if l > 0.0 { add(&mut f, &chal_eff(&c.id, l), 1.0) } }
        let free = has(&s.aupg, "auto") || has(&s.inf, "a1") || f.f("freeAuto");
        for u in &d.upg { if has(&s.upg, &u.id) || (u.cat == "auto" && free) { add(&mut f, &u.e, 1.0) } }
        for p in &d.pets { if let Some(o) = s.pets.own.get(&p.id) { let l = if s.pets.active.as_deref() == Some(&p.id) { o.lvl } else { o.lvl * 0.25 }; f.ap(&p.b.0, &json!(p.b.1), l) } }
        if s.story.claimed > 0.0 { f.gmult = f.gmult.mul(b(d.story_m).pow(s.story.claimed)) }
        let p = &s.perks;
        f.tap_mult = f.tap_mult.mul(b(3f64.powf(lv(p, "tapM"))));
        f.offline_h += 4.0 * lv(p, "offl"); f.cost *= 0.9f64.powf(lv(p, "cheap")); f.rp *= 1.5f64.powf(lv(p, "rsch"));
        f.frag_gain *= 1.25f64.powf(lv(p, "fgain")); f.mile += 0.05 * lv(p, "mile"); f.boss_dmg = f.boss_dmg.mul(b(2f64.powf(lv(p, "boss"))));
        f.evt_freq *= 1.1f64.powf(lv(p, "evt")); f.crit_c += 0.02 * lv(p, "critP"); f.ab_cd *= 0.9f64.powf(lv(p, "abCd"));
        // 0.20: permanent upgrades (non-production effects here; production is its own line) and the Rebirth Autopilot perk
        for u in &d.perm { if has(&s.perm, &u.id) { for (k, v) in &u.e { if k != "prod" { f.ap(k, v, 1.0) } } } }
        if lv(p, "autoR") > 0.0 { f.flag.insert("autoReb".into()); }
        f
    }
    // ---------------- helpers ----------------
    pub fn in_c(&self, id: &str) -> bool { self.s.chal.a.as_ref().map(|a| a.id == id).unwrap_or(false) }
    pub fn law(&self, id: &str) -> bool { self.s.laws.iter().any(|x| x == id) }
    fn law_b(&self, v: f64) -> f64 { v.powf(self.fx.law_pow) }
    fn law_d(&self, v: f64) -> f64 { if self.fx.f("paradox") { v.sqrt() } else { v } }
    pub fn buff_m(&self, k: &str) -> f64 { self.s.ev.buffs.iter().filter(|x| x.k == k).fold(1.0, |m, x| m * x.m) }
    pub fn reso(&self, id: &str) -> f64 { 1.25f64.powf(self.s.dims.get(id).map(|d| d.reso).unwrap_or(0.0)) }
    pub fn dim_on(&self, id: &str) -> bool { self.s.dims.get(id).map(|d| d.u != 0.0).unwrap_or(false) }
    fn dim_r(&self, id: &str) -> Big { self.s.dims.get(id).map(|d| d.r).unwrap_or(Big::ZERO) }
    pub fn dim_x(&self, id: &str, k: f64) -> f64 { if self.dim_on(id) { 1.0 + oom(self.dim_r(id)) * self.reso(id) / k } else { 1.0 } }
    pub fn off_h(&self) -> f64 { self.fx.offline_h + if self.dim_on("dream") { 24f64.min((oom(self.dim_r("dream")) * self.reso("dream")).floor()) } else { 0.0 } }
    pub fn dims_unlocked(&self) -> usize { data().dims.iter().filter(|d| self.dim_on(&d.id)).count() }
    pub fn bh_on(&self) -> bool { self.s.cnt.sing > 0.0 }
    fn dyn_v(&self, k: &str) -> Big {
        let s = &self.s;
        match k { "logE" => b(1.0 + oom(s.run_e) / 10.0), "gens" => b(1.0 + s.gens.iter().sum::<f64>() / 500.0), "ach" => b(1.02).pow(self.ach_n as f64), "time" => b((1.0 + s.st.run / 1800.0).min(5.0)),
            "taps" => b(1.0 + oom(b(s.st.taps)) / 2.0), "ip" => s.ip.addf(1.0).pow(2.0), "hidden" => b(1.5).pow(self.hid_n as f64), _ => b(1.0) }
    }
    pub fn mile_count(&self, n: f64) -> f64 {
        if self.fx.f("mobius") { return (n / 25.0).floor() }
        let mut c = MILES.iter().filter(|t| n >= **t).count() as f64;
        if n >= 400.0 { c += ((n - 300.0) / 100.0).floor() }
        c
    }
    pub fn next_mile(&self, n: f64) -> f64 {
        if self.fx.f("mobius") { return ((n / 25.0).floor() + 1.0) * 25.0 }
        for t in MILES { if n < t { return t } }
        ((n / 100.0).floor() + 1.0) * 100.0
    }
    fn stellar_eff(&self) -> f64 { if self.dim_on("stellar") { 0.25 * (1.0 + oom(self.dim_r("stellar"))).log10() * self.reso("stellar") } else { 0.0 } }
    pub fn mile_pow(&self) -> f64 { 2.0 + self.fx.mile + self.stellar_eff() }
    pub fn bh_pow(&self) -> f64 { let mut p = self.fx.bh_pow; if self.dim_on("void") { p += 0.01 * oom(self.dim_r("void")) * self.reso("void") } if self.law("collapse") { p += 2.0 * self.fx.law_pow } p }
    fn speed(&self) -> f64 { let mut s = self.fx.speed * self.buff_m("speed"); if self.law("haste") { s *= self.law_b(2.0) } if self.in_c("slow") { s *= 0.25 } s }
    pub fn boss_dmg(&self) -> Big { self.fx.boss_dmg }
    pub fn badge_n(&self) -> f64 { self.s.badge.values().sum() }
    // daily reward boost
    pub fn boost_on(&self) -> bool { self.s.daily.bt > now_ms() && self.s.daily.bm > 1.0 }
    fn boost(&mut self, m: f64, hours: f64) { let on = self.boost_on(); let d = &mut self.s.daily; if on && d.bm >= m { d.bt = d.bt.max(now_ms() + hours * 3600e3) } else { d.bm = m; d.bt = now_ms() + hours * 3600e3 } }
    pub fn perm_prod(&self) -> f64 { data().perm.iter().filter(|u| has(&self.s.perm, &u.id)).fold(1.0, |m, u| u.e.iter().fold(m, |a, (k, v)| if k == "prod" { a * v.as_f64().unwrap_or(1.0) } else { a })) }

    // ---------------- production ----------------
    pub fn mult_parts(&self) -> Vec<(String, Big)> {
        let s = &self.s; let d = data(); let mut p: Vec<(String, Big)> = vec![("Upgrades & bonuses".into(), self.fx.gmult)];
        if self.badge_n() > 0.0 { p.push(("Badges".into(), b(1.0 + 0.02 * self.badge_n()))) }
        for k in &self.fx.dyn_ { let n = match k.as_str() { "logE" => "Echo Chamber", "gens" => "Fleet Harmony", "ach" => "Hall of Records", "time" => "Momentum", "taps" => "Kinetic Memory", "ip" => "Infinite Fuel", _ => "Glitched Fragment" }; p.push((n.into(), self.dyn_v(k))) }
        if !self.in_c("fragile") { p.push(("Fragments".into(), s.frag.mulf(0.05 + 0.02 * lv(&s.perks, "reso")).addf(1.0))) }
        p.push(("Singularities".into(), s.sing_total.addf(1.0).pow(1.5)));
        if self.bh_on() { p.push(("Black Hole".into(), b(1.0 + oom(s.bh)).pow(self.bh_pow()))) }
        if s.up_total.gtf(0.0) { p.push(("Ultra Points".into(), s.up_total.addf(1.0).pow(4.0))) }
        p.push(("Ascension Cores".into(), s.cores_total.addf(1.0).pow(1.5))); p.push(("Void Essence".into(), s.ve_total.addf(1.0).pow(2.0))); p.push(("Infinity Points".into(), s.ip_total.addf(1.0).pow(2.0)));
        p.push(("Achievements".into(), b(1.0 + 0.02 * self.ach_n as f64 * self.fx.ach_mult))); p.push(("Boss trophies".into(), b(3.0).pow(s.boss.k.len() as f64)));
        if self.dim_on("material") { p.push(("Matter".into(), self.dim_r("material").addf(1.0).pow(0.05 * self.reso("material")))) }
        for (id, k) in DIM_PROD { if self.dim_on(id) { let r = &d.dims[crate::data::dim_idx(id)].res; p.push((r.clone(), b(1.0 + oom(self.dim_r(id)) * self.reso(id)).pow(k))) } }
        p.push(("Active buffs".into(), b(self.buff_m("prod"))));
        let mut lw = 1.0; if self.law("entropy") { lw *= self.law_b(1e4) } if self.law("fortune") { lw *= self.law_d(0.1) } if self.law("expansion") { lw *= self.law_d(0.01) }
        p.push(("Reality Laws".into(), b(lw)));
        if self.in_c("blackout") { p.push(("Blackout".into(), b(0.01))) }
        if self.in_c("entropy") { p.push(("Entropy".into(), b(0.5f64.powf((s.st.run - s.st.last_buy) / 60.0)))) }
        let pm = self.perm_prod(); if pm > 1.0 { p.push(("Permanent upgrades".into(), b(pm))) }
        if self.boost_on() { p.push((format!("Daily reward ({} left)", fmt_t(((s.daily.bt - now_ms()) / 1000.0).ceil())), b(s.daily.bm))) }
        p
    }
    pub fn calc(&mut self) {
        let parts = self.mult_parts(); let g = parts.iter().fold(b(1.0), |a, p| a.mul(p.1)); let mp = self.mile_pow(); let o = &self.s.gens; let d = data();
        let top = (0..NG).rev().find(|&i| o[i] > 0.0);
        let mut prod = vec![Big::ZERO; NG]; let mut eps = Big::ZERO;
        for i in 0..NG {
            let n = o[i] + if self.fx.f("hoi") && i < NG - 1 { o[i + 1] * 0.1 } else { 0.0 };
            let mut p = Big::ZERO;
            if n > 0.0 {
                p = d.gens[i].p.mulf(n).mul(self.fx.gen[i]).mul(g);
                if !self.in_c("nomile") { p = p.mul(b(mp).pow(self.mile_count(o[i]))) }
                if i < NG - 1 && self.fx.syn != 0.0 { p = p.mulf(1.0 + self.fx.syn * o[i + 1]) }
                if self.law("gravity") { p = p.mulf(if i < 5 { self.law_b(1e6) } else { self.law_d(0.01) }) }
                if self.law("inversion") { if i == 9 { p = p.mulf(self.law_b(1e10)) } if i < 3 { p = Big::ZERO } }
                if self.in_c("apex") && Some(i) != top { p = Big::ZERO }
            }
            prod[i] = p; eps = eps.add(p);
        }
        if self.fx.expo > 1.0 && eps.gtf(1.0) { let e2 = eps.pow(self.fx.expo); let k = e2.div(eps); for x in prod.iter_mut() { *x = x.mul(k) } eps = e2; }
        let mut tap = b(1.0 + self.fx.tap_flat).mul(self.fx.tap_mult).mulf(self.buff_m("tap"));
        if self.dim_on("plasma") { tap = tap.mul(self.dim_r("plasma").addf(1.0).pow(0.1 * self.reso("plasma"))) }
        if self.law("echoes") { tap = tap.mulf(self.law_b(1e6)) }
        if self.in_c("blackout") { tap = tap.mulf(100.0) }
        let tap_e = eps.mulf(self.fx.tap_eps).mulf(self.buff_m("tap"));
        self.c = Calc { g, eps, prod, tap, tap_e, sp: self.speed(), mp };
    }
    pub fn gain(&mut self, v: Big) {
        if !v.gtf(0.0) { return }
        let s = &mut self.s; s.energy = s.energy.add(v); s.run_e = s.run_e.add(v); s.life_e = s.life_e.add(v);
        if !self.fx.f("broken") { if s.energy.gt(limit()) { s.energy = limit() } if s.run_e.gt(limit()) { s.run_e = limit() } }
    }
    pub fn capped(&self) -> bool { !self.fx.f("broken") && self.s.run_e.gte(limit()) }

    // ---------------- energy makers ----------------
    pub fn gen_allowed(&self, i: usize) -> bool { !(self.in_c("lone") && i > 1) }
    pub fn gen_visible(&self, i: usize) -> bool { i == 0 || self.s.gens[i] > 0.0 || self.s.gens[i - 1] > 0.0 }
    pub fn gen_base(&self, i: usize) -> Big { let mut c = data().gens[i].c.mulf(self.fx.cost * self.buff_m("cost")); if self.dim_on("quantum") { c = c.div(self.dim_r("quantum").addf(1.0).pow(0.02 * self.reso("quantum"))) } c }
    pub fn gen_ratio(&self, i: usize) -> f64 { let mut r = data().gens[i].r - self.fx.growth_red; if self.in_c("inflation") { r += 0.1 } r.max(1.03) }
    pub fn gen_cost_n(&self, i: usize, n: f64) -> Big { Big::sum_geo(n, self.gen_base(i), self.gen_ratio(i), self.s.gens[i]) }
    pub fn gen_max_n(&self, i: usize) -> f64 { Big::afford_geo(self.s.energy, self.gen_base(i), self.gen_ratio(i), self.s.gens[i]).min(1e6) }
    /// mode: 0 = max, else that many
    pub fn buy_gen(&mut self, i: usize, mode: f64) -> f64 {
        if !self.gen_allowed(i) || !self.gen_visible(i) { return 0.0 }
        let n = if mode == 0.0 { self.gen_max_n(i) } else { mode };
        if n < 1.0 { return 0.0 }
        let c = self.gen_cost_n(i, n); if c.gt(self.s.energy) { return 0.0 }
        self.s.energy = self.s.energy.sub(c); self.s.gens[i] += n; self.s.st.last_buy = self.s.st.run; n
    }
    pub fn buy_everything(&mut self) -> f64 { let mut n = 0.0; for i in (0..NG).rev() { n += self.buy_gen(i, 0.0) } if n > 0.0 { self.calc() } n }

    // ---------------- upgrades ----------------
    pub fn cond_ok(&self, c: &Option<crate::data::Cond>) -> bool { match c { None => true, Some(c) => match c.t.as_str() { "gen" => self.s.gens[c.i] >= c.n, "reb" => self.s.cnt.reb >= c.n, "taps" => self.s.st.taps >= c.n, _ => false } } }
    pub fn cond_text(c: &crate::data::Cond) -> String { match c.t.as_str() { "gen" => format!("Own {} {}", c.n, data().gens[c.i].n), "reb" => format!("Rebirth {} time{}", c.n, if c.n > 1.0 { "s" } else { "" }), _ => format!("Tap the reactor {} times", c.n) } }
    pub fn upg_cost(&self, u: &crate::data::Upg) -> Big { let mut c = u.cost.mulf(self.fx.upg_cost); if self.law("entropy") { c = c.mulf(self.law_d(1000.0)) } c }
    pub fn upg_avail(&self, u: &crate::data::Upg) -> bool { !self.has_u(&u.id) && u.req.iter().all(|r| self.has_u(r)) && self.cond_ok(&u.cond) }
    pub fn buy_upg(&mut self, id: &str) -> bool {
        let u = match data().upg.iter().find(|u| u.id == id) { Some(u) => u, None => return false };
        if self.in_c("noupg") || !self.upg_avail(u) { return false }
        let c = self.upg_cost(u); if c.gt(self.s.energy) { return false }
        self.s.energy = self.s.energy.sub(c); self.s.upg.insert(id.into(), 1.0); self.s.st.upg += 1.0; self.dirty(); true
    }
    pub fn buy_all_upg(&mut self) -> usize {
        let mut n = 0;
        for _ in 0..200 { self.fx_check(); let mut av: Vec<_> = data().upg.iter().filter(|u| self.upg_avail(u)).collect(); av.sort_by(|a, b| self.upg_cost(a).cmp(&self.upg_cost(b))); match av.first() { Some(u) if self.buy_upg(&u.id.clone()) => n += 1, _ => break } }
        n
    }

    // ---------------- tapping and abilities ----------------
    pub fn do_tap(&mut self) -> Option<(Big, bool)> {
        if self.in_c("notap") || self.law("haste") { return None }
        self.heat = (self.heat + 0.04).min(1.0);
        let mut v = self.c.tap.add(self.c.tap_e).mulf(1.0 + self.heat); let crit = rnd() < self.fx.crit_c.min(1.0);
        if crit { v = v.mulf(self.fx.crit_m); self.s.st.crits += 1.0 }
        self.gain(v); self.s.st.taps += 1.0; self.s.idle = 0.0;
        let bd = self.boss_dmg(); if let Some(f) = self.s.boss.f.as_mut() { f.dmg = f.dmg.add(v.mulf(10.0).mul(bd)) }
        Some((v, crit))
    }
    pub fn ab_ready(&self, id: &str) -> bool { self.fx.ab.iter().any(|x| x == id) && !(lv(&self.s.ab, id) > 0.0) }
    pub fn use_ab(&mut self, id: &str) -> bool {
        if !self.ab_ready(id) { return false }
        let cd = data().abil.iter().find(|a| a.0 == id).map(|a| a.2).unwrap_or(60.0);
        self.s.ab.insert(id.into(), cd * self.fx.ab_cd); self.s.st.ab += 1.0;
        let (p, du) = (self.fx.ab_pow, self.fx.ab_dur);
        match id {
            "over" => self.buff("prod", 10.0 * p, 30.0 * du, "over"), "gold" => self.buff("tap", 50.0 * p, 20.0 * du, "gold"),
            "chrono" => { let v = self.c.eps.mulf(900.0 * p * self.c.sp); self.gain(v); self.notify("gain", format!("Chrono Burst: +{} energy", fmt(v)), String::new()) }
            "lure" => self.spawn_event(), _ => {}
        }
        true
    }
    fn buff(&mut self, k: &str, m: f64, t: f64, src: &str) { self.s.ev.buffs.push(Buff { k: k.into(), m, t, tt: t, src: src.into() }) }

    // ---------------- events ----------------
    fn evt_freq(&self) -> f64 { self.fx.evt_freq.min(EVT_CAP) * if self.law("fortune") { self.law_b(3.0) } else { 1.0 } }
    pub fn spawn_event(&mut self) {
        self.s.ev.next = 150.0 + rnd() * 210.0;
        let ev = &data().events; let w: Vec<f64> = ev.iter().map(|e| e.w * if e.k == "glitch" && self.fx.f("glitch") { 3.0 } else { 1.0 }).collect();
        let mut r = rnd() * w.iter().sum::<f64>(); let mut k = ev[0].k.clone();
        for (i, e) in ev.iter().enumerate() { r -= w[i]; if r <= 0.0 { k = e.k.clone(); break } }
        self.s.ev.cur = Some(Cur { k, t: 15.0 });
    }
    pub fn claim_event(&mut self) {
        let k = match self.s.ev.cur.take() { Some(c) => c.k, None => return };
        let d = self.fx.evt_dur; self.s.st.ev += 1.0; *self.s.st.ev_t.entry(k.clone()).or_insert(0.0) += 1.0;
        match k.as_str() {
            "storm" => self.buff("prod", 5.0, 30.0 * d, "storm"), "golden" => self.buff("prod", 25.0, 15.0 * d, "golden"), "surge" => self.buff("tap", 20.0, 30.0 * d, "surge"),
            "leak" => self.buff("cost", 0.5, 30.0 * d, "leak"),
            "glitch" => { let m = if self.fx.f("glitch") { 9.0 } else { 3.0 }; self.buff("speed", m, 20.0 * d, "glitch"); self.s.st.glitch += 1.0; if self.s.st.glitch >= 5.0 && !has(&self.s.arts, "glitched") { self.give_art("glitched") } }
            _ => {}
        }
        let e = data().events.iter().find(|e| e.k == k).unwrap(); let mut msg = e.d.clone();
        if k == "collapse" { let v = self.c.eps.mulf((120.0 + rnd() * 480.0) * self.c.sp); self.gain(v); msg = format!("+{} energy", fmt(v)) }
        self.notify("event", format!("{}! {}", e.n, msg), k);
        if rnd() < 0.03 * self.fx.art_drop * if self.law("fortune") { 3.0 } else { 1.0 } { self.roll_art(0) }
        if rnd() < 0.04 { self.give_egg(0, false); }
    }
    fn ev_tick(&mut self, dt: f64) {
        if let Some(c) = self.s.ev.cur.as_mut() { c.t -= dt; let gone = c.t <= 0.0; if self.fx.f("autoEvt") { self.claim_event() } else if gone { self.s.ev.cur = None } }
        else if self.s.st.play > 20.0 { self.s.ev.next -= dt * self.evt_freq(); if self.s.ev.next <= 0.0 { self.spawn_event() } }
    }

    // ---------------- artifacts ----------------
    pub fn roll_art(&mut self, min_r: usize) {
        let d = data(); let pool: Vec<_> = d.arts.iter().filter(|a| !a.secret && a.r >= min_r).collect(); if pool.is_empty() { return }
        let mut r = rnd() * pool.iter().map(|a| d.rar_w[a.r]).sum::<f64>(); let mut pick = pool[0];
        for a in &pool { r -= d.rar_w[a.r]; if r <= 0.0 { pick = a; break } }
        self.give_art(&pick.id.clone())
    }
    pub fn give_art(&mut self, id: &str) {
        let d = data(); let a = match d.arts.iter().find(|a| a.id == id) { Some(a) => a, None => return };
        let max = if a.r == 4 { 1.0 } else { 5.0 }; let cur = lv(&self.s.arts, id);
        if cur >= max { let v = self.c.eps.mulf(600.0); self.gain(v); self.notify("art", format!("{} (duplicate) → +{} energy", a.n, fmt(v)), id.into()); return }
        self.s.arts.insert(id.into(), cur + 1.0); self.dirty();
        self.notify("art", format!("{} artifact: {}{}", d.rar[a.r], a.n, if cur > 0.0 { format!(" → Lv {}", cur + 1.0) } else { String::new() }), id.into());
    }

    // ---------------- bosses ----------------
    pub fn boss_unlocked(&self, i: usize) -> bool { self.s.cnt.reb > 0.0 && (i == 0 || self.s.boss.k.contains_key(&(i - 1).to_string())) && (i < 10 || self.fx.f("broken")) }
    pub fn boss_ready(&self, i: usize) -> bool { self.boss_unlocked(i) && !(lv(&self.s.boss.cd, &i.to_string()) > 0.0) && self.s.boss.f.is_none() }
    pub fn start_boss(&mut self, i: usize) -> bool { if !self.boss_ready(i) { return false } self.s.boss.f = Some(Fight { i, t: 60.0, dmg: Big::ZERO }); true }
    fn boss_tick(&mut self, dt: f64) {
        for v in self.s.boss.cd.values_mut() { *v = (*v - dt).max(0.0) }
        let (eps, bd) = (self.c.eps, self.boss_dmg());
        let f = match self.s.boss.f.as_mut() { Some(f) => f, None => return };
        f.dmg = f.dmg.add(eps.mul(bd).mulf(dt)); f.t -= dt; let (i, dmg, t) = (f.i, f.dmg, f.t);
        let bo = &data().bosses[i];
        if dmg.gte(bo.hp) {
            let key = i.to_string(); let first = !self.s.boss.k.contains_key(&key);
            *self.s.boss.k.entry(key.clone()).or_insert(0.0) += 1.0; self.s.st.kills += 1.0; self.s.boss.cd.insert(key, 600.0); self.s.boss.f = None;
            let v = self.c.eps.mulf(120.0); self.gain(v);
            self.notify("boss", format!("{} defeated! +{} energy{}", bo.n, fmt(v), if first { " · permanent ×3 production" } else { "" }), i.to_string());
            if first { self.roll_art((i / 2).min(4)); self.give_egg((i / 2).min(4), true); } else if rnd() < 0.5 * self.fx.art_drop { self.roll_art(i / 3) }
            if i == 9 { self.s.sec.insert("omega".into(), json!(1)); }
            self.dirty();
        } else if t <= 0.0 { self.s.boss.f = None; self.s.boss.cd.insert(i.to_string(), 60.0); self.notify("bad", format!("{} escaped. Try again in 60s.", bo.n), String::new()) }
    }

    // ---------------- prestige ----------------
    fn start_energy(&self) -> Big { let l = lv(&self.s.perks, "head"); if l > 0.0 { b(100.0).mul(b(5.0).pow(l)) } else { b(10.0) } }
    pub fn reset_layer(&mut self, l: u32) {
        self.fx_check(); let f = self.fx.flag.clone(); let keep_auto = lv(&self.s.perks, "keepAuto") > 0.0; let has_f = |k: &str| f.contains(k);
        if l > 1 || !has_f("ouro") {
            let mut kept = M::new();
            if l == 1 { for id in self.s.upg.keys() { if let Some(u) = data().upg.iter().find(|u| &u.id == id) { if (keep_auto && (u.cat == "auto" || u.cat == "spec")) || (has_f("keepGenUpg") && u.cat == "gen") { kept.insert(id.clone(), 1.0); } } } }
            self.s.upg = kept;
        }
        if l >= 2 {
            self.s.frag = Big::ZERO; self.s.bh = Big::ZERO;
            let keep_p = match l { 2 => has_f("keepPerk") || has_f("aKeepPerk") || has_f("vKeepSing"), 3 => has_f("aKeepPerk") || has_f("vKeepSing"), 4 => has_f("vKeepSing"), _ => false };
            if !keep_p { self.s.perks = M::new() }
        }
        if l >= 3 { let keep = match l { 3 => has_f("aKeepSU") || has_f("vKeepSing"), 4 => has_f("vKeepSing"), _ => false }; if !keep { self.s.supg = M::new() } }
        if l >= 4 {
            if !(l == 4 && has_f("vKeepAsc")) { self.s.aupg = M::new() }
            if !(l == 4 && has_f("dimKeep")) { for d in self.s.dims.values_mut() { let u = d.u; *d = DimS::default(); d.u = u; if u != 0.0 { d.r = b(10.0) } } }
        }
        if l >= 5 { self.s.ve = Big::ZERO; self.s.ve_total = Big::ZERO; self.s.vupg = M::new(); self.s.laws = vec![]; if !has_f("keepDims") { for d in self.s.dims.values_mut() { *d = DimS::default() } } }
        self.dirty(); self.fx_check();
        let f2 = self.fx.flag.clone();
        if l >= 3 { self.s.sing = b(if f2.contains("start100") { 100.0 } else if f2.contains("startSing") { 5.0 } else { 0.0 }); self.s.sing_total = self.s.sing }
        if l >= 4 { self.s.cores = b(if f2.contains("startCores") { 10.0 } else { 0.0 }); self.s.cores_total = self.s.cores }
        self.s.energy = self.start_energy(); self.s.run_e = Big::ZERO; self.s.gens = vec![0.0; NG];
        if f2.contains("seed") { for i in 0..5 { self.s.gens[i] = 10.0 } }
        self.s.ev.buffs.clear(); self.s.st.run = 0.0; self.s.st.last_buy = 0.0; self.s.boss.f = None;
        self.calc();
    }
    pub fn reb_req(&self) -> Big { b(if self.fx.f("rebReq") { 1e9 } else { 1e12 }) }
    fn celestial_eff(&self) -> f64 { if self.dim_on("celestial") { (1.0 + oom(self.dim_r("celestial")) * self.reso("celestial")).powi(2) } else { 1.0 } }
    pub fn layer_gain(&self, id: &str) -> Big {
        let s = &self.s;
        match id {
            "reb" => { if s.run_e.lt(self.reb_req()) { return Big::ZERO } let mut fg = self.fx.frag_gain * self.celestial_eff(); if self.law("collapse") { fg *= self.law_d(0.5) } s.run_e.div(self.reb_req()).pow(0.33).mulf(3.0 * fg * PACE).floor() }
            "sing" => if s.run_e.ltf(1e30) { Big::ZERO } else { b((s.run_e.log10() / 30.0).powi(3) * self.fx.sing_gain * self.celestial_eff() * PACE).floor().maxf(1.0) },
            "asc" => if s.run_e.ltf(1e75) { Big::ZERO } else { b((s.run_e.log10() / 75.0).powi(4) * self.fx.asc_gain * self.dim_x("fractal", 4.0) * PACE).floor().maxf(1.0) },
            "tra" => if s.run_e.ltf(1e125) || self.dims_unlocked() < 4 { Big::ZERO } else { b((s.run_e.log10() / 125.0).powi(5) * self.fx.void_gain * self.dim_x("mirror", 5.0) * PACE).floor().maxf(1.0) },
            "inf" => {
                if s.run_e.lt(limit()) { return Big::ZERO }
                let m = self.fx.ip_gain * if self.fx.f("ipAch") { 1.0 + self.ach_n as f64 / 10.0 } else { 1.0 } * if self.dim_on("infinite") { 1.0 + oom(self.dim_r("infinite")) * self.reso("infinite") / 10.0 } else { 1.0 };
                let base = if self.fx.f("broken") { b(10.0).pow(s.run_e.log10() / 308.0 - 0.75) } else { b(1.0) };
                base.mulf(m * PACE).mul(s.up_total.addf(1.0).pow(0.75)).floor().maxf(1.0)
            }
            "ult" => { if s.ip_total.ltf(data().ult_ip) { return Big::ZERO } s.ip_total.divf(data().ult_ip).pow(1.0).mulf(3.0 * PACE).mulf(self.fx.ult_gain).floor() }
            _ => Big::ZERO,
        }
    }
    pub fn layer_req(&self, id: &str) -> String { match id { "reb" => format!("{} energy this run", fmt(self.reb_req())), "sing" => "1e30 energy this run".into(), "asc" => "1e75 energy this run".into(), "tra" => "1e125 energy this run & 4 dimensions".into(), "inf" => "1.79e308 energy (Infinity)".into(), _ => format!("{} Infinity Points since your last Ultra", fmt(b(data().ult_ip))) } }
    pub fn can_prestige(&self, id: &str) -> bool { self.s.chal.a.is_none() && self.layer_gain(id).gtef(1.0) }
    pub fn prestige(&mut self, id: &str, auto: bool) -> Option<Big> {
        if !self.can_prestige(id) { return None }
        let g = self.layer_gain(id); let t = self.s.st.run;
        let (n, cur, l) = LAYERS.iter().find(|x| x.0 == id).map(|x| (x.1, x.2, x.3)).unwrap();
        let s = &mut self.s;
        match id { "reb" => { s.frag = s.frag.add(g); s.frag_life = s.frag_life.add(g); s.cnt.reb += 1.0 } "sing" => { s.sing = s.sing.add(g); s.sing_total = s.sing_total.add(g); s.cnt.sing += 1.0 }
            "asc" => { s.cores = s.cores.add(g); s.cores_total = s.cores_total.add(g); s.cnt.asc += 1.0 } "tra" => { s.ve = s.ve.add(g); s.ve_total = s.ve_total.add(g); s.cnt.tra += 1.0 }
            "inf" => { s.ip = s.ip.add(g); s.ip_total = s.ip_total.add(g); s.cnt.inf += 1.0 } _ => { s.up = s.up.add(g); s.up_total = s.up_total.add(g); s.cnt.ult += 1.0 } }
        if id == "reb" { self.s.st.best_reb = self.s.st.best_reb.min(t) }
        if l == 6 { self.ultra_reset() } else { self.reset_layer(l) }
        self.q.push(Note { kind: "prestige".into(), text: format!("{}! +{} {}", n, fmt(g), cur), extra: id.into(), auto });
        Some(g)
    }
    fn ultra_reset(&mut self) {
        let f = self.fx.flag.clone(); let o = std::mem::take(&mut self.s); let mut n = S::default();
        n.ach = o.ach; n.sec = o.sec; n.set = o.set; n.badge = o.badge; n.pets = o.pets; n.story = o.story; n.st = o.st; n.started = o.started; n.up = o.up; n.up_total = o.up_total; n.uupg = o.uupg; n.auto = o.auto; n.life_e = o.life_e; n.v = o.v;
        n.cnt.ult = o.cnt.ult;
        if f.contains("uKeepRes") { n.res = o.res } if f.contains("uKeepArts") { n.arts = o.arts } if f.contains("uKeepBoss") { n.boss.k = o.boss.k } if f.contains("uKeepChal") { n.chal.done = o.chal.done }
        n.st.run = 0.0; n.last = now_ms(); n.ev = o.ev; n.ev.buffs.clear(); n.ev.cur = None;
        n.perm = o.perm; n.daily = o.daily; // 0.20: permanent upgrades and the daily streak survive Ultra
        self.s = n; self.dirty(); self.fx_check(); self.calc();
    }
    pub fn buy_uu(&mut self, id: &str) -> bool {
        let u = match data().uupg.iter().find(|x| x.id == id) { Some(u) => u, None => return false };
        if has(&self.s.uupg, id) || self.s.up.lt(u.c) { return false }
        self.s.up = self.s.up.sub(u.c); self.s.uupg.insert(id.into(), 1.0); self.dirty();
        if id == "omega" && self.s.st.beat == 0.0 { self.s.st.beat = now_ms(); self.notify("beat", "You beat Ten Cells!".into(), "0".into()) }
        true
    }

    // ---------------- shops ----------------
    pub fn perk_cost(id: &str, l: f64) -> f64 {
        (match id { "head" => 2.0 * 3f64.powf(l), "keepAuto" => 25.0, "reso" => 5.0 * 2.5f64.powf(l), "tapM" => 3.0 * 4f64.powf(l), "offl" => 10.0 * 3f64.powf(l), "cheap" => 8.0 * 3f64.powf(l), "rsch" => 15.0 * 2.5f64.powf(l),
            "fgain" => 20.0 * 3f64.powf(l), "mile" => 50.0 * 4f64.powf(l), "boss" => 10.0 * 3f64.powf(l), "evt" => 30.0 * 2.5f64.powf(l), "critP" => 12.0 * 3f64.powf(l), "abCd" => 40.0 * 3f64.powf(l), "autoR" => 60.0, _ => 1e9 }).ceil()
    }
    pub fn perk_lv(&self, id: &str) -> f64 { lv(&self.s.perks, id) }
    pub fn buy_perk(&mut self, id: &str) -> bool {
        let p = match data().perks.iter().find(|x| x.id == id) { Some(p) => p, None => return false }; let l = lv(&self.s.perks, id);
        if l >= p.max as f64 { return false } let c = b(Self::perk_cost(id, l)); if self.s.frag.lt(c) { return false }
        self.s.frag = self.s.frag.sub(c); self.s.perks.insert(id.into(), l + 1.0); self.dirty(); true
    }
    /// shop: "su" Singularity, "au" Ascension, "vu" Void, "uu" Ultra, "pm" permanent (energy)
    pub fn shop_list(k: &str) -> &'static Vec<crate::data::Shop> { let d = data(); match k { "su" => &d.supg, "au" => &d.aupg, "vu" => &d.vupg, "uu" => &d.uupg, "pm" => &d.perm, _ => &d.inf } }
    pub fn shop_owned(&self, k: &str, id: &str) -> bool { has(match k { "su" => &self.s.supg, "au" => &self.s.aupg, "vu" => &self.s.vupg, "uu" => &self.s.uupg, "pm" => &self.s.perm, _ => &self.s.inf }, id) }
    pub fn shop_cur(&self, k: &str) -> Big { match k { "su" => self.s.sing, "au" => self.s.cores, "vu" => self.s.ve, "uu" => self.s.up, "pm" => self.s.energy, _ => self.s.ip } }
    pub fn shop_buy(&mut self, k: &str, id: &str) -> bool {
        if k == "uu" { return self.buy_uu(id) } if k == "inf" { return self.buy_inf(id) }
        let u = match Self::shop_list(k).iter().find(|x| x.id == id) { Some(u) => u, None => return false };
        if self.shop_owned(k, id) || self.shop_cur(k).lt(u.c) { return false }
        let c = u.c; let s = &mut self.s;
        match k { "su" => { s.sing = s.sing.sub(c); s.supg.insert(id.into(), 1.0); } "au" => { s.cores = s.cores.sub(c); s.aupg.insert(id.into(), 1.0); } "vu" => { s.ve = s.ve.sub(c); s.vupg.insert(id.into(), 1.0); } "pm" => { s.energy = s.energy.sub(c); s.perm.insert(id.into(), 1.0); } _ => {} }
        self.dirty(); self.fx_check(); self.calc(); true
    }
    pub fn inf_avail(&self, id: &str) -> bool { data().inf.iter().find(|n| n.id == id).map(|n| !has(&self.s.inf, id) && n.req.iter().all(|r| has(&self.s.inf, r))).unwrap_or(false) }
    pub fn buy_inf(&mut self, id: &str) -> bool {
        let n = match data().inf.iter().find(|x| x.id == id) { Some(n) => n, None => return false };
        if !self.inf_avail(id) || self.s.ip.lt(n.c) { return false }
        self.s.ip = self.s.ip.sub(n.c); self.s.inf.insert(id.into(), 1.0); self.dirty(); true
    }
    /// 0.19 Buy max: cheapest first, again and again. Never buys the Omega Engine.
    pub fn shop_max(&mut self, k: &str) -> usize {
        let mut n = 0;
        for _ in 0..2000 {
            self.fx_check();
            let mut best: Option<(Big, String)> = None;
            if k == "perk" { for p in &data().perks { let l = lv(&self.s.perks, &p.id); if l < p.max as f64 { let c = b(Self::perk_cost(&p.id, l)); if self.s.frag.gte(c) && best.as_ref().map(|x| c.lt(x.0)).unwrap_or(true) { best = Some((c, p.id.clone())) } } } }
            else { for u in Self::shop_list(k) { if u.id == "omega" || self.shop_owned(k, &u.id) || (k == "inf" && !self.inf_avail(&u.id)) { continue } if self.shop_cur(k).gte(u.c) && best.as_ref().map(|x| u.c.lt(x.0)).unwrap_or(true) { best = Some((u.c, u.id.clone())) } } }
            let ok = match best { None => false, Some((_, id)) => if k == "perk" { self.buy_perk(&id) } else { self.shop_buy(k, &id) } };
            if !ok { break } n += 1;
        }
        n
    }
    pub fn feed_bh(&mut self, pct: f64) -> bool { if !self.bh_on() { return false } if self.s.energy.ltf(1.0) { self.s.sec.insert("feedNothing".into(), json!(1)); } let x = self.s.energy.mulf(pct); self.s.energy = self.s.energy.sub(x); self.s.bh = self.s.bh.add(x); true }

    // ---------------- research ----------------
    pub fn tier_open(&self, t: usize) -> bool { [self.s.cnt.reb, self.s.cnt.sing, self.s.cnt.asc, self.s.cnt.tra][t.min(3)] > 0.0 }
    pub fn rp_rate(&self) -> f64 { let mut r = self.fx.rp; if self.dim_on("temporal") { r *= 1.0 + oom(self.dim_r("temporal")) * self.reso("temporal") } r }
    pub fn res_avail(&self, i: usize) -> bool { let r = &data().res[i]; !has(&self.s.res, &r.id) && self.tier_open(r.tier) && (i % 4 == 0 || has(&self.s.res, &format!("x{}", i - 1))) }
    pub fn buy_res(&mut self, i: usize) -> bool { let r = &data().res[i]; if !self.res_avail(i) || self.s.rp.lt(r.c) { return false } self.s.rp = self.s.rp.sub(r.c); self.s.res.insert(r.id.clone(), 1.0); self.dirty(); true }

    // ---------------- challenges ----------------
    pub fn chal_open(&self) -> bool { self.s.cnt.sing > 0.0 }
    pub fn start_chal(&mut self, id: &str, l: u32) -> bool { if self.s.chal.a.is_some() || !self.chal_open() || l > 3 || lv(&self.s.chal.done, id) < l as f64 - 1.0 { return false } self.reset_layer(1); self.s.chal.a = Some(ChalA { id: id.into(), l }); true }
    pub fn exit_chal(&mut self) { if self.s.chal.a.is_none() { return } self.s.chal.a = None; self.reset_layer(1) }
    fn complete_chal(&mut self) {
        let a = self.s.chal.a.take().unwrap(); let fast = self.s.st.run < 60.0;
        let cur = lv(&self.s.chal.done, &a.id); self.s.chal.done.insert(a.id.clone(), cur.max(a.l as f64)); self.s.st.chal += 1.0;
        if fast { self.s.sec.insert("speedrun".into(), json!(1)); }
        self.dirty(); self.reset_layer(1);
        let n = data().chal.iter().find(|c| c.id == a.id).map(|c| c.n.clone()).unwrap_or_default();
        self.notify("chal", format!("Challenge complete: {} {}", n, "I".repeat(a.l as usize)), String::new());
    }

    // ---------------- dimensions ----------------
    pub fn dim_unlock_ok(&self, i: usize) -> bool {
        let d = &data().dims[i]; if self.dim_on(&d.id) || self.s.cnt.asc < 1.0 { return false }
        if d.tra > 0 && self.s.cnt.tra < d.tra as f64 { return false }
        if let Some((pid, need)) = &d.cond { if !self.dim_on(pid) || self.dim_r(pid).ltf(*need) { return false } }
        true
    }
    pub fn unlock_dim(&mut self, i: usize) -> bool {
        let d = &data().dims[i]; if !self.dim_unlock_ok(i) || self.s.cores.ltf(d.cost) { return false }
        self.s.cores = self.s.cores.sub(b(d.cost)); let x = self.s.dims.get_mut(&d.id).unwrap(); x.u = 1.0; x.r = b(10.0); true
    }
    pub fn dim_mult(&self, id: &str) -> Big {
        let dd = &self.s.dims[id]; let i = crate::data::dim_idx(id); let ds = &data().dims;
        let mut m = self.fx.dim_mult.mul(b(2.0).pow(dd.amp));
        if dd.link != 0.0 { m = m.mulf(1.0 + oom(if i > 0 { self.dim_r(&ds[i - 1].id) } else { self.s.energy })) }
        if self.dim_on("infinite") { m = m.mulf(1.0 + oom(self.dim_r("infinite")) * self.reso("infinite")) }
        if self.law("expansion") { m = m.mulf(self.law_b(100.0)) }
        m.mulf(match id { "plasma" => 5.0, "quantum" => self.s.qr.m, "stellar" => 1.0 + oom(self.s.energy) / 20.0, "void" => (1.0 + self.s.idle / 30.0).min(10.0), "celestial" => self.celestial_phase(),
            "fractal" => 1.5f64.powf((self.dims_unlocked() as f64 - 1.0).max(0.0)), "mirror" => if self.s.idle < 10.0 { 3.0 } else { 0.5 }, "dream" => (1.0 + self.s.st.run / 190.0).min(20.0), "origin" => if self.origin_flare() { 10.0 } else { 1.0 }, _ => 1.0 })
    }
    pub fn origin_flare(&self) -> bool { self.s.st.play % 60.0 < 10.0 }
    pub fn celestial_phase(&self) -> f64 { 1.6 + 1.4 * (self.s.st.play * 2.0 * std::f64::consts::PI / 120.0).sin() }
    pub fn dim_eff(dd: &DimS, k: usize) -> Big { b(dd.g[k].b).add(dd.g[k].a).mul(b(2.0).pow((dd.g[k].b / 10.0).floor())) }
    pub fn dim_rate(&self, id: &str) -> Big { Self::dim_eff(&self.s.dims[id], 0).mulf(data().dgen[0].p).mul(self.dim_mult(id)) }
    fn dim_tick(&mut self, dt: f64) {
        let q = &mut self.s.qr; q.t -= dt; q.lock = (q.lock - dt).max(0.0); q.cd = (q.cd - dt).max(0.0);
        if q.t <= 0.0 { q.t = 5.0; if q.lock <= 0.0 { q.m = if self.fx.f("paradox") { 1.0 + rnd() * 3.0 } else { rnd() * 4.0 } } }
        self.s.tskip_cd = (self.s.tskip_cd - dt).max(0.0);
        let dg = &data().dgen;
        for d in &data().dims {
            if !self.dim_on(&d.id) { continue }
            let m = self.dim_mult(&d.id); let dd = self.s.dims.get_mut(&d.id).unwrap();
            let e2 = Self::dim_eff(dd, 2); dd.g[1].a = dd.g[1].a.add(e2.mulf(dg[2].p * dt));
            let e1 = Self::dim_eff(dd, 1); dd.g[0].a = dd.g[0].a.add(e1.mulf(dg[1].p * dt));
            let p = Self::dim_eff(dd, 0).mulf(dg[0].p).mul(m);
            if d.id == "plasma" { let k = -(0.98f64.ln()); let e = (-k * dt).exp(); dd.r = dd.r.mulf(e).add(p.mulf((1.0 - e) / k)) } else { dd.r = dd.r.add(p.mulf(dt)) }
        }
    }
    pub fn dim_gen_cost(dd: &DimS, k: usize) -> Big { let g = &data().dgen[k]; b(g.c).mul(b(g.r).pow(dd.g[k].b)) }
    pub fn buy_dim_gen(&mut self, id: &str, k: usize, max: bool) -> u32 {
        let dd = self.s.dims.get_mut(id).unwrap(); if dd.u == 0.0 { return 0 } let mut n = 0;
        loop { let c = Self::dim_gen_cost(dd, k); if dd.r.lt(c) { break } dd.r = dd.r.sub(c); dd.g[k].b += 1.0; n += 1; if !max || n >= 1000 { break } }
        n
    }
    pub fn dupg_cost(dd: &DimS, k: &str) -> Big { match k { "amp" => b(100.0).mul(b(20.0).pow(dd.amp)), "reso" => b(1e4).mul(b(1e3).pow(dd.reso)), _ => b(1e6) } }
    pub fn dupg_max(k: &str) -> f64 { match k { "amp" => f64::INFINITY, "reso" => 4.0, _ => 1.0 } }
    pub fn buy_dim_upg(&mut self, id: &str, k: &str) -> bool {
        let dd = self.s.dims.get_mut(id).unwrap(); let cur = match k { "amp" => dd.amp, "reso" => dd.reso, _ => dd.link };
        if dd.u == 0.0 || cur >= Self::dupg_max(k) { return false } let c = Self::dupg_cost(dd, k); if dd.r.lt(c) { return false }
        dd.r = dd.r.sub(c); match k { "amp" => dd.amp += 1.0, "reso" => dd.reso += 1.0, _ => dd.link += 1.0 } true
    }
    pub fn observe(&mut self) -> bool { if !self.dim_on("quantum") || self.s.qr.cd > 0.0 { return false } self.s.qr.lock = 60.0; self.s.qr.cd = 120.0; true }
    pub fn time_skip(&mut self) -> bool {
        let dd = self.s.dims.get_mut("temporal").unwrap(); if dd.u == 0.0 || self.s.tskip_cd > 0.0 || dd.r.ltf(10.0) { return false }
        let spent = dd.r.divf(2.0); dd.r = dd.r.sub(spent);
        let secs = 3600f64.min(oom(spent) * 300.0); let v = self.c.eps.mulf(secs * self.c.sp);
        self.gain(v); self.s.tskip_cd = 30.0; self.notify("gain", format!("Time Skip: {} min → +{} energy", (secs / 60.0).round(), fmt(v)), String::new()); true
    }
    pub fn dim_needs(&self, i: usize) -> Vec<(String, bool, f64, String)> {
        let ds = &data().dims; let d = &ds[i]; let mut out = vec![("Ascend at least once".to_string(), self.s.cnt.asc >= 1.0, self.s.cnt.asc.min(1.0), String::new())];
        if let Some((pid, need)) = &d.cond {
            let prev = &ds[crate::data::dim_idx(pid)]; let on = self.dim_on(pid); let r = self.dim_r(pid); let g = b(*need);
            out.push((format!("Unlock {} first", prev.n), on, if on { 1.0 } else { 0.0 }, String::new()));
            out.push((format!("Grow {}'s {} to {}", prev.n, prev.res, fmt(g)), on && r.gte(g), if on { (r.maxf(1.0).log10() / g.log10()).clamp(0.0, 1.0) } else { 0.0 }, format!("{} / {}", fmt(r), fmt(g))));
        }
        if d.tra > 0 { out.push((format!("Transcend {} time{}", d.tra, if d.tra > 1 { "s" } else { "" }), self.s.cnt.tra >= d.tra as f64, (self.s.cnt.tra / d.tra as f64).min(1.0), format!("{} / {}", self.s.cnt.tra, d.tra))) }
        out.push((format!("Have {} Ascension Core{}", d.cost, if d.cost > 1.0 { "s" } else { "" }), self.s.cores.gtef(d.cost), (self.s.cores.to_f64() / d.cost).min(1.0), format!("{} / {}", fmt(self.s.cores), d.cost)));
        out
    }
    pub fn top_dim(&self) -> Option<usize> { (0..data().dims.len()).rev().find(|&i| self.dim_on(&data().dims[i].id)) }

    // ---------------- reality laws ----------------
    pub fn laws_open(&self) -> bool { self.s.cnt.tra > 0.0 }
    pub fn toggle_law(&mut self, id: &str) -> bool {
        if !self.laws_open() || self.s.law_cd > 0.0 { return false }
        if self.law(id) { self.s.laws.retain(|x| x != id) } else if (self.s.laws.len() as f64) < self.fx.law_slots { self.s.laws.push(id.into()) } else { return false }
        self.s.law_cd = 10.0; self.dirty(); true
    }

    // ---------------- automation ----------------
    fn auto_rule(&self, k: &str, g: Big) -> bool {
        let a = &self.s.auto; let m = a.rule(k, "M");
        if m == "time" { return g.gtef(1.0) && self.s.st.run >= a.rule(k, "T").parse::<f64>().unwrap_or(10.0).max(0.1) * 60.0 }
        if m == "mult" { let cur = match k { "reb" => self.s.frag, "sing" => self.s.sing, "asc" => self.s.cores, "tra" => self.s.ve, _ => self.s.ip }; return g.gtef(1.0) && g.gte(cur.maxf(1.0).mulf(a.rule(k, "X").parse::<f64>().unwrap_or(2.0).max(0.01))) }
        g.gte(Big::parse(a.at(k)).filter(|x| x.m.is_finite()).unwrap_or(b(1.0)))
    }
    pub fn auto_keys(&self) -> Vec<String> { let mut k = vec![]; if self.fx.autotap > 0.0 { k.push("tap".into()) } if self.fx.auto_upg > 0.0 { k.push("upg".into()) } if self.fx.auto_ab > 0.0 { k.push("ab".into()) } for (i, x) in self.fx.ag.iter().enumerate() { if *x { k.push(format!("g{i}")) } } k }
    pub fn auto_name(k: &str) -> String { match k { "tap" => "Auto-tap".into(), "upg" => "Upgrade daemon".into(), "ab" => "Ability daemon".into(), _ => format!("{} autobuyer", data().gens[k[1..].parse::<usize>().unwrap_or(0)].n) } }
    /// 0.20: automations start OFF the first time they unlock, with a note saying where to turn them on
    pub fn auto_watch(&mut self) {
        let now = self.auto_keys();
        if self.s.set.auto_seen.is_none() { self.s.set.auto_seen = Some(now.iter().map(|k| (k.clone(), 1.0)).collect()); return }
        let seen = self.s.set.auto_seen.as_mut().unwrap(); let fresh: Vec<String> = now.into_iter().filter(|k| !seen.contains_key(k)).collect(); if fresh.is_empty() { return }
        for k in &fresh { seen.insert(k.clone(), 1.0); if let Some(i) = k.strip_prefix('g').and_then(|x| x.parse::<usize>().ok()) { self.s.auto.gens[i] = 0.0 } else { self.s.auto.set_on(k, false) } }
        let msg = if fresh.len() > 1 { format!("{} new automations unlocked. They start off: switch them on in Automation", fresh.len()) } else { format!("New automation: {}. It starts off: switch it on in Automation", Self::auto_name(&fresh[0])) };
        self.toast(msg, "gear", "#8a63ff");
    }
    fn automate(&mut self) {
        self.auto_watch();
        if !self.law("echoes") { for i in (0..NG).rev() { if self.fx.ag[i] && self.s.auto.gens[i] > 0.0 { self.buy_gen(i, if self.fx.auto_max > 0.0 { 0.0 } else { 1.0 }); } } }
        if self.fx.auto_upg > 0.0 && self.s.auto.upg > 0.0 && !self.in_c("noupg") {
            let mut av: Vec<_> = data().upg.iter().filter(|u| self.upg_avail(u)).collect(); av.sort_by(|a, b| self.upg_cost(a).cmp(&self.upg_cost(b)));
            let ids: Vec<String> = av.iter().take(5).map(|u| u.id.clone()).collect(); for id in ids { if !self.buy_upg(&id) { break } }
        }
        if self.fx.auto_ab > 0.0 && self.s.auto.ab > 0.0 { for id in self.fx.ab.clone() { if self.ab_ready(&id) { self.use_ab(&id); } } }
        for (k, fl) in [("inf", "autoInf"), ("tra", "autoTra"), ("asc", "autoAsc"), ("sing", "autoSing"), ("reb", "autoReb")] {
            if self.fx.f(fl) && self.s.auto.on(k) && self.can_prestige(k) && self.auto_rule(k, self.layer_gain(k)) { self.prestige(k, true); return }
        }
    }

    // ---------------- the main tick ----------------
    pub fn tick(&mut self, dt: f64, off: bool) {
        self.fx_check(); self.calc();
        self.s.st.play += dt; self.s.st.run += dt; self.s.idle += dt;
        if !off { self.s.st.act += dt; self.session += dt; let day = today(); if self.s.st.last_day != day { if !self.s.st.last_day.is_empty() { self.s.st.day_n += 1.0 } self.s.st.last_day = day } }
        self.heat *= 0.75f64.powf(dt);
        let sdt = dt * self.c.sp;
        let e = self.c.eps.mulf(sdt); self.gain(e);
        if self.fx.autotap > 0.0 && self.s.auto.tap > 0.0 && !self.in_c("notap") { let v = self.c.tap.mulf(1.0 + self.fx.crit_c.min(1.0) * (self.fx.crit_m - 1.0)).mulf(self.fx.autotap * dt); self.gain(v) }
        if self.in_c("decay") { self.s.energy = self.s.energy.mulf(0.98f64.powf(sdt)) }
        if self.in_c("hunger") { let x = self.s.energy.mulf(1.0 - 0.95f64.powf(sdt)); self.s.energy = self.s.energy.sub(x); self.s.bh = self.s.bh.add(x) }
        if self.bh_on() {
            if self.fx.bh_passive > 0.0 { self.s.bh = self.s.bh.add(self.c.eps.mulf(self.fx.bh_passive * sdt)) }
            if self.s.auto.feed > 0.0 { let x = self.s.energy.mulf((self.s.auto.feed / 100.0 * dt).min(1.0)); self.s.energy = self.s.energy.sub(x); self.s.bh = self.s.bh.add(x) }
        }
        self.dim_tick(sdt);
        if self.s.cnt.reb > 0.0 { self.s.rp = self.s.rp.addf(self.rp_rate() * dt) }
        if !off { self.ev_tick(dt) }
        for x in self.s.ev.buffs.iter_mut() { x.t -= dt } self.s.ev.buffs.retain(|x| x.t > 0.0);
        for v in self.s.ab.values_mut() { *v = (*v - dt).max(0.0) }
        self.boss_tick(sdt);
        self.s.law_cd = (self.s.law_cd - dt).max(0.0);
        for e in self.s.pets.eggs.iter_mut() { e.t = (e.t - dt).max(0.0) }
        if self.c.eps.gt(self.s.st.max_eps) { self.s.st.max_eps = self.c.eps }
        if let Some(a) = &self.s.chal.a { if self.s.run_e.gte(data().chal_goal[a.l as usize - 1]) { self.complete_chal() } }
        self.auto_t += dt; if self.auto_t >= self.fx.auto_int { self.auto_t = 0.0; self.automate() }
        self.ach_t += dt; if self.ach_t >= 1.0 { self.ach_t = 0.0; self.check_ach(); self.check_badges() }
    }

    // ---------------- pets ----------------
    pub fn pet_stage(lvl: f64) -> usize { if lvl >= 20.0 { 2 } else if lvl >= 10.0 { 1 } else { 0 } }
    pub fn egg_cost(&self) -> Big { self.c.eps.mulf(120.0).maxf(1e4).mul(b(4.0).pow(self.s.pets.bought)) }
    pub fn feed_cost(&self, id: &str) -> Big { self.c.eps.mulf(60.0).maxf(1e3).mul(b(1.35).pow(self.s.pets.own[id].lvl)) }
    fn roll_rar(min_r: usize) -> usize { let w: Vec<f64> = data().egg_w.iter().enumerate().map(|(i, x)| if i >= min_r { *x } else { 0.0 }).collect(); let mut r = rnd() * w.iter().sum::<f64>(); for (i, x) in w.iter().enumerate() { r -= x; if r <= 0.0 { return i } } min_r }
    pub fn give_egg(&mut self, min_r: usize, force: bool) -> bool {
        if !force && self.s.pets.eggs.len() >= 3 { return false }
        let r = Self::roll_rar(min_r); let t = data().egg_time[r]; self.s.pets.eggs.push(Egg { r, t });
        self.notify("egg", format!("{} egg found! It will hatch in {}.", data().rar[r], fmt_t(t)), r.to_string()); true
    }
    pub fn buy_egg(&mut self) -> bool { if self.s.pets.eggs.len() >= 3 { return false } let c = self.egg_cost(); if self.s.energy.lt(c) { return false } self.s.energy = self.s.energy.sub(c); self.s.pets.bought += 1.0; self.give_egg(0, false) }
    pub fn hatch(&mut self, i: usize) -> Option<String> {
        let e = self.s.pets.eggs.get(i)?; if e.t > 0.0 { return None } let er = e.r; self.s.pets.eggs.remove(i);
        let pets = &data().pets; let mut pool: Vec<_> = pets.iter().filter(|p| p.r == er).collect(); let mut r = er as i32; while pool.is_empty() && r >= 0 { pool = pets.iter().filter(|p| p.r as i32 == r).collect(); r -= 1 }
        let p = pool[(rnd() * pool.len() as f64) as usize % pool.len()];
        if let Some(o) = self.s.pets.own.get_mut(&p.id) { o.lvl = (o.lvl + 3.0).min(30.0); self.notify("pet", format!("{} again! Your {} gains 3 levels.", p.n, p.n), p.id.clone()) }
        else { self.s.pets.own.insert(p.id.clone(), Own { lvl: 1.0 }); self.notify("pet", format!("{} hatched!", p.n), p.id.clone()) }
        if self.s.pets.active.is_none() { self.s.pets.active = Some(p.id.clone()) }
        self.s.pets.hatched += 1.0; self.dirty(); Some(p.id.clone())
    }
    pub fn feed_pet(&mut self, id: &str) -> bool {
        let lvl = match self.s.pets.own.get(id) { Some(o) => o.lvl, None => return false }; if lvl >= 30.0 { return false }
        let c = self.feed_cost(id); if self.s.energy.lt(c) { return false }
        self.s.energy = self.s.energy.sub(c); self.s.pets.own.get_mut(id).unwrap().lvl += 1.0; self.dirty();
        if Self::pet_stage(lvl + 1.0) > Self::pet_stage(lvl) { let n = data().pets.iter().find(|p| p.id == id).unwrap().n.clone(); self.notify("pet", format!("{} evolved: {}!", n, data().pet_stage[Self::pet_stage(lvl + 1.0)]), id.into()) }
        true
    }
    pub fn set_active_pet(&mut self, id: &str) { if self.s.pets.own.contains_key(id) { self.s.pets.active = Some(id.into()); self.dirty() } }

    // ---------------- story ----------------
    pub fn chapter_i(&self) -> usize { (self.s.story.ch as usize).min(data().chapters.len() - 1) }
    pub fn story_done(&self) -> bool { self.s.story.ch as usize >= data().chapters.len() }
    pub fn mission_val(&self, m: &crate::data::Mission) -> (f64, f64) {
        let s = &self.s; let n = m.n; let a = &m.a; let ai = a.as_f64().unwrap_or(0.0) as usize; let astr = a.as_str().unwrap_or("");
        let v = match m.k.as_str() {
            "act" => s.st.act / 60.0, "boss" => if s.boss.k.contains_key(&ai.to_string()) { 1.0 } else { 0.0 }, "dimu" => if self.dim_on(astr) { 1.0 } else { 0.0 },
            "dimoom" => if self.dim_on(astr) { 12f64.min(oom(self.dim_r(astr)).floor()) } else { 0.0 }, "taps" => s.st.taps, "gen" => s.gens[ai], "epsoom" => 3f64.min(oom(s.st.max_eps).floor()),
            "upg" => s.st.upg, "ab" => s.st.ab, "ev" => s.st.ev, "runoom" => n.min(oom(s.run_e).floor()), "reb" => s.cnt.reb, "perks" => s.perks.values().sum(), "res" => s.res.len() as f64,
            "hatched" => s.pets.hatched, "sing" => s.cnt.sing, "bhoom" => n.min(oom(s.bh).floor()), "chal" => s.st.chal, "asc" => s.cnt.asc, "dims" => self.dims_unlocked() as f64, "arts" => s.arts.len() as f64,
            "tra" => s.cnt.tra, "laws" => s.laws.len() as f64, "bosses" => s.boss.k.len() as f64, "petlvl" => s.pets.own.values().map(|o| o.lvl).fold(0.0, f64::max), "inf" => s.cnt.inf,
            "break" => if has(&s.inf, "break") { 1.0 } else { 0.0 }, "ipt" => n.min(s.ip_total.to_f64()), "infn" => s.inf.len() as f64, "nap" => if s.sec.contains_key("nap") { 1.0 } else { 0.0 },
            "ult" => s.cnt.ult, "uupg" => s.uupg.len() as f64, _ => 0.0,
        };
        (v, n)
    }
    pub fn missions_done(&self, ch: usize) -> bool { data().missions.get(ch).map(|ms| ms.iter().all(|m| { let (a, b) = self.mission_val(m); a >= b })).unwrap_or(false) }
    pub fn missions_half(&self, ch: usize) -> bool { data().missions.get(ch).map(|ms| { let d = ms.iter().filter(|m| { let (a, b) = self.mission_val(m); a >= b }).count(); d * 2 >= ms.len() }).unwrap_or(false) }
    pub fn claim_chapter(&mut self) -> bool {
        if self.story_done() || !self.missions_done(self.s.story.ch as usize) { return false }
        let ch = self.s.story.ch as usize; self.s.story.claimed += 1.0; self.s.story.ch += 1.0; self.dirty();
        self.give_egg((ch / 8).min(4), true);
        self.notify("story", format!("Chapter complete: {}. Production ×{} forever.", data().chapters[ch].n, data().story_m), ch.to_string()); true
    }

    // ---------------- achievements and badges ----------------
    pub fn ach_ok(&self, id: &str) -> bool {
        let s = &self.s; let d = data();
        let num = |p: &str| -> Option<Big> { id.strip_prefix(p).and_then(Big::parse) };
        match id {
            "omega" => return s.st.beat > 0.0, "upall" => return d.upg.iter().all(|u| self.has_u(&u.id)), "laws3" => return s.laws.len() >= 3,
            "chall3" => return d.chal.iter().all(|c| lv(&s.chal.done, &c.id) >= 3.0), "artmyth" => return d.arts.iter().any(|a| a.r == 4 && has(&s.arts, &a.id)), "artmax" => return s.arts.values().any(|l| *l >= 5.0),
            "pet1" => return s.pets.hatched >= 1.0, "pet4" => return s.pets.own.len() >= 4, "pet8" => return s.pets.own.len() >= 8, "petmax" => return s.pets.own.values().any(|p| p.lvl >= 30.0),
            "ch1" => return s.story.claimed >= 1.0, "ch5" => return s.story.claimed >= 5.0, "ch10" => return s.story.claimed >= 10.0, "ch20" => return s.story.claimed >= 20.0, "ch30" => return s.story.claimed >= 30.0, "chAll" => return s.story.claimed >= d.chapters.len() as f64,
            "l_own1k" => return s.gens[..10].iter().all(|g| *g >= 1000.0), "l_ladder" => return s.gens.iter().all(|g| *g > 0.0), "l_crown" => return s.gens[NG - 1] >= 100.0,
            "l_e1000" => return s.run_e.gte(bs("1e1000")), "l_e10k" => return s.life_e.gte(bs("1e10000")), "l_eps" => return s.st.max_eps.gte(bs("1e1000")), "l_tap10m" => return s.st.taps >= 1e7, "l_crit" => return s.st.crits >= 250000.0,
            "l_reb1k" => return s.cnt.reb >= 1000.0, "l_reb5k" => return s.cnt.reb >= 5000.0, "l_sing500" => return s.cnt.sing >= 500.0, "l_asc250" => return s.cnt.asc >= 250.0, "l_tra100" => return s.cnt.tra >= 100.0,
            "l_inf1k" => return s.cnt.inf >= 1000.0, "l_ult100" => return s.cnt.ult >= 100.0, "l_speed" => return s.cnt.reb > 0.0 && s.st.best_reb <= 30.0,
            "l_perks" => return d.perks.iter().all(|p| lv(&s.perks, &p.id) >= p.max as f64), "l_infall" => return d.inf.iter().all(|n| has(&s.inf, &n.id)), "l_ultall" => return d.uupg.iter().all(|u| has(&s.uupg, &u.id)),
            "l_boss" => return d.bosses.iter().all(|x| s.boss.k.contains_key(&x.i.to_string())), "l_pets" => return d.pets.iter().all(|p| s.pets.own.get(&p.id).map(|o| o.lvl >= 30.0).unwrap_or(false)),
            "l_frag" => return s.frag.gte(bs("1e100")), "l_bh" => return s.bh.gte(bs("1e500")), "l_dims" => return d.dims.iter().all(|x| self.dim_on(&x.id) && self.dim_r(&x.id).gtef(1e50)),
            "l_ev" => return s.st.ev >= 5000.0, "l_ab" => return s.st.ab >= 20000.0, "l_play" => return s.st.play >= 500.0 * 3600.0, "l_hands" => return s.idle >= 3600.0, "l_vigil" => return self.session >= 36000.0,
            "l_all" => return d.ach.iter().all(|a| a.id == "l_all" || has(&s.ach, &a.id)),
            _ => {}
        }
        if let Some(k) = id.strip_prefix("h_") { return if k == "glitch5" { s.st.glitch >= 5.0 } else { s.sec.contains_key(k) } }
        if let Some(k) = id.strip_prefix("ch_") { return lv(&s.chal.done, k) >= 1.0 }
        if let Some(k) = id.strip_prefix("evt_") { return lv(&s.st.ev_t, k) > 0.0 }
        if let Some(k) = id.strip_prefix("dim") { if let Ok(i) = k.parse::<usize>() { return d.dims.get(i).map(|x| self.dim_on(&x.id)).unwrap_or(false) } }
        if let Some(k) = id.strip_prefix("boss") { if let Ok(i) = k.parse::<usize>() { return s.boss.k.contains_key(&i.to_string()) } }
        if id.starts_with('g') && id.contains('_') { let (a, n) = id[1..].split_once('_').unwrap(); if let (Ok(i), Ok(n)) = (a.parse::<usize>(), n.parse::<f64>()) { return s.gens.get(i).map(|g| *g >= n).unwrap_or(false) } }
        if let Some(n) = num("eps") { return s.st.max_eps.gte(n) }
        if let Some(n) = num("ev") { return s.st.ev >= n.to_f64() }
        if let Some(n) = num("tap") { return s.st.taps >= n.to_f64() }
        if let Some(n) = num("ult") { return s.cnt.ult >= n.to_f64() }
        if let Some(n) = num("up") { return s.st.upg >= n.to_f64() }
        if let Some(n) = num("reb") { return s.cnt.reb >= n.to_f64() }
        if let Some(n) = num("fr") { return s.frag_life.gte(n) }
        if let Some(n) = num("sing") { return s.cnt.sing >= n.to_f64() }
        if let Some(n) = num("bh") { return s.bh.gte(n) }
        if let Some(n) = num("asc") { return s.cnt.asc >= n.to_f64() }
        if let Some(n) = num("tra") { return s.cnt.tra >= n.to_f64() }
        if let Some(n) = num("inf") { return s.cnt.inf >= n.to_f64() }
        if let Some(n) = num("art") { return s.arts.len() as f64 >= n.to_f64() }
        if let Some(n) = num("res") { return s.res.len() as f64 >= n.to_f64() }
        if let Some(n) = num("ab") { return s.st.ab >= n.to_f64() }
        if let Some(x) = id.strip_prefix('e').and_then(|k| k.parse::<f64>().ok()) { return s.life_e.gte(bs(&format!("1e{}", x))) }
        false
    }
    fn count_ach(&mut self) { let d = data(); let (mut n, mut h) = (0, 0); for a in &d.ach { if has(&self.s.ach, &a.id) { n += 1; if a.h { h += 1 } } } self.ach_n = n; self.hid_n = h; }
    pub fn check_ach(&mut self) {
        if local_hour() == 3 { self.s.sec.insert("owl".into(), json!(1)); }
        for a in &data().ach { if !has(&self.s.ach, &a.id) && self.ach_ok(&a.id) { self.s.ach.insert(a.id.clone(), 1.0); self.notify("ach", a.n.clone(), a.id.clone()); self.dirty(); } }
        self.count_ach();
    }
    pub fn badge_val(&self, id: &str) -> f64 {
        let s = &self.s; let d = data();
        match id { "tap" => s.st.taps, "crit" => s.st.crits, "build" => s.gens.iter().sum(), "upg" => s.st.upg, "hoard" => s.life_e.log10().max(0.0), "reb" => s.cnt.reb, "sing" => s.cnt.sing, "asc" => s.cnt.asc, "tra" => s.cnt.tra, "inf" => s.cnt.inf,
            "kill" => s.st.kills, "art" => s.arts.len() as f64, "ach" => s.ach.len() as f64, "ev" => s.st.ev, "chal" => s.st.chal, "pet" => s.pets.hatched, "story" => s.story.claimed, "vet" => (now_ms() - s.started) / 864e5,
            "eps" => s.st.max_eps.log10().max(0.0), "abil" => s.st.ab, "res" => s.res.len() as f64, "perk" => s.perks.values().sum(), "dim" => self.dims_unlocked() as f64, "hole" => s.bh.log10().max(0.0),
            "tier" => s.gens.iter().enumerate().fold(0.0, |m, (i, g)| if *g > 0.0 { (i + 1) as f64 } else { m }), "hunt" => d.bosses.iter().fold(0.0, |m, x| if s.boss.k.contains_key(&x.i.to_string()) { (x.i + 1) as f64 } else { m }),
            "love" => s.pets.own.values().map(|o| o.lvl).sum(), "evo" => s.pets.own.values().map(|o| (o.lvl >= 10.0) as i32 as f64 + (o.lvl >= 20.0) as i32 as f64).sum(), "time" => s.st.play / 3600.0, "ult" => s.cnt.ult,
            "upts" => s.up_total.to_f64(), "secret" => s.sec.len() as f64, _ => 0.0 }
    }
    pub fn check_badges(&mut self) {
        for bd in &data().badges {
            let v = self.badge_val(&bd.id); let v = if v.is_finite() { v } else { 0.0 };
            let tier = bd.th.iter().filter(|x| v >= **x).count() as f64; let had = lv(&self.s.badge, &bd.id);
            if tier > had { self.s.badge.insert(bd.id.clone(), tier); self.notify("badge", format!("Badge: {} {}", bd.n, data().btier[tier as usize - 1].0), format!("{}:{}", bd.id, tier)) }
        }
    }

    // ---------------- daily rewards ----------------
    pub fn daily_ready(&self) -> bool { self.s.daily.last != today() }
    pub fn prod_for(&self, secs: f64) -> Big { self.c.eps.mulf(self.c.sp).mulf(secs).maxf(1000.0) }
    pub fn claim_daily(&mut self) -> Option<(usize, String)> {
        if !self.daily_ready() { return None }
        let k = (self.s.daily.day as usize) % 7;
        let msg = match k {
            0 => { let v = self.prod_for(600.0); self.gain(v); format!("+{} energy", fmt(v)) }
            1 => { let v = self.prod_for(1800.0); self.gain(v); format!("+{} energy", fmt(v)) }
            2 => { self.give_egg(1, true); "A pet egg is waiting in the Crew tab".into() }
            3 => if self.s.cnt.reb > 0.0 { let f = self.layer_gain("reb").mulf(0.3).maxf(5.0).floor(); self.s.frag = self.s.frag.add(f); self.s.frag_life = self.s.frag_life.add(f); format!("+{} Fragments", fmt(f)) } else { let v = self.prod_for(3600.0); self.gain(v); format!("+{} energy", fmt(v)) },
            4 => { self.boost(2.0, 24.0); "Energy ×2 for the next 24 hours".into() }
            5 => { let v = self.prod_for(7200.0); self.gain(v); format!("+{} energy", fmt(v)) }
            _ => { self.boost(3.0, 6.0); self.give_egg(2, true); "Energy ×3 for 6 hours, and a rare egg".into() }
        };
        self.s.daily.last = today(); self.s.daily.day += 1.0; self.s.daily.n += 1.0; self.dirty(); self.fx_check(); self.calc();
        Some((k, msg))
    }
    pub fn buy_perm(&mut self, id: &str) -> bool { self.shop_buy("pm", id) }

    // ---------------- offline progress ----------------
    pub fn offline(&mut self) -> Option<(f64, f64, f64, Big)> {
        let away = (now_ms() - self.s.last) / 1000.0; if !(away > 10.0) { return None }
        self.fx_check(); if away >= 43200.0 { self.s.sec.insert("nap".into(), json!(1)); }
        let cap = self.off_h() * 3600.0; let used = away.min(cap); let e0 = self.s.life_e;
        let steps = 600f64.min((used / 2.0).ceil()).max(1.0); let h = used / steps;
        let mut got = Big::ZERO; for _ in 0..steps as usize { self.tick(h, true); got = got.add(self.c.eps.mulf(h * self.c.sp)) }
        self.s.last = now_ms(); let _ = e0; Some((away, used, cap, got))
    }
}
/// [id, name, currency, layer]
pub const LAYERS: [(&str, &str, &str, u32); 6] = [("reb", "Rebirth", "Fragments", 1), ("sing", "Singularity", "Singularities", 2), ("asc", "Ascension", "Ascension Cores", 3), ("tra", "Transcendence", "Void Essence", 4), ("inf", "Infinity", "Infinity Points", 5), ("ult", "Ultra Prestige", "Ultra Points", 6)];
pub fn chal_eff(id: &str, l: f64) -> Vec<(String, Value)> {
    let e = |k: &str, v: f64| (k.to_string(), json!(v));
    match id { "notap" => vec![e("tapMult", 10f64.powf(l))], "lone" => vec![e("gen0", 100f64.powf(l)), e("gen1", 100f64.powf(l))], "inflation" => vec![e("growthRed", 0.005 * l)], "noupg" => vec![e("upgCost", 0.1f64.powf(l))],
        "decay" => vec![e("gmult", 5f64.powf(l))], "blackout" => vec![e("mile", 0.1 * l)], "slow" => vec![e("speed", 1.05f64.powf(l))], "nomile" => vec![e("mile", 0.05 * l)], "fragile" => vec![e("fragGain", 2f64.powf(l))],
        "apex" => vec![e("gen9", 1000f64.powf(l))], "hunger" => vec![e("bhPow", 0.25 * l)], "entropy" => vec![e("syn", 0.002 * l)], _ => vec![] }
}
#[allow(dead_code)] pub fn unused_map() -> HashMap<u8, u8> { HashMap::new() }

#[cfg(test)]
mod sim {
    use super::*;
    /// a greedy player: taps twice a second, buys everything, prestiges when it pays. Checks the climb works end to end.
    #[test]
    fn greedy_climb() {
        let mut g = Game::new(S::default()); let dt = 0.5; let mut first_reb = None; let mut first_sing = None;
        for step in 0..(8 * 3600 * 2) {
            let t = step as f64 * dt;
            g.do_tap(); g.do_tap(); g.tick(dt, false);
            if step % 2 == 0 { g.buy_everything(); g.buy_all_upg(); g.shop_max("perk"); g.shop_max("su"); }
            if g.s.ev.cur.is_some() { g.claim_event() }
            for i in 0..3 { if g.s.pets.eggs.get(i).map(|e| e.t <= 0.0).unwrap_or(false) { g.hatch(i); } }
            if g.can_prestige("sing") && g.layer_gain("sing").gtef(1.0) && g.s.cnt.sing < 1.0 { g.prestige("sing", false); first_sing.get_or_insert(t); }
            else if g.can_prestige("reb") && g.layer_gain("reb").gte(g.s.frag.maxf(3.0)) { g.prestige("reb", false); first_reb.get_or_insert(t); }
            assert!(g.s.energy.m.is_finite() && g.c.eps.m.is_finite(), "NaN at {t}");
        }
        println!("first rebirth {:?} min, first singularity {:?} min, rebirths {}, energy {}", first_reb.map(|x| x / 60.0), first_sing.map(|x| x / 60.0), g.s.cnt.reb, crate::fmt::fmt(g.s.energy));
        assert!(first_reb.is_some());
    }
}
