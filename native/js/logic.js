// ===== INFINITY ENGINE — game logic (no DOM) =====
const SAVE_KEY = 'infinityEngine', SAVE_VERSION = 1;
let S, FX, C, fxDirty = true, achN = 0, hidN = 0, heat = 0; // heat: Overdrive meter from rapid manual taps (0–1)
const Q = []; // notifications for the UI
const notify = (type, text, extra) => Q.push({ type, text, extra });
const UMAP = Object.fromEntries(UPG.map(u => [u.id, u]));
const AMAP = Object.fromEntries(ARTS.map(a => [a.id, a]));
const lv = (m, id) => m[id] || 0;
const oom = x => D(x).add(1).log10();

function newDim() { return { u: 0, r: D(0), g: [0, 1, 2].map(() => ({ b: 0, a: D(0) })), amp: 0, reso: 0, link: 0 }; }
function newState() {
  const dims = {}; DIMS.forEach(d => (dims[d.id] = newDim()));
  return {
    v: SAVE_VERSION, energy: D(10), runE: D(0), lifeE: D(0), gens: Array(NG).fill(0), upg: {},
    frag: D(0), fragLife: D(0), perks: {},
    sing: D(0), singTotal: D(0), supg: {}, bh: D(0),
    cores: D(0), coresTotal: D(0), aupg: {}, dims, qr: { m: 1, t: 0, lock: 0, cd: 0 }, tskipCd: 0,
    ve: D(0), veTotal: D(0), vupg: {}, laws: [], lawCd: 0,
    ip: D(0), ipTotal: D(0), inf: {}, up: D(0), upTotal: D(0), uupg: {},
    rp: D(0), res: {}, arts: {},
    boss: { k: {}, cd: {}, f: null }, chal: { done: {}, a: null },
    ev: { next: 90, cur: null, buffs: [] }, ab: {},
    auto: { tap: 1, gens: Array(NG).fill(1), upg: 1, ab: 1, feed: 0, reb: 0, rebAt: '1', sing: 0, singAt: '1', asc: 0, ascAt: '1', tra: 0, traAt: '1', inf: 0, infAt: '1' },
    cnt: { reb: 0, sing: 0, asc: 0, tra: 0, inf: 0, ult: 0 },
    st: { taps: 0, maxEps: D(0), upg: 0, ev: 0, evT: {}, ab: 0, kills: 0, crits: 0, play: 0, run: 0, bestReb: 1e12, lastBuy: 0, glitch: 0, chal: 0, act: 0, dayN: 1, lastDay: '' },
    pets: { eggs: [], own: {}, active: null, bought: 0, hatched: 0 }, story: { ch: 0, seen: {}, claimed: 0, v: 3 }, badge: {},
    ach: {}, sec: {}, set: { not: 'suffix', hap: 1, skin: 'default', fx: 1, sfx: 1, mus: 1, sv: 0.7, mv: 0.5, tos: 0, badge: null, seenV: '', fileSync: 0, dskV: '', uiz: 'auto' },
    last: Date.now(), idle: 0, mode: 1, started: Date.now(),
  };
}

// ---------- effects ----------
const MUL = new Set(['cost', 'upgCost', 'evtFreq', 'evtDur', 'abCd', 'abDur', 'abPow', 'fragGain', 'singGain', 'ascGain', 'voidGain', 'ipGain', 'rp', 'speed', 'artDrop', 'achMult', 'autoInt', 'expo', 'ultGain']);
function ap(f, k, v, l = 1) {
  if (k.startsWith('gen')) { const i = +k.slice(3); f.gen[i] = f.gen[i].mul(D(v).pow(l)); return; }
  if (k.startsWith('ag')) { f.ag[+k.slice(2)] = 1; return; }
  if (k === 'ab') { f.ab[v] = 1; return; }
  if (k === 'dyn') { if (!f.dyn.includes(v)) f.dyn.push(v); return; }
  if (k === 'flag') { f.flag[v] = 1; return; }
  const cur = f[k];
  if (cur instanceof Decimal) f[k] = cur.mul(D(v).pow(l));
  else if (MUL.has(k)) f[k] = cur * Math.pow(v, l);
  else f[k] = cur + v * l;
}
const freeAuto = () => !!(S.aupg.auto || S.inf.a1);
const hasU = id => !!S.upg[id] || (UMAP[id].cat === 'auto' && freeAuto());
function fxBuild() {
  const f = {
    gmult: D(1), gen: GENS.map(() => D(1)), cost: 1, upgCost: 1, mile: 0, syn: 0, tapFlat: 0, tapMult: D(1), tapEps: 0, critC: 0.02, critM: 3,
    autotap: 0, ag: Array(NG).fill(0), autoInt: 1, autoUpg: 0, autoMax: 0, autoAb: 0, ab: {}, abCd: 1, abDur: 1, abPow: 1, evtFreq: 1, evtDur: 1,
    offlineH: 24, fragGain: 1, singGain: 1, ascGain: 1, voidGain: 1, ipGain: 1, rp: 1, bhPow: 1.5, bhPassive: 0, dimMult: D(1), speed: 1,
    bossDmg: D(1), artDrop: 1, achMult: 1, ultGain: 1, growthRed: 0, lawSlots: 2, lawPow: 1, expo: 1, dyn: [], flag: {},
  };
  const add = (e, l = 1) => e.forEach(([k, v]) => ap(f, k, v, l));
  AUPG.forEach(u => S.aupg[u.id] && add(u.e));
  INF.forEach(u => S.inf[u.id] && add(u.e));
  UUPG.forEach(u => S.uupg && S.uupg[u.id] && add(u.e));
  SUPG.forEach(u => S.supg[u.id] && add(u.e));
  VUPG.forEach(u => S.vupg[u.id] && add(u.e));
  RES.forEach(r => S.res[r.id] && add(r.e));
  ARTS.forEach(a => S.arts[a.id] && add(a.e, S.arts[a.id]));
  CHAL.forEach(c => S.chal.done[c.id] && add(c.e(S.chal.done[c.id])));
  UPG.forEach(u => hasU(u.id) && add(u.e));
  PETS.forEach(p => { const o = S.pets.own[p.id]; if (o) ap(f, p.b[0], p.b[1], S.pets.active === p.id ? o.lvl : o.lvl * 0.25); });
  if (S.story.claimed) f.gmult = f.gmult.mul(D(STORY_M).pow(S.story.claimed));
  const P = S.perks;
  f.tapMult = f.tapMult.mul(3 ** lv(P, 'tapM'));
  f.offlineH += 4 * lv(P, 'offl'); f.cost *= 0.9 ** lv(P, 'cheap'); f.rp *= 1.5 ** lv(P, 'rsch');
  f.fragGain *= 1.25 ** lv(P, 'fgain'); f.mile += 0.05 * lv(P, 'mile'); f.bossDmg = f.bossDmg.mul(2 ** lv(P, 'boss'));
  f.evtFreq *= 1.1 ** lv(P, 'evt'); f.critC += 0.02 * lv(P, 'critP'); f.abCd *= 0.9 ** lv(P, 'abCd');
  return f;
}
function fxCheck() { if (fxDirty || !FX) { FX = fxBuild(); fxDirty = false; } }
const dirty = () => { fxDirty = true; };

// ---------- helpers ----------
const inC = id => !!(S.chal.a && S.chal.a.id === id);
const law = id => S.laws.includes(id);
const lawB = v => Math.pow(v, FX.lawPow);
const lawD = v => (FX.flag.paradox ? Math.sqrt(v) : v);
const buffM = k => S.ev.buffs.reduce((m, b) => (b.k === k ? m * b.m : m), 1);
const reso = id => 1.25 ** S.dims[id].reso;
const dimOn = id => !!S.dims[id].u;
const dimX = (id, k) => (dimOn(id) ? 1 + oom(S.dims[id].r) * reso(id) / k : 1); // a world's '×(1+OoM/k)' effect
const offH = () => FX.offlineH + (dimOn('dream') ? Math.min(24, Math.floor(oom(S.dims.dream.r) * reso('dream'))) : 0);
const dimsUnlocked = () => DIMS.filter(d => S.dims[d.id].u).length;
const bhOn = () => S.cnt.sing > 0;
const achCount = () => achN;
const DYN = {
  logE: () => D(1 + oom(S.runE) / 10),
  gens: () => D(1 + S.gens.reduce((a, b) => a + b, 0) / 500),
  ach: () => D(1.02).pow(achN),
  time: () => D(Math.min(5, 1 + S.st.run / 1800)),
  taps: () => D(1 + oom(S.st.taps) / 2),
  ip: () => S.ip.add(1).pow(2),
  hidden: () => D(1.5).pow(hidN),
};
const MILES = [25, 50, 100, 150, 200, 250, 300];
function mileCount(n) {
  if (FX.flag.mobius) return Math.floor(n / 25);
  let c = 0; for (const t of MILES) if (n >= t) c++;
  if (n >= 400) c += Math.floor((n - 300) / 100);
  return c;
}
function nextMile(n) {
  if (FX.flag.mobius) return (Math.floor(n / 25) + 1) * 25;
  for (const t of MILES) if (n < t) return t;
  return (Math.floor(n / 100) + 1) * 100;
}
const stellarEff = () => (dimOn('stellar') ? 0.25 * Math.log10(1 + oom(S.dims.stellar.r)) * reso('stellar') : 0);
const milePow = () => 2 + FX.mile + stellarEff();
function bhPow() {
  let p = FX.bhPow;
  if (dimOn('void')) p += 0.01 * oom(S.dims.void.r) * reso('void');
  if (law('collapse')) p += 2 * FX.lawPow;
  return p;
}
function speed() {
  let s = FX.speed * buffM('speed');
  if (law('haste')) s *= lawB(2);
  if (inC('slow')) s *= 0.25;
  return s;
}
function bossDmg() { return FX.bossDmg; }

// ---------- production ----------
function multParts() {
  const P = [['Upgrades & bonuses', FX.gmult]];
  if (S.badge && badgeN()) P.push(['Badges', D(1 + 0.02 * badgeN())]);
  for (const d of FX.dyn) P.push([{ logE: 'Echo Chamber', gens: 'Fleet Harmony', ach: 'Hall of Records', time: 'Momentum', taps: 'Kinetic Memory', ip: 'Infinite Fuel', hidden: 'Glitched Fragment' }[d], DYN[d]()]);
  if (!inC('fragile')) P.push(['Fragments', S.frag.mul(0.05 + 0.02 * lv(S.perks, 'reso')).add(1)]);
  P.push(['Singularities', S.singTotal.add(1).pow(1.5)]);
  if (bhOn()) P.push(['Black Hole', D(1 + oom(S.bh)).pow(bhPow())]);
  if (S.upTotal && S.upTotal.gt(0)) P.push(['Ultra Points', S.upTotal.add(1).pow(4)]);
  P.push(['Ascension Cores', S.coresTotal.add(1).pow(1.5)], ['Void Essence', S.veTotal.add(1).pow(2)], ['Infinity Points', S.ipTotal.add(1).pow(2)]);
  P.push(['Achievements', D(1 + 0.02 * achN * FX.achMult)], ['Boss trophies', D(3).pow(Object.keys(S.boss.k).length)]);
  if (dimOn('material')) P.push(['Matter', S.dims.material.r.add(1).pow(0.05 * reso('material'))]);
  for (const [id, k] of DIM_PROD) if (dimOn(id)) P.push([DIMS.find(d => d.id === id).res, D(1 + oom(S.dims[id].r) * reso(id)).pow(k)]);
  P.push(['Active buffs', D(buffM('prod'))]);
  let lw = 1; if (law('entropy')) lw *= lawB(1e4); if (law('fortune')) lw *= lawD(0.1); if (law('expansion')) lw *= lawD(0.01);
  P.push(['Reality Laws', D(lw)]);
  if (inC('blackout')) P.push(['Blackout', D(0.01)]);
  if (inC('entropy')) P.push(['Entropy', D(Math.pow(0.5, (S.st.run - S.st.lastBuy) / 60))]);
  return P;
}
function globalMult() { return multParts().reduce((a, p) => a.mul(p[1]), D(1)); }
function calc() {
  const g = globalMult(), mp = milePow(), o = S.gens;
  let top = -1; for (let i = 0; i < NG; i++) if (o[i] > 0) top = i;
  const prod = []; let eps = D(0);
  for (let i = 0; i < NG; i++) {
    const n = o[i] + (FX.flag.hoi && i < NG - 1 ? o[i + 1] * 0.1 : 0);
    let p = D(0);
    if (n > 0) {
      p = D(GENS[i].p).mul(n).mul(FX.gen[i]).mul(g);
      if (!inC('nomile')) p = p.mul(D(mp).pow(mileCount(o[i])));
      if (i < NG - 1 && FX.syn) p = p.mul(1 + FX.syn * o[i + 1]);
      if (law('gravity')) p = p.mul(i < 5 ? lawB(1e6) : lawD(0.01));
      if (law('inversion')) { if (i === 9) p = p.mul(lawB(1e10)); if (i < 3) p = D(0); }
      if (inC('apex') && i !== top) p = D(0);
    }
    prod.push(p); eps = eps.add(p);
  }
  if (FX.expo > 1 && eps.gt(1)) {
    const e2 = eps.pow(FX.expo), k = e2.div(eps);
    for (let i = 0; i < NG; i++) prod[i] = prod[i].mul(k);
    eps = e2;
  }
  let tap = D(1 + FX.tapFlat).mul(FX.tapMult).mul(buffM('tap'));
  if (dimOn('plasma')) tap = tap.mul(S.dims.plasma.r.add(1).pow(0.1 * reso('plasma')));
  if (law('echoes')) tap = tap.mul(lawB(1e6));
  if (inC('blackout')) tap = tap.mul(100);
  const tapE = eps.mul(FX.tapEps).mul(buffM('tap'));
  C = { g, eps, prod, tap, tapE, sp: speed(), mp };
  return C;
}
function gain(v) {
  if (!v.gt(0)) return;
  S.energy = S.energy.add(v); S.runE = S.runE.add(v); S.lifeE = S.lifeE.add(v);
  if (!FX.flag.broken) {
    if (S.energy.gt(LIMIT)) S.energy = LIMIT;
    if (S.runE.gt(LIMIT)) S.runE = LIMIT;
  }
}
const capped = () => !FX.flag.broken && S.runE.gte(LIMIT);

// ---------- generators ----------
const genAllowed = i => !(inC('lone') && i > 1);
const genVisible = i => i === 0 || S.gens[i] > 0 || (S.gens[i - 1] > 0 && (!GENS[i].dim || dimOn(GENS[i].dim)));
function genBase(i) {
  let c = D(GENS[i].c).mul(FX.cost * buffM('cost'));
  if (dimOn('quantum')) c = c.div(S.dims.quantum.r.add(1).pow(0.02 * reso('quantum')));
  return c;
}
function genRatio(i) { let r = GENS[i].r - FX.growthRed; if (inC('inflation')) r += 0.1; return Math.max(1.03, r); }
const genCostN = (i, n) => Decimal.sumGeometricSeries(n, genBase(i), genRatio(i), S.gens[i]);
const genMaxN = i => Math.min(1e6, Decimal.affordGeometricSeries(S.energy, genBase(i), genRatio(i), S.gens[i]).toNumber());
function buyGen(i, mode) {
  if (!genAllowed(i) || !genVisible(i)) return 0;
  const n = mode === 'max' ? genMaxN(i) : mode;
  if (n < 1) return 0;
  const c = genCostN(i, n);
  if (c.gt(S.energy)) return 0;
  S.energy = S.energy.sub(c); S.gens[i] += n; S.st.lastBuy = S.st.run;
  return n;
}

// ---------- upgrades ----------
function condOk(c) {
  if (!c) return true;
  if (c.t === 'gen') return S.gens[c.i] >= c.n;
  if (c.t === 'reb') return S.cnt.reb >= c.n;
  if (c.t === 'taps') return S.st.taps >= c.n;
  return false;
}
function condText(c) {
  if (c.t === 'gen') return `Own ${c.n} ${GENS[c.i].n}`;
  if (c.t === 'reb') return `Rebirth ${c.n} time${c.n > 1 ? 's' : ''}`;
  if (c.t === 'taps') return `Tap the reactor ${c.n} times`;
}
function upgCost(u) { let c = u.cost.mul(FX.upgCost); if (law('entropy')) c = c.mul(lawD(1000)); return c; }
const upgAvail = u => !hasU(u.id) && u.req.every(hasU) && condOk(u.cond);
function buyUpg(id) {
  const u = UMAP[id];
  if (inC('noupg') || !upgAvail(u)) return false;
  const c = upgCost(u);
  if (c.gt(S.energy)) return false;
  S.energy = S.energy.sub(c); S.upg[id] = 1; S.st.upg++; dirty();
  return true;
}

// ---------- tapping & abilities ----------
function doTap() {
  if (inC('notap') || law('haste')) return null;
  heat = Math.min(1, heat + 0.04);
  let v = C.tap.add(C.tapE).mul(1 + heat); const crit = Math.random() < Math.min(1, FX.critC);
  if (crit) { v = v.mul(FX.critM); S.st.crits++; }
  gain(v); S.st.taps++; S.idle = 0;
  if (S.boss.f) S.boss.f.dmg = S.boss.f.dmg.add(v.mul(10).mul(bossDmg()));
  return { v, crit };
}
const abReady = id => !!FX.ab[id] && !(S.ab[id] > 0);
function useAb(id) {
  if (!abReady(id)) return false;
  S.ab[id] = ABIL[id].cd * FX.abCd; S.st.ab++;
  if (id === 'over') buff('prod', 10 * FX.abPow, 30 * FX.abDur, 'over');
  if (id === 'gold') buff('tap', 50 * FX.abPow, 20 * FX.abDur, 'gold');
  if (id === 'chrono') { const v = C.eps.mul(900 * FX.abPow * C.sp); gain(v); notify('gain', `Chrono Burst: +${fmt(v)} energy`); }
  if (id === 'lure') spawnEvent();
  return true;
}
function buff(k, m, t, src) { S.ev.buffs.push({ k, m, t, T: t, src }); }

// ---------- events ----------
const evtFreq = () => FX.evtFreq * (law('fortune') ? lawB(3) : 1);
function spawnEvent() {
  const ev = S.ev; ev.next = 60 + Math.random() * 120;
  const keys = Object.keys(EVENTS);
  const w = keys.map(k => EVENTS[k].w * (k === 'glitch' && FX.flag.glitch ? 3 : 1));
  let r = Math.random() * w.reduce((a, b) => a + b, 0), k = keys[0];
  for (let i = 0; i < keys.length; i++) { r -= w[i]; if (r <= 0) { k = keys[i]; break; } }
  ev.cur = { k, t: 15 };
}
function claimEvent() {
  const ev = S.ev; if (!ev.cur) return;
  const k = ev.cur.k, d = FX.evtDur; ev.cur = null;
  S.st.ev++; S.st.evT[k] = (S.st.evT[k] || 0) + 1;
  if (k === 'storm') buff('prod', 5, 30 * d, k);
  if (k === 'golden') buff('prod', 25, 15 * d, k);
  if (k === 'surge') buff('tap', 20, 30 * d, k);
  if (k === 'leak') buff('cost', 0.5, 30 * d, k);
  if (k === 'glitch') { buff('speed', FX.flag.glitch ? 9 : 3, 20 * d, k); S.st.glitch++; if (S.st.glitch >= 5 && !S.arts.glitched) giveArt('glitched'); }
  let msg = EVENTS[k].d;
  if (k === 'collapse') { const v = C.eps.mul((120 + Math.random() * 480) * C.sp); gain(v); msg = `+${fmt(v)} energy`; }
  notify('event', `${EVENTS[k].n}! ${msg}`, k);
  if (Math.random() < 0.03 * FX.artDrop * (law('fortune') ? 3 : 1)) rollArt(0);
  if (CREW_ON && Math.random() < 0.04) giveEgg();
}
function evTick(dt) {
  const ev = S.ev;
  if (ev.cur) { ev.cur.t -= dt; if (FX.flag.autoEvt) claimEvent(); else if (ev.cur.t <= 0) ev.cur = null; }
  else if (S.st.play > 20) { ev.next -= dt * evtFreq(); if (ev.next <= 0) spawnEvent(); }
}

// ---------- artifacts ----------
function rollArt(minR) {
  const pool = ARTS.filter(a => !a.secret && a.r >= minR);
  let r = Math.random() * pool.reduce((s, a) => s + RAR_W[a.r], 0), pick = pool[0];
  for (const a of pool) { r -= RAR_W[a.r]; if (r <= 0) { pick = a; break; } }
  return giveArt(pick.id);
}
function giveArt(id) {
  const a = AMAP[id], max = a.r === 4 ? 1 : 5, cur = S.arts[id] || 0;
  if (cur >= max) { const v = C.eps.mul(600); gain(v); notify('art', `${a.n} (duplicate) → +${fmt(v)} energy`, id); return; }
  S.arts[id] = cur + 1; dirty();
  notify('art', `${RAR[a.r]} artifact: ${a.n}${cur ? ' → Lv ' + (cur + 1) : ''}`, id);
}

// ---------- bosses ----------
const bossUnlocked = i => S.cnt.reb > 0 && (i === 0 || !!S.boss.k[i - 1]) && (i < 10 || !!FX.flag.broken);
const bossReady = i => bossUnlocked(i) && !(S.boss.cd[i] > 0) && !S.boss.f;
function startBoss(i) { if (!bossReady(i)) return false; S.boss.f = { i, t: 60, dmg: D(0) }; return true; }
function bossTick(dt) {
  for (const k in S.boss.cd) S.boss.cd[k] = Math.max(0, S.boss.cd[k] - dt);
  const f = S.boss.f; if (!f) return;
  f.dmg = f.dmg.add(C.eps.mul(bossDmg()).mul(dt)); f.t -= dt;
  const b = BOSSES[f.i];
  if (f.dmg.gte(b.hp)) {
    const first = !S.boss.k[f.i];
    S.boss.k[f.i] = (S.boss.k[f.i] || 0) + 1; S.st.kills++; S.boss.cd[f.i] = 600; S.boss.f = null;
    const v = C.eps.mul(120); gain(v);
    notify('boss', `${b.n} defeated! +${fmt(v)} energy${first ? ' · permanent ×3 production' : ''}`, f.i);
    if (first) { rollArt(Math.min(4, Math.floor(f.i / 2))); if (CREW_ON) giveEgg(Math.min(4, Math.floor(f.i / 2)), true); }
    else if (Math.random() < 0.5 * FX.artDrop) rollArt(Math.floor(f.i / 3));
    if (f.i === 9) S.sec.omega = 1;
  } else if (f.t <= 0) { S.boss.f = null; S.boss.cd[f.i] = 60; notify('bad', `${b.n} escaped. Try again in 60s.`); }
}

// ---------- prestige ----------
function startEnergy() { const l = lv(S.perks, 'head'); return l ? D(100).mul(D(10).pow(l)) : D(10); }
function resetLayer(L) {
  fxCheck();
  const F = FX.flag, keepAuto = lv(S.perks, 'keepAuto');
  if (L > 1 || !F.ouro) {
    const kept = {};
    if (L === 1) for (const id in S.upg) { const c = UMAP[id].cat; if ((keepAuto && (c === 'auto' || c === 'spec')) || (F.keepGenUpg && c === 'gen')) kept[id] = 1; }
    S.upg = kept;
  }
  if (L >= 2) {
    S.frag = D(0); S.bh = D(0);
    const keepP = L === 2 ? F.keepPerk || F.aKeepPerk || F.vKeepSing : L === 3 ? F.aKeepPerk || F.vKeepSing : L === 4 ? F.vKeepSing : false;
    if (!keepP) S.perks = {};
  }
  if (L >= 3) { if (!(L === 3 ? F.aKeepSU || F.vKeepSing : L === 4 ? F.vKeepSing : false)) S.supg = {}; }
  if (L >= 4) {
    if (!(L === 4 && F.vKeepAsc)) S.aupg = {};
    if (!(L === 4 && F.dimKeep)) DIMS.forEach(d => { const u = S.dims[d.id].u; S.dims[d.id] = newDim(); S.dims[d.id].u = u; if (u) S.dims[d.id].r = D(10); });
  }
  if (L >= 5) { S.ve = D(0); S.veTotal = D(0); S.vupg = {}; S.laws = []; if (!F.keepDims) DIMS.forEach(d => (S.dims[d.id] = newDim())); }
  dirty(); fxCheck();
  const F2 = FX.flag;
  if (L >= 3) { S.sing = D(F2.start100 ? 100 : F2.startSing ? 5 : 0); S.singTotal = S.sing; }
  if (L >= 4) { S.cores = D(F2.startCores ? 10 : 0); S.coresTotal = S.cores; }
  S.energy = startEnergy(); S.runE = D(0); S.gens = Array(NG).fill(0);
  if (F2.seed) for (let i = 0; i < 5; i++) S.gens[i] = 10;
  S.ev.buffs = []; S.st.run = 0; S.st.lastBuy = 0; S.boss.f = null;
  calc();
}
const rebReq = () => D(FX.flag.rebReq ? 1e7 : 1e10);
function celestialEff() { return dimOn('celestial') ? (1 + oom(S.dims.celestial.r) * reso('celestial')) ** 2 : 1; }
function rebGain() {
  if (S.runE.lt(rebReq())) return D(0);
  let fg = FX.fragGain * celestialEff(); if (law('collapse')) fg *= lawD(0.5);
  return S.runE.div(rebReq()).pow(0.33).mul(3 * fg).floor();
}
function singGain() { return S.runE.lt(1e30) ? D(0) : D((S.runE.log10() / 30) ** 3 * FX.singGain * celestialEff()).floor(); }
function ascGain() { return S.runE.lt(1e75) ? D(0) : D((S.runE.log10() / 75) ** 4 * FX.ascGain * dimX('fractal', 4)).floor(); }
function voidGain() { return S.runE.lt(1e125) || dimsUnlocked() < 4 ? D(0) : D((S.runE.log10() / 125) ** 5 * FX.voidGain * dimX('mirror', 5)).floor(); }
function ipGain() {
  if (S.runE.lt(LIMIT)) return D(0);
  let m = FX.ipGain * (FX.flag.ipAch ? 1 + achN / 10 : 1) * (dimOn('infinite') ? 1 + oom(S.dims.infinite.r) * reso('infinite') / 10 : 1);
  const base = FX.flag.broken ? D(10).pow(S.runE.log10() / 308 - 0.75) : D(1);
  // Ultra Points ever earned boost Infinity Point gain ×(UP+1)^0.75, so each Ultra cycle runs faster than the last
  const ultra = S.upTotal ? S.upTotal.add(1).pow(0.75) : D(1);
  return base.mul(m).mul(ultra).floor().max(1);
}
const LAYERS = [
  { id: 'reb', n: 'Rebirth', cur: 'Fragments', gain: rebGain, key: 'frag', tot: 'fragLife', L: 1 },
  { id: 'sing', n: 'Singularity', cur: 'Singularities', gain: singGain, key: 'sing', tot: 'singTotal', L: 2 },
  { id: 'asc', n: 'Ascension', cur: 'Ascension Cores', gain: ascGain, key: 'cores', tot: 'coresTotal', L: 3 },
  { id: 'tra', n: 'Transcendence', cur: 'Void Essence', gain: voidGain, key: 've', tot: 'veTotal', L: 4 },
  { id: 'inf', n: 'Infinity', cur: 'Infinity Points', gain: ipGain, key: 'ip', tot: 'ipTotal', L: 5 },
  { id: 'ult', n: 'Ultra Prestige', cur: 'Ultra Points', gain: ultGain, key: 'up', tot: 'upTotal', L: 6 },
];
const LREQ = { reb: () => `${fmt(rebReq())} energy this run`, sing: () => '1e30 energy this run', asc: () => '1e75 energy this run', tra: () => '1e125 energy this run & 4 dimensions', inf: () => '1.79e308 energy (Infinity)', ult: () => `${fmt(D(ULT_IP))} Infinity Points since your last Ultra` };
// Ultra Points: earn ULT_IP Infinity Points since your last Ultra; points grow as a power of how far past that you got
function ultGain() {
  if (!S.ipTotal || S.ipTotal.lt(ULT_IP)) return D(0);
  return S.ipTotal.div(ULT_IP).pow(ULT_POW).mul(3).mul(FX.ultGain).floor();
}
function buyUU(id) {
  const u = UUPG.find(x => x.id === id); if (!u || S.uupg[id] || S.up.lt(u.c)) return false;
  S.up = S.up.sub(u.c); S.uupg[id] = 1; dirty();
  if (id === 'omega' && !S.st.beat) { S.st.beat = Date.now(); notify('beat', 'You beat Ten Cells!', 0); }
  return true;
}
// the full wipe: only meta progress survives (achievements, badges, secrets, settings, pets, story, lifetime stats, Ultra)
function ultraReset() {
  const F = FX.flag, o = S, n = newState();
  for (const k of ['ach', 'sec', 'set', 'badge', 'pets', 'story', 'st', 'started', 'up', 'upTotal', 'uupg', 'auto', 'lifeE', 'v']) n[k] = o[k];
  n.cnt.ult = o.cnt.ult;
  if (F.uKeepRes) n.res = o.res;
  if (F.uKeepArts) n.arts = o.arts;
  if (F.uKeepBoss) n.boss.k = o.boss.k;
  if (F.uKeepChal) n.chal.done = o.chal.done;
  n.st.run = 0; n.st.bestReb = o.st.bestReb;
  n.last = Date.now(); n.ev = o.ev; n.ev.buffs = []; n.ev.cur = null;
  S = n; dirty(); fxCheck(); calc();
}
function canPrestige(id) { return !S.chal.a && LAYERS.find(l => l.id === id).gain().gte(1); }
// automation prestiges get flagged so the UI shows a toast instead of the full cinematic
function autoPrestige(id) { const n = Q.length, r = prestige(id); for (let k = n; k < Q.length; k++) if (Q[k].type === 'prestige') Q[k].auto = 1; return r; }
function prestige(id) {
  const ly = LAYERS.find(l => l.id === id);
  if (!canPrestige(id)) return null;
  const g = ly.gain(), t = S.st.run;
  S[ly.key] = S[ly.key].add(g); S[ly.tot] = S[ly.tot].add(g); S.cnt[id]++;
  if (id === 'reb') S.st.bestReb = Math.min(S.st.bestReb, t);
  if (ly.L === 6) ultraReset(); else resetLayer(ly.L);
  notify('prestige', `${ly.n}! +${fmt(g)} ${ly.cur}`, id);
  return g;
}

// ---------- shops ----------
function buyPerk(id) {
  const p = PERKS.find(x => x.id === id), l = lv(S.perks, id);
  if (l >= p.max) return false;
  const c = D(Math.ceil(p.c(l)));
  if (S.frag.lt(c)) return false;
  S.frag = S.frag.sub(c); S.perks[id] = l + 1; dirty(); return true;
}
function buyFrom(list, owned, cur, id) {
  const u = list.find(x => x.id === id);
  if (owned[id] || S[cur].lt(u.c)) return false;
  S[cur] = S[cur].sub(u.c); owned[id] = 1; dirty(); return true;
}
const buySU = id => buyFrom(SUPG, S.supg, 'sing', id);
const buyAU = id => buyFrom(AUPG, S.aupg, 'cores', id);
const buyVU = id => buyFrom(VUPG, S.vupg, 've', id);
const infAvail = n => !S.inf[n.id] && n.req.every(r => S.inf[r]);
function buyInf(id) {
  const n = INF.find(x => x.id === id);
  if (!infAvail(n) || S.ip.lt(n.c)) return false;
  S.ip = S.ip.sub(n.c); S.inf[id] = 1; dirty(); return true;
}
function feedBH(pct) {
  if (!bhOn()) return false;
  if (S.energy.lt(1)) S.sec.feedNothing = 1;
  const x = S.energy.mul(pct); S.energy = S.energy.sub(x); S.bh = S.bh.add(x); return true;
}

// ---------- research ----------
const tierOpen = t => [S.cnt.reb, S.cnt.sing, S.cnt.asc, S.cnt.tra][t] > 0;
function rpRate() { let r = FX.rp; if (dimOn('temporal')) r *= 1 + oom(S.dims.temporal.r) * reso('temporal'); return r; }
const resReq = i => (i % 4 ? 'x' + (i - 1) : null);
function resAvail(r, i) { return !S.res[r.id] && tierOpen(r.tier) && (!resReq(i) || S.res[resReq(i)]); }
function buyRes(id) {
  const i = RES.findIndex(r => r.id === id), r = RES[i];
  if (!resAvail(r, i) || S.rp.lt(r.c)) return false;
  S.rp = S.rp.sub(r.c); S.res[id] = 1; dirty(); return true;
}

// ---------- challenges ----------
const chalOpen = () => S.cnt.sing > 0;
function startChal(id, l) {
  if (S.chal.a || !chalOpen() || l > 3 || (S.chal.done[id] || 0) < l - 1) return false;
  resetLayer(1); S.chal.a = { id, l }; return true;
}
function exitChal() { if (!S.chal.a) return; S.chal.a = null; resetLayer(1); }
function completeChal() {
  const { id, l } = S.chal.a, fast = S.st.run < 60;
  S.chal.done[id] = Math.max(S.chal.done[id] || 0, l); S.st.chal++; S.chal.a = null;
  if (fast) S.sec.speedrun = 1;
  dirty(); resetLayer(1);
  notify('chal', `Challenge complete: ${CHAL.find(c => c.id === id).n} ${'I'.repeat(l)}`);
}

// ---------- dimensions ----------
function dimUnlockOk(d) {
  if (S.dims[d.id].u || S.cnt.asc < 1) return false;
  if (d.tra && S.cnt.tra < d.tra) return false;
  if (d.cond) { const p = S.dims[d.cond[0]]; if (!p.u || p.r.lt(d.cond[1])) return false; }
  return true;
}
function unlockDim(id) {
  const d = DIMS.find(x => x.id === id);
  if (!dimUnlockOk(d) || S.cores.lt(d.cost)) return false;
  S.cores = S.cores.sub(d.cost); S.dims[id].u = 1; S.dims[id].r = D(10); return true;
}
function dimMult(id) {
  const dd = S.dims[id], i = DIMS.findIndex(x => x.id === id);
  let m = FX.dimMult.mul(D(2).pow(dd.amp));
  if (dd.link) m = m.mul(1 + oom(i ? S.dims[DIMS[i - 1].id].r : S.energy));
  if (dimOn('infinite')) m = m.mul(1 + oom(S.dims.infinite.r) * reso('infinite'));
  if (law('expansion')) m = m.mul(lawB(100));
  if (id === 'plasma') m = m.mul(5);
  if (id === 'quantum') m = m.mul(S.qr.m);
  if (id === 'stellar') m = m.mul(1 + oom(S.energy) / 20);
  if (id === 'void') m = m.mul(Math.min(10, 1 + S.idle / 30));
  if (id === 'celestial') m = m.mul(celestialPhase());
  if (id === 'fractal') m = m.mul(D(1.5).pow(Math.max(0, dimsUnlocked() - 1)));
  if (id === 'mirror') m = m.mul(S.idle < 10 ? 3 : 0.5);
  if (id === 'dream') m = m.mul(Math.min(20, 1 + S.st.run / 190));
  if (id === 'origin') m = m.mul(originFlare() ? 10 : 1);
  return m;
}
const DIM_PROD = [['fractal', 2], ['mirror', 2.5], ['dream', 3], ['origin', 4]]; // new worlds: all production ×(1+OoM of their resource)^k
const originFlare = () => S.st.play % 60 < 10;
const celestialPhase = () => 1.6 + 1.4 * Math.sin(S.st.play * 2 * Math.PI / 120);
const dimEff = (dd, k) => D(dd.g[k].b).add(dd.g[k].a).mul(D(2).pow(Math.floor(dd.g[k].b / 10)));
function dimRate(id) { const dd = S.dims[id]; return dimEff(dd, 0).mul(DGEN[0].p).mul(dimMult(id)); }
function dimTick(dt) {
  const q = S.qr; q.t -= dt; q.lock = Math.max(0, q.lock - dt); q.cd = Math.max(0, q.cd - dt);
  if (q.t <= 0) { q.t = 5; if (q.lock <= 0) q.m = FX.flag.paradox ? 1 + Math.random() * 3 : Math.random() * 4; }
  S.tskipCd = Math.max(0, S.tskipCd - dt);
  for (const d of DIMS) {
    const dd = S.dims[d.id]; if (!dd.u) continue;
    const m = dimMult(d.id).mul(dt), g = dd.g;
    g[1].a = g[1].a.add(dimEff(dd, 2).mul(DGEN[2].p * dt));
    g[0].a = g[0].a.add(dimEff(dd, 1).mul(DGEN[1].p * dt));
    if (d.id === 'plasma') {
      // Heat decays 2%/s. Solve dr/dt = P - k·r exactly so big steps (offline progress) give the same result as small ones
      const k = -Math.log(0.98), e = Math.exp(-k * dt), P = dimEff(dd, 0).mul(DGEN[0].p).mul(dimMult(d.id));
      dd.r = dd.r.mul(e).add(P.mul((1 - e) / k));
    } else dd.r = dd.r.add(dimEff(dd, 0).mul(DGEN[0].p).mul(m));
  }
}
const dimGenCost = (dd, k) => D(DGEN[k].c).mul(D(DGEN[k].r).pow(dd.g[k].b));
function buyDimGen(id, k, max) {
  const dd = S.dims[id]; if (!dd.u) return 0; let n = 0;
  do { const c = dimGenCost(dd, k); if (dd.r.lt(c)) break; dd.r = dd.r.sub(c); dd.g[k].b++; n++; } while (max && n < 1000);
  return n;
}
const DUPG = {
  amp: { n: 'Amplify', d: 'Dimension production ×2', c: dd => D(100).mul(D(20).pow(dd.amp)), max: Infinity },
  reso: { n: 'Resonance', d: 'Dimension effect strength ×1.25', c: dd => D(1e4).mul(D(1e3).pow(dd.reso)), max: 4 },
  link: { n: 'Link', d: 'Production ×(1 + OoM of previous world\'s resource)', c: () => D(1e6), max: 1 },
};
function buyDimUpg(id, k) {
  const dd = S.dims[id], u = DUPG[k];
  if (!dd.u || dd[k] >= u.max) return false;
  const c = u.c(dd); if (dd.r.lt(c)) return false;
  dd.r = dd.r.sub(c); dd[k]++; return true;
}
function observe() { if (!dimOn('quantum') || S.qr.cd > 0) return false; S.qr.lock = 60; S.qr.cd = 120; return true; }
function timeSkip() {
  const dd = S.dims.temporal; if (!dd.u || S.tskipCd > 0 || dd.r.lt(10)) return false;
  const spent = dd.r.div(2); dd.r = dd.r.sub(spent);
  const secs = Math.min(3600, oom(spent) * 300), v = C.eps.mul(secs * C.sp);
  gain(v); S.tskipCd = 30; notify('gain', `Time Skip: ${Math.round(secs / 60)} min → +${fmt(v)} energy`); return true;
}

// ---------- reality laws ----------
const lawsOpen = () => S.cnt.tra > 0;
function toggleLaw(id) {
  if (!lawsOpen() || S.lawCd > 0) return false;
  if (law(id)) S.laws = S.laws.filter(x => x !== id);
  else if (S.laws.length < FX.lawSlots) S.laws.push(id);
  else return false;
  S.lawCd = 10; dirty(); return true;
}

// ---------- automation ----------
const parseNum = s => { try { const d = D(String(s).trim() || '1'); return isFinite(d.mantissa) ? d : D(1); } catch (e) { return D(1); } };
function automate() {
  if (!law('echoes')) for (let i = NG - 1; i >= 0; i--) if (FX.ag[i] && S.auto.gens[i]) buyGen(i, FX.autoMax ? 'max' : 1);
  if (FX.autoUpg && S.auto.upg && !inC('noupg')) {
    const av = UPG.filter(upgAvail).sort((a, b) => upgCost(a).cmp(upgCost(b)));
    for (const u of av.slice(0, 5)) if (!buyUpg(u.id)) break;
  }
  if (FX.autoAb && S.auto.ab) for (const id in FX.ab) if (abReady(id)) useAb(id);
  const a = S.auto, F = FX.flag;
  if (F.autoInf && a.inf && canPrestige('inf') && ipGain().gte(parseNum(a.infAt))) return autoPrestige('inf');
  if (F.autoTra && a.tra && canPrestige('tra') && voidGain().gte(parseNum(a.traAt))) return autoPrestige('tra');
  if (F.autoAsc && a.asc && canPrestige('asc') && ascGain().gte(parseNum(a.ascAt))) return autoPrestige('asc');
  if (F.autoSing && a.sing && canPrestige('sing') && singGain().gte(parseNum(a.singAt))) return autoPrestige('sing');
  if (F.autoReb && a.reb && canPrestige('reb') && rebGain().gte(parseNum(a.rebAt))) return autoPrestige('reb');
}

// ---------- main tick ----------
let autoT = 0, achT = 0;
function tick(dt, off) {
  fxCheck(); calc();
  S.st.play += dt; S.st.run += dt; S.idle += dt;
  // story pacing: time with the game actually open, and the number of different calendar days you played
  if (!off) { S.st.act += dt; const day = new Date().toDateString(); if (S.st.lastDay !== day) { if (S.st.lastDay) S.st.dayN++; S.st.lastDay = day; } } heat *= Math.pow(0.75, dt);
  const sdt = dt * C.sp;
  gain(C.eps.mul(sdt));
  if (FX.autotap && S.auto.tap && !inC('notap')) gain(C.tap.mul(1 + Math.min(1, FX.critC) * (FX.critM - 1)).mul(FX.autotap * dt));
  if (inC('decay')) S.energy = S.energy.mul(Math.pow(0.98, sdt));
  if (inC('hunger')) { const x = S.energy.mul(1 - Math.pow(0.95, sdt)); S.energy = S.energy.sub(x); S.bh = S.bh.add(x); }
  if (bhOn()) {
    if (FX.bhPassive) S.bh = S.bh.add(C.eps.mul(FX.bhPassive * sdt));
    if (S.auto.feed > 0) { const x = S.energy.mul(Math.min(1, S.auto.feed / 100 * dt)); S.energy = S.energy.sub(x); S.bh = S.bh.add(x); }
  }
  dimTick(sdt);
  if (S.cnt.reb > 0) S.rp = S.rp.add(rpRate() * dt);
  if (!off) evTick(dt);
  S.ev.buffs.forEach(b => (b.t -= dt)); S.ev.buffs = S.ev.buffs.filter(b => b.t > 0);
  for (const k in S.ab) S.ab[k] = Math.max(0, S.ab[k] - dt);
  bossTick(sdt);
  S.lawCd = Math.max(0, S.lawCd - dt);
  for (const e of S.pets.eggs) e.t = Math.max(0, e.t - dt);
  if (C.eps.gt(S.st.maxEps)) S.st.maxEps = C.eps;
  if (S.chal.a && S.runE.gte(CHAL_GOAL[S.chal.a.l - 1])) completeChal();
  autoT += dt; if (autoT >= FX.autoInt) { autoT = 0; automate(); }
  achT += dt; if (achT >= 1) { achT = 0; checkAch(); checkBadges(); }
}

// ---------- pets ----------
const PMAP = Object.fromEntries(PETS.map(p => [p.id, p]));
const petStage = lvl => (lvl >= 20 ? 2 : lvl >= 10 ? 1 : 0);
const eggCost = () => Decimal.max(1e4, C.eps.mul(120)).mul(D(4).pow(S.pets.bought));
const feedCost = id => Decimal.max(1e3, C.eps.mul(60)).mul(D(1.35).pow(S.pets.own[id].lvl));
function rollRar(minR = 0) { const w = EGG_W.map((x, i) => (i >= minR ? x : 0)); let r = Math.random() * w.reduce((a, b) => a + b, 0); for (let i = 0; i < w.length; i++) { r -= w[i]; if (r <= 0) return i; } return minR; }
const CREW_ON = true;
function giveEgg(minR = 0, force) {
  if (!force && S.pets.eggs.length >= 3) return false;
  const r = rollRar(minR); S.pets.eggs.push({ r, t: EGG_TIME[r] }); notify('egg', `${RAR[r]} egg found! It will hatch in ${fmtT(EGG_TIME[r])}.`, r); return true;
}
function buyEgg() { if (S.pets.eggs.length >= 3) return false; const c = eggCost(); if (S.energy.lt(c)) return false; S.energy = S.energy.sub(c); S.pets.bought++; return giveEgg(); }
function hatch(i) {
  const e = S.pets.eggs[i]; if (!e || e.t > 0) return null;
  S.pets.eggs.splice(i, 1);
  let pool = PETS.filter(p => p.r === e.r); for (let r = e.r; !pool.length && r >= 0; r--) pool = PETS.filter(p => p.r === r);
  const p = pool[Math.floor(Math.random() * pool.length)], o = S.pets.own[p.id];
  if (o) { o.lvl = Math.min(30, o.lvl + 3); notify('pet', `${p.n} again! Your ${p.n} gains 3 levels.`, p.id); }
  else { S.pets.own[p.id] = { lvl: 1 }; notify('pet', `${p.n} hatched!`, p.id); }
  if (!S.pets.active) S.pets.active = p.id;
  S.pets.hatched++; dirty(); return p.id;
}
function feedPet(id) {
  const o = S.pets.own[id]; if (!o || o.lvl >= 30) return false;
  const c = feedCost(id); if (S.energy.lt(c)) return false;
  const st0 = petStage(o.lvl); S.energy = S.energy.sub(c); o.lvl++; dirty();
  if (petStage(o.lvl) > st0) notify('pet', `${PMAP[id].n} evolved: ${PET_STAGE[petStage(o.lvl)]}!`, id);
  return true;
}
function setActivePet(id) { if (!S.pets.own[id]) return false; S.pets.active = id; dirty(); return true; }
// ---------- campaign ----------
const chapter = () => CHAPTERS[Math.min(S.story.ch, CHAPTERS.length - 1)];
const storyDone = () => S.story.ch >= CHAPTERS.length;
const missionsDone = ch => CHAPTERS[ch].m.every(([, f]) => { const [a, b] = f(); return a >= b; });
function claimChapter() {
  if (storyDone() || !missionsDone(S.story.ch)) return false;
  const ch = S.story.ch; S.story.claimed++; S.story.ch++; dirty();
  giveEgg(Math.min(4, Math.floor(ch / 8)), true);
  notify('story', `Chapter complete: ${CHAPTERS[ch].n}. Production ×${STORY_M} forever.`, ch); return true;
}

// ---------- achievements ----------
const ACH = [];
const A = (id, n, d, f, h) => ACH.push({ id, n, d, f, h: !!h });
['Spark', 'Glow', 'Current', 'Surge', 'Torrent', 'Deluge', 'Overflow', 'Cascade', 'Flood', 'Tsunami', 'Maelstrom', 'Supernova', 'Hypernova', 'Quasar', 'Big Bang', 'Omniverse', 'Absolute', 'Infinity']
  .forEach((n, i) => { const x = [3, 6, 9, 12, 15, 20, 25, 30, 40, 50, 60, 75, 100, 125, 150, 200, 250, 308][i], t = D('1e' + x); A('e' + x, n, `Produce 1e${x} energy in total`, () => S.lifeE.gte(t)); });
GENS.forEach((g, i) => [1, 25, 100, 250, 500].forEach((n, k) =>
  A(`g${i}_${n}`, `${['First Light', 'Collector', 'Centurion', 'Magnate', 'Overlord'][k]}: ${g.n}`, `Own ${n} ${g.n}`, () => S.gens[i] >= n)));
[[100, 'Poke'], [1e3, 'Drummer'], [1e4, 'Hammer'], [1e5, 'Piston'], [1e6, 'Jackhammer']].forEach(([n, nm]) => A('tap' + n, nm, `Tap the reactor ${fmt(D(n))} times`, () => S.st.taps >= n));
[[10, 'Tinkerer'], [25, 'Engineer'], [50, 'Architect'], [100, 'Inventor'], [250, 'Visionary'], [500, 'Polymath']].forEach(([n, nm]) => A('up' + n, nm, `Buy ${n} upgrades (lifetime)`, () => S.st.upg >= n));
[[1, 'Beyond Infinity'], [3, 'Again, Bigger'], [10, 'Ultra Veteran'], [25, 'Ultra Legend']].forEach(([n, nm]) => A('ult' + n, nm, `Ultra Prestige ${n} time${n > 1 ? 's' : ''}`, () => (S.cnt.ult || 0) >= n));
A('omega', 'The Omega Engine', 'Beat the game', () => !!S.st.beat);
A('upall', 'Complete Blueprint', 'Own all 105 upgrades at once', () => UPG.every(u => hasU(u.id)));
[[1, 'Reborn'], [5, 'Cycle'], [10, 'Wheel of Time'], [25, 'Samsara'], [100, 'Eternal Return']].forEach(([n, nm]) => A('reb' + n, nm, `Rebirth ${n} time${n > 1 ? 's' : ''}`, () => S.cnt.reb >= n));
[[10, 'Shards'], [1e3, 'Mosaic'], [1e6, 'Crystal Cathedral'], [1e12, 'Kaleidoscope']].forEach(([n, nm]) => A('fr' + n, nm, `Earn ${fmt(D(n))} fragments (lifetime)`, () => S.fragLife.gte(n)));
[[1, 'Collapse'], [5, 'Gravity Well'], [25, 'Event Horizon'], [100, 'Spaghettified']].forEach(([n, nm]) => A('sing' + n, nm, `Reach Singularity ${n} time${n > 1 ? 's' : ''}`, () => S.cnt.sing >= n));
[[1e10, 'Hungry Hole'], [1e30, 'Supermassive'], [1e60, 'Ultramassive']].forEach(([n, nm]) => A('bh' + n, nm, `Black Hole mass ${fmt(D(n))}`, () => S.bh.gte(n)));
[[1, 'Ascendant'], [5, 'Exalted'], [25, 'Seraphic']].forEach(([n, nm]) => A('asc' + n, nm, `Ascend ${n} time${n > 1 ? 's' : ''}`, () => S.cnt.asc >= n));
DIMS.forEach((d, i) => A('dim' + i, `Explorer: ${d.n}`, `Unlock the ${d.n} dimension`, () => S.dims[d.id].u));
[[1, 'Beyond'], [5, 'Untethered'], [25, 'Formless']].forEach(([n, nm]) => A('tra' + n, nm, `Transcend ${n} time${n > 1 ? 's' : ''}`, () => S.cnt.tra >= n));
A('laws3', 'Lawmaker', 'Have 3 Reality Laws active at once', () => S.laws.length >= 3);
[[1, 'Infinite'], [5, 'Recurring'], [25, 'Unbounded'], [100, 'Ouroboros']].forEach(([n, nm]) => A('inf' + n, nm, `Reach Infinity ${n} time${n > 1 ? 's' : ''}`, () => S.cnt.inf >= n));
CHAL.forEach(c => A('ch_' + c.id, `Overcame: ${c.n}`, `Complete ${c.n} I`, () => (S.chal.done[c.id] || 0) >= 1));
A('chall3', 'Perfectionist', 'Complete every challenge at level III', () => CHAL.every(c => S.chal.done[c.id] >= 3));
[[1, 'Finder'], [5, 'Curator'], [10, 'Hoarder'], [20, 'Museum'], [25, 'Complete Collection']].forEach(([n, nm]) => A('art' + n, nm, `Collect ${n} different artifacts`, () => Object.keys(S.arts).length >= n));
A('artmyth', 'Mythbreaker', 'Find a Mythic artifact', () => ARTS.some(a => a.r === 4 && S.arts[a.id]));
A('artmax', 'Polished Relic', 'Raise an artifact to level 5', () => Object.values(S.arts).some(l => l >= 5));
BOSSES.forEach(b => A('boss' + b.i, `Slayer: ${b.n}`, `Defeat ${b.n}`, () => !!S.boss.k[b.i]));
[[1, 'Student'], [10, 'Scholar'], [20, 'Professor'], [32, 'Omniscient']].forEach(([n, nm]) => A('res' + n, nm, `Complete ${n} research`, () => Object.keys(S.res).length >= n));
[[1, 'Lucky Break'], [10, 'Storm Chaser'], [50, 'Weather Wizard'], [200, 'Chaos Magnet']].forEach(([n, nm]) => A('ev' + n, nm, `Claim ${n} random event${n > 1 ? 's' : ''}`, () => S.st.ev >= n));
Object.keys(EVENTS).forEach(k => A('evt_' + k, `Witness: ${EVENTS[k].n}`, `Claim a ${EVENTS[k].n}`, () => (S.st.evT[k] || 0) > 0));
[[1e3, 'Hum'], [1e10, 'Roar'], [1e50, 'Thunder'], [1e100, 'Cosmic Choir']].forEach(([n, nm]) => A('eps' + n, nm, `Reach ${fmt(D(n))} energy per second`, () => S.st.maxEps.gte(n)));
[[10, 'Activated'], [100, 'Power User']].forEach(([n, nm]) => A('ab' + n, nm, `Use abilities ${n} times`, () => S.st.ab >= n));
[['fidget', 'Fidget Spinner', 'Tap 15 times in 2 seconds'], ['owl', 'Night Owl', 'Play between 3 and 4 AM'], ['nap', 'Long Nap', 'Return after 12+ hours away'],
 ['title', 'Curious Mind', 'Find the secret in the title'], ['export', 'Archivist', 'Export your save'], ['import', 'Time Traveller', 'Import a save'],
 ['notation', 'Notation Nerd', 'Change the number notation'], ['feedNothing', 'Empty Offering', 'Feed the Black Hole with no energy'],
 ['speedrun', 'Speedrunner', 'Complete a challenge in under 60 seconds'], ['glitch5', 'Glitch in the Matrix', 'Claim 5 Reality Glitches']]
  .forEach(([id, n, d]) => A('h_' + id, n, d, id === 'glitch5' ? () => S.st.glitch >= 5 : () => !!S.sec[id], true));

A('pet1', 'New Friend', 'Hatch your first pet', () => S.pets.hatched >= 1);
A('pet4', 'Menagerie', 'Own 4 different pets', () => Object.keys(S.pets.own).length >= 4);
A('pet8', 'Full Crew', 'Own all 8 pets', () => Object.keys(S.pets.own).length >= 8);
A('petmax', 'Best Friend', 'Raise a pet to level 30', () => Object.values(S.pets.own).some(p => p.lvl >= 30));
A('ch1', 'The Story Begins', 'Complete chapter 1', () => S.story.claimed >= 1);
A('ch5', 'Halfway Up the Ladder', 'Complete chapter 5', () => S.story.claimed >= 5);
A('ch10', 'The Record Continues', 'Complete chapter 10', () => S.story.claimed >= 10);
A('ch20', 'Deep in the Record', 'Complete chapter 20', () => S.story.claimed >= 20);
A('ch30', 'Margins', 'Complete chapter 30', () => S.story.claimed >= 30);
A('chAll', 'The First Page', 'Complete the whole campaign', () => S.story.claimed >= CHAPTERS.length);
// badges: highest tier reached per track, kept forever
const badgeN = () => Object.values(S.badge).reduce((a, b) => a + b, 0);
function checkBadges() {
  for (const b of BADGES) {
    let v = 0; try { v = +b.val() || 0; } catch (e) { }
    const tier = b.th.filter(x => v >= x).length, had = S.badge[b.id] || 0;
    if (tier > had) { S.badge[b.id] = tier; notify('badge', `Badge: ${b.n} ${BTIER[tier - 1][0]}`, b.id + ':' + tier); }
  }
}
function checkAch() {
  if (new Date().getHours() === 3) S.sec.owl = 1;
  let n = 0, h = 0;
  for (const a of ACH) {
    if (!S.ach[a.id] && a.f()) { S.ach[a.id] = 1; notify('ach', a.n, a.id); dirty(); }
    if (S.ach[a.id]) { n++; if (a.h) h++; }
  }
  achN = n; hidN = h;
}

// ---------- skins / secrets ----------
const SKINS = [
  { id: 'default', n: 'Nebula', ok: () => true },
  { id: 'solar', n: 'Solar', ok: () => !!S.sec.title },
  { id: 'void', n: 'Void', ok: () => S.cnt.tra > 0 },
  { id: 'glitch', n: 'Glitch', ok: () => !!S.arts.glitched },
  { id: 'omega', n: 'Omega', ok: () => !!S.sec.omega },
];

// ---------- save / load ----------
function hash(s) { let h = 7; for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) | 0; return h; }
function packSave() { S.last = Date.now(); const d = JSON.stringify(S); return JSON.stringify({ h: hash(d), v: SAVE_VERSION, d }); }
function save() {
  try {
    const blob = packSave(), prev = localStorage.getItem(SAVE_KEY);
    if (prev) { try { parseSave(prev); localStorage.setItem(SAVE_KEY + '_bak', prev); } catch (e) { } }
    localStorage.setItem(SAVE_KEY, blob); return true;
  } catch (e) { return false; }
}
function migrate(d) {
  if (!d || typeof d !== 'object') throw new Error('bad save');
  d.v = d.v || 1; // future versions transform older data here
  return d;
}
function parseSave(str) {
  const o = JSON.parse(str);
  if (!o || typeof o.d !== 'string' || hash(o.d) !== o.h) throw new Error('checksum mismatch');
  return migrate(JSON.parse(o.d));
}
function merge(t, d) {
  if (d === undefined || d === null) return t;
  if (t instanceof Decimal) { try { const x = new Decimal(d); return isFinite(x.mantissa) && isFinite(x.exponent) && x.gte(0) ? x : t; } catch (e) { return t; } }
  if (t === null) return d;
  if (Array.isArray(t)) { if (!Array.isArray(d)) return t; return t.length ? t.map((v, i) => merge(v, d[i])) : d.slice(); }
  if (typeof t === 'object') {
    if (typeof d !== 'object' || Array.isArray(d)) return t;
    const o = {}; for (const k in t) o[k] = merge(t[k], d[k]);
    for (const k in d) if (!(k in t)) o[k] = d[k];
    return o;
  }
  if (typeof t === 'number') return typeof d === 'number' && isFinite(d) ? d : t;
  return typeof d === typeof t ? d : t;
}
// 0.10 grew the story from 12 to 38 chapters: move old saves to the same chapter in the new list and credit the ones slotted in before it
function storyV1(st) {
  const at = i => (i >= CH_V1.length ? CHAPTERS.findIndex(c => c.n === CH_V1[CH_V1.length - 1]) + 1 : CHAPTERS.findIndex(c => c.n === CH_V1[i]));
  const ch = at(Math.max(0, Math.min(CH_V1.length, +st.ch || 0))), seen = {};
  for (const k in st.seen || {}) { const i = parseInt(k, 10); if (i >= 0 && i < CH_V1.length) seen[at(i) + k.slice(String(i).length)] = 1; }
  st.ch = ch; st.claimed = ch; st.seen = seen; st.v = 3;
}
// 0.16 put four chapters after "After the Wall": saves already past them move forward and get them credited
function storyV2(st) {
  const at = CH16_AT, n = CH16.length, seen = {};
  for (const k in st.seen || {}) { const i = parseInt(k, 10); if (i >= 0) seen[(i >= at ? i + n : i) + k.slice(String(i).length)] = 1; }
  if ((+st.ch || 0) > at) { st.ch = +st.ch + n; st.claimed = (+st.claimed || 0) + n; st.added = n; }
  st.seen = seen; st.v = 3;
}
function hydrate(data) {
  if (data && data.story && !data.story.v) storyV1(data.story);
  if (data && data.story && data.story.v === 2) storyV2(data.story);
  const s = merge(newState(), data);
  s.boss.f = null; s.ev.buffs = (s.ev.buffs || []).filter(b => b && typeof b.m === 'number' && typeof b.t === 'number');
  s.laws = (s.laws || []).filter(id => LAWS.some(l => l.id === id));
  // saves from before 0.10.1 had no active-time or play-day counters: credit them from what they already played
  if (!(data.st && data.st.act != null) && s.st.play > 60) { s.st.act = s.st.play; s.st.dayN = Math.max(1, Math.ceil((Date.now() - s.started) / 864e5)); }
  return s;
}
function load() {
  let corrupt = false;
  for (const k of [SAVE_KEY, SAVE_KEY + '_bak']) {
    try { const str = localStorage.getItem(k); if (!str) continue; S = hydrate(parseSave(str)); dirty(); fxCheck(); calc(); return { ok: true, corrupt, fromBak: k !== SAVE_KEY }; }
    catch (e) { corrupt = true; }
  }
  S = newState(); dirty(); fxCheck(); calc(); return { ok: false, corrupt };
}
function exportSave() { S.sec.export = 1; return btoa(unescape(encodeURIComponent(packSave()))); }
function importSave(str) {
  const data = parseSave(decodeURIComponent(escape(atob(str.trim()))));
  S = hydrate(data); S.sec.import = 1; S.last = Date.now(); dirty(); fxCheck(); calc(); save(); return true;
}
function offlineProgress() {
  const away = (Date.now() - S.last) / 1000;
  if (!(away > 10)) return null;
  fxCheck();
  if (away >= 43200) S.sec.nap = 1;
  const cap = offH() * 3600, used = Math.min(away, cap);
  const e0 = S.lifeE, steps = Math.min(600, Math.ceil(used / 2)), h = used / steps;
  for (let i = 0; i < steps; i++) tick(h, true);
  S.last = Date.now();
  return { away, used, cap, gained: S.lifeE.sub(e0) };
}

