// ===== Native bridge: the C++ app calls NAT.*; everything here wraps the web game's own rules (data.js + logic.js) =====
// Screens are described as lists of simple blocks, so the C++ side only has to know how to draw a handful of block types.
var NAT = (() => {
  const LDESC = {
    reb: ['Each unspent Fragment boosts production. Spend them on permanent perks below. Rebirth also opens Research and Cosmic Bosses.', 'Resets energy, generators and upgrades.'],
    sing: ['Every Singularity ever earned multiplies production. Spend them on upgrades, feed the Black Hole and take on Challenges.', 'Resets everything from Rebirth, plus Fragments, perks and the Black Hole.'],
    asc: ['Cores open the Dimensions (coming to this app in stage 2) and buy Ascension upgrades.', 'Resets everything from Singularity, plus Singularities and their upgrades.'],
    tra: ['Void Essence rewrites reality: enact Reality Laws that bend the rules, and buy Void upgrades.', 'Resets everything from Ascension, plus Cores, Ascension upgrades and dimension progress (unlocks stay).'],
    inf: ['Reach 1.79e308 energy to crunch into Infinity. Spend IP on the Infinity tree; Break Infinity to go further.', 'Resets every layer below. Research, artifacts, bosses, challenges and achievements stay.'],
    ult: ['The last layer. Earn ' + fmt(D(ULT_IP)) + ' Infinity Points to burn the whole climb down for Ultra Points. They multiply production and Infinity Point gain. Buy the Omega Engine to beat the game.', 'Resets everything except achievements, badges, pets, the story and your Ultra upgrades.'],
  };
  const LC = { reb: '#45c8f5', sing: '#ff8c3f', asc: '#ffcc4d', tra: '#8a63ff', inf: '#dfe6ff', chal: '#ff4f73', ult: '#ff5fd0' };
  const LSPR = { reb: 'diamond', sing: 'hole', asc: 'arrowup', tra: 'spiral', inf: 'inf', ult: 'crown', chal: 'swords' };
  const GSPR = ['cell', 'core', 'flame', 'atom', 'sun', 'dark', 'star', 'galaxy', 'cube', 'inf', 'tri', 'moon', 'moonz', 'sparkle'];
  const HUE = [200, 185, 15, 175, 45, 270, 50, 300, 230, 0, 95, 220, 25, 48];
  const CURS = [['frag', 'diamond', () => S.cnt.reb || S.frag.gt(0)], ['sing', 'hole', () => S.cnt.sing], ['cores', 'tri', () => S.cnt.asc], ['ve', 'moon', () => S.cnt.tra], ['ip', 'inf', () => S.cnt.inf], ['up', 'crown', () => S.cnt.ult], ['rp', 'flask', () => S.cnt.reb]];
  const layerSeen = i => { const c = [S.cnt.reb, S.cnt.sing, S.cnt.asc, S.cnt.tra, S.cnt.inf, S.cnt.ult || 0]; return i === 0 || c[i - 1] > 0 || c.slice(i).some(x => x > 0); };
  const LREQ2 = { reb: () => rebReq(), sing: () => D(1e30), asc: () => D(1e75), inf: () => D('1.79e308') };
  const eta = (c, have = S.energy) => { const r = C.eps.mul(C.sp); if (c.lte(have) || !r.gt(0)) return ''; const t = c.sub(have).div(r).toNumber(); return isFinite(t) && t < 864000 ? fmtT(Math.ceil(t)) : ''; };
  const st = (owned, can) => (owned ? 'done' : can ? 'on' : 'off');
  const row = (title, desc, btn, extra = {}) => Object.assign({ t: 'row', title, desc, btn }, extra);
  const B = (label, act, s, sub) => ({ label, act, st: s, sub: sub || '' });
  const UI = { mode: 1 };
  let lastT = 0;

  function hud() {
    return {
      e: fmt(S.energy), eLog: oom(S.energy), eps: fmt(C.eps),
      curs: CURS.filter(c => c[2]()).map(([k, ic]) => [ic, fmt(S[k])]),
      buffs: S.ev.buffs.map(b => [({ prod: 'Production', tap: 'Taps', cost: 'Costs', speed: 'Speed' })[b.k] + ' ×' + (b.m >= 10 ? Math.round(b.m) : b.m), Math.ceil(b.t)]),
      chal: S.chal.a ? `${CHAL.find(c => c.id === S.chal.a.id).n} ${'I'.repeat(S.chal.a.l)} · ${fmt(S.runE)} / ${fmt(CHAL_GOAL[S.chal.a.l - 1])}` : '',
      ev: S.ev.cur && !FX.flag.autoEvt ? { n: EVENTS[S.ev.cur.k].n, col: EVENTS[S.ev.cur.k].col, t: S.ev.cur.t } : null,
      ver: BUILD_V, not: S.set.not,
    };
  }

  // ---------- Engine ----------
  function engine() {
    const P = [];
    P.push({ t: 'reactor', tap: fmt(C.tap.add(C.tapE)), crit: Math.round(Math.min(1, FX.critC) * 100), heat: typeof heat === 'number' ? heat : 0, notap: inC('notap') || law('haste') });
    const ab = Object.keys(FX.ab);
    if (ab.length) P.push({ t: 'btns', items: ab.map(id => B(ABIL[id].n, 'ab:' + id, abReady(id) ? 'on' : 'off', abReady(id) ? 'Ready' : fmtT(S.ab[id] || 0))) });
    P.push({ t: 'chips', items: [[1, '×1'], [10, '×10'], [25, '×25'], ['max', 'MAX']].map(([m, l]) => [String(m), l, false, String(UI.mode) === String(m)]), act: 'mode' });
    P.push({ t: 'btns', items: [B('Buy everything', 'buyall', GENS.some((g, i) => genVisible(i) && genAllowed(i) && genMaxN(i) > 0) ? 'on' : 'off', 'Max of every generator')] });
    GENS.forEach((g, i) => {
      if (!genVisible(i)) return;
      const q = UI.mode === 'max' ? Math.max(1, genMaxN(i)) : UI.mode, c = genCostN(i, q), can = genAllowed(i) && c.lte(S.energy) && (UI.mode !== 'max' || genMaxN(i) > 0);
      const n = S.gens[i], nx = nextMile(n), pv = [0, ...MILES].filter(t => t <= n).pop() || (FX.flag.mobius ? Math.floor(n / 25) * 25 : Math.floor(n / 100) * 100);
      const e = !can && genAllowed(i) ? eta(c) : '';
      P.push({ t: 'gen', i, n: g.n, own: n.toLocaleString('en-US'), spr: GSPR[i], hue: HUE[i], ban: !genAllowed(i),
        prod: C.prod[i].gt(0) ? `${fmt(C.prod[i])}/s · ${(x => (x < 1 ? '<1' : Math.round(x)))(C.eps.gt(0) ? C.prod[i].div(C.eps).toNumber() * 100 : 0)}% of output` : `Produces ${fmt(D(g.p).mul(C.g))}/s each`,
        mile: (n - pv) / Math.max(1, nx - pv), mileT: inC('nomile') ? 'Milestones disabled' : `Next milestone at ${nx} · ×${C.mp.toFixed(2)}`,
        btn: B(fmt(c), 'buy:' + i, can ? 'on' : 'off', e ? 'in ' + e : UI.mode === 'max' ? `Max ${genMaxN(i)}` : `Buy ${UI.mode}`) });
    });
    const nx = GENS.findIndex((g, i) => !genVisible(i));
    if (nx > 0) P.push(row('Unknown signal', GENS[nx].dim && !dimOn(GENS[nx].dim) ? `Comes with the ${DIMS.find(d => d.id === GENS[nx].dim).n} dimension, far past the multiverse` : `Buy a ${GENS[nx - 1].n} to reveal the next generator`, null, { icon: 'question', dim: true }));
    return P;
  }

  // ---------- Upgrades ----------
  const CATN = { gen: 'Gens', prod: 'Prod', eff: 'Effic', tap: 'Taps', auto: 'Auto', spec: 'Spec' };
  function upgrades(sub, cat) {
    const P = [{ t: 'chips', items: [['tree', 'Upgrades', false, sub === 'tree'], ['research', 'Research', S.cnt.reb < 1, sub === 'research'], ['auto', 'Automation', false, sub === 'auto']], act: 'sub:upg' }];
    if (sub === 'research') {
      P.push({ t: 'p', text: `${fmt(S.rp)} research points · ${fmt(D(rpRate()))}/s · ${Object.keys(S.res).length}/${RES.length} done` });
      RES.forEach((r, i) => { if (!tierOpen(r.tier)) return; const done = !!S.res[r.id], can = resAvail(r, i) && S.rp.gte(r.c);
        P.push(row(r.n, r.d + (resReq(i) && !S.res[resReq(i)] ? ` · needs ${RES[i - 1].n}` : ''), done ? B('Done', '', 'done') : B(fmt(D(r.c)), 'res:' + r.id, can ? 'on' : 'off', 'Research'))); });
      return P;
    }
    if (sub === 'auto') return P.concat(autoPage());
    P.push({ t: 'chips', items: UCATS.map(([k]) => [k, `${CATN[k]} ${UPG.filter(u => u.cat === k && hasU(u.id)).length}/${UPG.filter(u => u.cat === k).length}`, false, cat === k]), act: 'cat' });
    const own = UPG.filter(u => hasU(u.id)).length;
    P.push({ t: 'btns', items: [B(`Buy all affordable · ${own}/${UPG.length} owned`, 'ubuyall', UPG.some(u => upgAvail(u) && upgCost(u).lte(S.energy)) && !inC('noupg') ? 'on' : 'off')] });
    const list = UPG.filter(u => u.cat === cat).sort((a, b) => (hasU(a.id) - hasU(b.id)) || (upgAvail(b) - upgAvail(a)) || a.cost.cmp(b.cost));
    for (const u of list) {
      const owned = hasU(u.id), av = upgAvail(u), c = upgCost(u), can = av && c.lte(S.energy) && !inC('noupg');
      const need = !av && !owned ? [...u.req.filter(r => !hasU(r)).map(r => 'needs ' + UMAP[r].n), ...(u.cond && !condOk(u.cond) ? [condText(u.cond)] : [])].join(' · ') : '';
      const e = av && !can ? eta(c) : '';
      P.push(row(u.n, u.d, owned ? B('Owned', '', 'done') : B(fmt(c), 'upg:' + u.id, can ? 'on' : 'off', e ? 'in ' + e : av ? 'Buy' : 'Locked'), { note: need, dim: !owned && !av }));
    }
    return P;
  }
  function autoPage() {
    const a = S.auto, F = FX.flag, P = []; let any = false;
    const tog = (title, desc, on, act) => P.push({ t: 'toggle', title, desc, on: !!on, act });
    if (FX.autotap) { any = true; tog('Auto-tap', `${FX.autotap} tap${FX.autotap === 1 ? '' : 's'} per second`, a.tap, 'tog:tap'); }
    if (FX.ag.some(x => x)) { any = true; P.push({ t: 'h', text: `Generator autobuyers · ${FX.autoMax ? 'buy max' : 'buy one'} every ${FX.autoInt}s` }); GENS.forEach((g, i) => { if (FX.ag[i]) tog(g.n, '', a.gens[i], 'tog:gen:' + i); }); }
    if (FX.autoUpg) { any = true; tog('Upgrade daemon', 'Buys the cheapest available upgrades', a.upg, 'tog:upg'); }
    if (FX.autoAb) { any = true; tog('Ability daemon', 'Fires abilities as soon as they are ready', a.ab, 'tog:ab'); }
    if (bhOn()) { any = true; P.push({ t: 'btns', title: `Black Hole auto-feed · ${a.feed}% of your energy per second`, items: [0, 1, 2.5, 5, 10].map(v => B(v + '%', 'feedauto:' + v, a.feed === v ? 'on' : 'off')) }); }
    const pre = [['reb', F.autoReb, 'Auto-Rebirth', 'Rebirth Daemon (Singularity shop)'], ['sing', F.autoSing, 'Auto-Singularity', 'Singularity Daemon (Ascension shop)'], ['asc', F.autoAsc, 'Auto-Ascension', 'Ascension Daemon (Void shop)'], ['tra', F.autoTra, 'Auto-Transcendence', 'Transcend Daemon (Infinity tree)'], ['inf', F.autoInf, 'Auto-Infinity', 'Crunch Daemon (Infinity tree)']];
    pre.forEach(([k, ok, n]) => { if (ok) { any = true; tog(n, `When the gain is at least ${a[k + 'At']} (set it in Settings)`, a[k], 'tog:' + k); } });
    if (!any) P.push({ t: 'p', text: 'Nothing is automated yet. Buy Pulse Drone in the Auto branch of the upgrades to start.', muted: true });
    const locked = pre.filter(p => !p[1]).map(p => `${p[2]}: ${p[3]}`);
    if (locked.length) P.push({ t: 'p', text: 'Still to unlock: ' + locked.join(' · '), muted: true });
    return P;
  }

  // ---------- Prestige ----------
  const LSUB = [['reb', 'Rebirth', 0], ['sing', 'Singularity', 1], ['asc', 'Ascension', 2], ['tra', 'Transcend', 3], ['inf', 'Infinity', 4], ['ult', 'Ultra', 5], ['chal', 'Challenges', 1]];
  const shop = (title, list, owned, act, curKey) => [{ t: 'h', text: title }].concat(list.map(u => row(u.n, u.d, owned[u.id] ? B('Owned', '', 'done') : B(fmt(D(u.c)), act + ':' + u.id, S[curKey].gte(u.c) ? 'on' : 'off'))));
  function bonus(id) { return fmt({ reb: () => S.frag.mul(0.05 + 0.02 * lv(S.perks, 'reso')).add(1), sing: () => S.singTotal.add(1).pow(1.5), asc: () => S.coresTotal.add(1).pow(1.5), tra: () => S.veTotal.add(1).pow(2), inf: () => S.ipTotal.add(1).pow(2), ult: () => S.upTotal.add(1).pow(4) }[id]()); }
  function prestigePage(sub) {
    const P = [{ t: 'chips', items: LSUB.map(([k, n, i]) => [k, n, k === 'chal' ? !chalOpen() : !layerSeen(i), k === sub, LC[k]]), act: 'sub:pre' }];
    if (sub === 'chal') return P.concat(chalPage());
    const ly = LAYERS.find(l => l.id === sub), i = LAYERS.indexOf(ly);
    if (!layerSeen(i)) { P.push({ t: 'p', text: `Reach ${LAYERS[i - 1].n} first.`, muted: true }); return P; }
    const can = canPrestige(sub), prog = (sub === 'ult' ? `${fmt(S.ipTotal)} / ${fmt(D(ULT_IP))} Infinity Points` : `${fmt(S.runE)} / ${LREQ[sub]()}`);
    let nxt = '';
    if (can) { const m = S.st.run / 60; if (m > 0.1) nxt = ` · +${fmt(ly.gain().div(m))}/min`; } else if (LREQ2[sub] && !S.chal.a) { const e = eta(LREQ2[sub](), S.runE); if (e) nxt = ` · ready in ${e}`; }
    P.push({ t: 'layer', id: sub, col: LC[sub], spr: LSPR[sub], amt: fmt(S[ly.key]), cur: ly.cur, bonus: bonus(sub) });
    P.push({ t: 'big', label: `${ly.n} for ${fmt(ly.gain())} ${ly.cur}`, act: 'pre:' + sub, st: can ? 'on' : 'off', col: LC[sub] });
    P.push({ t: 'p', text: prog + nxt, muted: true }, { t: 'p', text: LDESC[sub][0] }, { t: 'p', text: LDESC[sub][1], muted: true });
    if (sub === 'reb') { P.push({ t: 'h', text: 'Fragment perks' }); PERKS.forEach(p => { const l = lv(S.perks, p.id), c = Math.ceil(p.c(l)); P.push(row(p.n, p.d, l >= p.max ? B('Maxed', '', 'done') : B(fmt(D(c)), 'perk:' + p.id, S.frag.gte(c) ? 'on' : 'off'), { tag: `${l}/${p.max}` })); }); }
    if (sub === 'sing') {
      P.push({ t: 'h', text: 'Black Hole' }, { t: 'p', text: `Mass ${fmt(S.bh)} · production ×${fmt(D(1 + oom(S.bh)).pow(bhPow()))} (power ${bhPow().toFixed(2)}). Energy you feed is gone, so time it well.` });
      P.push({ t: 'btns', items: [[0.1, '10%'], [0.5, '50%'], [1, 'All']].map(([p, l]) => B('Feed ' + l, 'feed:' + p, 'on')) });
      P.push(...shop('Singularity upgrades', SUPG, S.supg, 'su', 'sing'));
    }
    if (sub === 'asc') P.push(...shop('Ascension upgrades', AUPG, S.aupg, 'au', 'cores'));
    if (sub === 'tra') {
      P.push({ t: 'h', text: `Reality Laws · ${S.laws.length}/${FX.lawSlots} enacted` }, { t: 'p', text: S.lawCd > 0 ? `Reality settles in ${Math.ceil(S.lawCd)}s` : 'Tap a law to enact or repeal it', muted: true });
      LAWS.forEach(l => P.push({ t: 'toggle', title: l.n, desc: `+ ${l.b}   − ${l.d}`, on: law(l.id), act: 'law:' + l.id }));
      P.push(...shop('Void upgrades', VUPG, S.vupg, 'vu', 've'));
    }
    if (sub === 'inf') {
      P.push({ t: 'h', text: `Infinity tree · ${Object.keys(S.inf).length}/${INF.length}` }, { t: 'p', text: FX.flag.broken ? 'Infinity is broken. Energy flows past 1.79e308.' : 'Energy stops at 1.79e308 until you Break Infinity.', muted: true });
      INF.filter(n => n.id !== 'root').sort((a, b) => (!!S.inf[a.id] - !!S.inf[b.id]) || (infAvail(b) - infAvail(a)) || D(a.c).cmp(D(b.c))).forEach(n => {
        const own = !!S.inf[n.id], av = infAvail(n);
        P.push(row(n.n, n.d, own ? B('Owned', '', 'done') : B(fmt(D(n.c)) + ' IP', 'inf:' + n.id, av && S.ip.gte(n.c) ? 'on' : 'off', av ? 'Buy' : 'Locked'), { note: !own && !av ? 'needs ' + n.req.filter(r => !S.inf[r]).map(r => (INF.find(x => x.id === r) || { n: r }).n).join(', ') : '', dim: !own && !av }));
      });
    }
    if (sub === 'ult') { P.push({ t: 'p', text: S.st.beat ? `You beat the game on ${new Date(S.st.beat).toDateString()}. The climb goes on if you want it to.` : 'Buy the Omega Engine at the end of this list to beat the game.' }); P.push(...shop(`Ultra upgrades · ${Object.keys(S.uupg).length}/${UUPG.length}`, UUPG, S.uupg, 'uu', 'up')); }
    return P;
  }
  function chalPage() {
    if (!chalOpen()) return [{ t: 'p', text: 'Reach your first Singularity to unlock Challenges.', muted: true }];
    const P = [{ t: 'p', text: `Entering a challenge restarts your current run under a restriction. Reach the goal to complete it for a permanent reward that grows each level. Goals: ${CHAL_GOAL.map(fmt).join(', ')} energy.`, muted: true }];
    CHAL.forEach(c => {
      const d = S.chal.done[c.id] || 0, act = S.chal.a && S.chal.a.id === c.id;
      P.push(row(c.n, c.d, null, { tag: `${d}/3 cleared`, note: 'Reward: ' + c.rw, good: true }));
      P.push({ t: 'btns', items: act ? [B('Leave challenge', 'chalx', 'bad')] : [1, 2, 3].map(l => d >= l ? B('Level ' + 'I'.repeat(l), '', 'done') : l === d + 1 && !S.chal.a ? B('Start ' + 'I'.repeat(l), `chal:${c.id}:${l}`, 'on') : B('Level ' + 'I'.repeat(l), '', 'off')) });
    });
    return P;
  }

  // ---------- Stats & settings ----------
  function stats(sub) {
    const P = [{ t: 'chips', items: [['stats', 'Statistics', false, sub === 'stats'], ['set', 'Settings', false, sub === 'set']], act: 'sub:sta' }];
    if (sub === 'set') {
      P.push({ t: 'h', text: 'Number format' }, { t: 'chips', items: [['suffix', '1.23M'], ['sci', '1.23e6'], ['eng', '12.3e3']].map(([k, l]) => [k, l, false, S.set.not === k]), act: 'not' });
      P.push({ t: 'toggle', title: 'Sound effects', desc: 'Taps, purchases, events and prestige', on: !!S.set.sfx, act: 'tog:set:sfx' });
      P.push({ t: 'toggle', title: 'Visual effects', desc: 'Particles, glow and starfield motion', on: !!S.set.fx, act: 'tog:set:fx' });
      P.push({ t: 'h', text: 'Your save' }, { t: 'p', text: 'Saves every 5 seconds and when you close the game, with a backup copy. Export copies a save code to your clipboard. Import reads a code from your clipboard, including one exported from the web or iPhone version.', muted: true });
      P.push({ t: 'btns', items: [B('Save now', 'save', 'on'), B('Export', 'export', 'on'), B('Import', 'import', 'on')] });
      P.push({ t: 'btns', items: [B('Open save folder', 'folder', 'on')] });
      P.push({ t: 'h', text: 'Auto-prestige thresholds' });
      ['reb', 'sing', 'asc', 'tra', 'inf'].forEach(k => P.push(row(LAYERS.find(l => l.id === k).n, `Auto fires when the gain is at least ${S.auto[k + 'At']}`, B('Set from clipboard', 'at:' + k, 'on'))));
      P.push({ t: 'h', text: 'Hard reset' }, row('Erase everything', 'All progress is deleted. This cannot be undone.', B('Reset', 'reset', 'bad')));
      P.push({ t: 'p', text: `Ten Cells native ${BUILD_V} · save version ${SAVE_VERSION}`, muted: true, center: true });
      return P;
    }
    const kv = [['Time played', fmtT(S.st.play)], ['This run', fmtT(S.st.run)], ['Energy this run', fmt(S.runE)], ['Lifetime energy', fmt(S.lifeE)], ['Best energy/s', fmt(S.st.maxEps)], ['Game speed', '×' + C.sp.toFixed(2)],
      ['Reactor taps', S.st.taps.toLocaleString('en-US')], ['Critical taps', S.st.crits.toLocaleString('en-US')], ['Upgrades bought', S.st.upg.toLocaleString('en-US')], ['Events caught', S.st.ev.toLocaleString('en-US')],
      ['Rebirths', S.cnt.reb], ['Singularities', S.cnt.sing], ['Ascensions', S.cnt.asc], ['Transcendences', S.cnt.tra], ['Infinities', S.cnt.inf], ['Ultra Prestiges', S.cnt.ult || 0],
      ['Challenges completed', S.st.chal], ['Achievements', achN + '/' + ACH.length], ['Story chapters', S.story.claimed + '/' + CHAPTERS.length], ['Offline cap', offH() + 'h'], ['Milestone multiplier', '×' + C.mp.toFixed(3)]];
    P.push({ t: 'kv', items: kv.map(([a, b]) => [a, String(b)]) });
    P.push({ t: 'h', text: 'What multiplies your production' }, { t: 'kv', items: multParts().filter(p => !D(p[1]).eq(1)).map(([n, v]) => [n, '×' + fmt(D(v))]) });
    P.push({ t: 'p', text: 'Worlds, the story, bosses, pets and achievements screens come to the native app in the next stages. Their bonuses already count here.', muted: true });
    return P;
  }

  // ---------- actions ----------
  function act(a, arg) {
    const [k, x, y] = a.split(':'); let r = {};
    const ok = v => { r.sfx = v ? 'buy' : 'bad'; return v; };
    if (k === 'tap') { const t = doTap(); r = t ? { tap: fmt(t.v), crit: t.crit, sfx: 'tap' } : { sfx: 'bad' }; }
    else if (k === 'mode') UI.mode = x === 'max' ? 'max' : +x;
    else if (k === 'buy') { const n = buyGen(+x, UI.mode === 'max' ? 'max' : UI.mode); ok(n > 0); if (n) calc(); }
    else if (k === 'buyall') { let n = 0; for (let i = GENS.length - 1; i >= 0; i--) n += buyGen(i, 'max') || 0; if (n) { calc(); r = { sfx: 'buy', toast: `Bought ${n.toLocaleString('en-US')} generators` }; } else r = { sfx: 'bad', toast: 'Nothing affordable yet' }; }
    else if (k === 'upg') { if (ok(buyUpg(x))) { fxCheck(); calc(); r.sfx = 'upg'; } }
    else if (k === 'ubuyall') { let n = 0, any = true; while (any) { any = false; for (const u of UPG.filter(upgAvail).sort((a, b) => upgCost(a).cmp(upgCost(b)))) if (buyUpg(u.id)) { n++; any = true; } fxCheck(); } calc(); r = n ? { sfx: 'upg', toast: `Bought ${n} upgrade${n > 1 ? 's' : ''}` } : { sfx: 'bad', toast: 'Nothing affordable yet' }; }
    else if (k === 'res') { if (ok(buyRes(x))) r.sfx = 'upg'; }
    else if (k === 'ab') { if (ok(useAb(x))) r.sfx = 'ab'; }
    else if (k === 'claim') { if (S.ev.cur) { claimEvent(); r.sfx = 'claim'; } }
    else if (k === 'pre') { const l = LAYERS.find(q => q.id === x); if (!canPrestige(x)) r = { sfx: 'bad', toast: S.chal.a ? 'Finish or leave your challenge first' : `Needs ${LREQ[x]()}` }; else r = { confirm: { title: l.n, text: `+${fmt(l.gain())} ${l.cur}\n\n${LDESC[x][1]}`, yes: 'dopre:' + x, label: l.n } }; }
    else if (k === 'dopre') { const g = prestige(x); if (g) { calc(); r = { flash: LAYERS.find(q => q.id === x).n, sub: `+${fmt(g)} ${LAYERS.find(q => q.id === x).cur}`, col: LC[x], sfx: 'prestige', go: 'engine' }; Q.splice(0, Q.length, ...Q.filter(q => q.type !== 'prestige')); } }
    else if (k === 'perk') { if (ok(buyPerk(x))) r.sfx = 'upg'; }
    else if (k === 'su') { if (ok(buySU(x))) r.sfx = 'upg'; } else if (k === 'au') { if (ok(buyAU(x))) r.sfx = 'upg'; } else if (k === 'vu') { if (ok(buyVU(x))) r.sfx = 'upg'; }
    else if (k === 'uu') { if (ok(buyUU(x))) r.sfx = 'upg'; } else if (k === 'inf') { if (ok(buyInf(x))) r.sfx = 'upg'; }
    else if (k === 'feed') { if (ok(feedBH(+x))) r.sfx = 'well'; }
    else if (k === 'feedauto') S.auto.feed = +x;
    else if (k === 'law') { if (!toggleLaw(x)) r = { sfx: 'bad', toast: S.lawCd > 0 ? 'Reality is still settling' : 'All law slots are full' }; else r.sfx = 'upg'; }
    else if (k === 'chal') { const c = CHAL.find(q => q.id === x); r = { confirm: { title: `Start ${c.n} ${'I'.repeat(+y)}?`, text: `Your current run restarts under the restriction. Goal: ${fmt(CHAL_GOAL[+y - 1])} energy.`, yes: `dochal:${x}:${y}`, label: 'Start' } }; }
    else if (k === 'dochal') { if (startChal(x, +y)) r = { go: 'engine', sfx: 'prestige' }; }
    else if (k === 'chalx') r = { confirm: { title: 'Leave the challenge?', text: 'Your current run restarts without a reward.', yes: 'dochalx', label: 'Leave', bad: true } };
    else if (k === 'dochalx') exitChal();
    else if (k === 'tog') { if (x === 'gen') S.auto.gens[+y] = +!S.auto.gens[+y]; else if (x === 'set') S.set[y] = +!S.set[y]; else S.auto[x] = +!S.auto[x]; }
    else if (k === 'not') { S.set.not = x; }
    else if (k === 'at') { const v = String(arg || '').trim(); if (v && isFinite(parseNum(v).mantissa)) { S.auto[x + 'At'] = v; r.toast = `Threshold set to ${v}`; } else r = { sfx: 'bad', toast: 'Copy a number like 1e20 first, then press the button' }; }
    else if (k === 'save') { save(); r.toast = 'Saved'; }
    else if (k === 'export') { r = { clip: exportSave(), toast: 'Save code copied to your clipboard' }; }
    else if (k === 'import') { try { if (!String(arg || '').trim()) throw 0; importSave(String(arg)); fxCheck(); calc(); save(); r = { toast: 'Save loaded', sfx: 'claim', go: 'engine' }; } catch (e) { r = { sfx: 'bad', toast: 'Your clipboard has no Ten Cells save code. Copy one with Export first.' }; } }
    else if (k === 'reset') r = { confirm: { title: 'Erase everything?', text: 'All progress, prestige layers, artifacts and achievements will be deleted. This cannot be undone.', yes: 'doreset', label: 'Erase', bad: true } };
    else if (k === 'doreset') { S = newState(); dirty(); fxCheck(); calc(); save(); r = { toast: 'A new engine hums to life', go: 'engine' }; }
    if (!('sfx' in r) && !r.confirm && k !== 'tap') r.sfx = 'ui';
    return JSON.stringify(r);
  }

  return {
    boot() {
      const L = load(); const off = offlineProgress(); fxCheck(); calc();
      return JSON.stringify({ ok: L.ok, corrupt: L.corrupt, off: off && off.away > 60 ? { away: fmtT(off.away), used: fmtT(off.used), capped: off.away > off.used, gained: fmt(off.gained) } : null });
    },
    // the web game ticks every 100 ms with dt capped at 1 s; a longer gap (window asleep) counts as offline time
    step(dt) {
      if (dt > 3) { S.last = Date.now() - dt * 1000; const off = offlineProgress(); calc(); return off && off.away > 60 ? JSON.stringify({ away: fmtT(off.away), gained: fmt(off.gained) }) : ''; }
      if (dt > 0) tick(Math.min(dt, 1));
      S.last = Date.now(); return '';
    },
    notices() { const out = Q.splice(0, Q.length).map(q => ({ type: q.type, text: q.text, auto: !!q.auto })); return JSON.stringify(out); },
    view(tab, sub, cat) {
      const page = tab === 'engine' ? engine() : tab === 'upgrades' ? upgrades(sub || 'tree', cat || 'gen') : tab === 'prestige' ? prestigePage(sub || 'reb') : stats(sub || 'stats');
      return JSON.stringify({ hud: hud(), page, dots: { upgrades: !inC('noupg') && UPG.some(u => upgAvail(u) && upgCost(u).lte(S.energy)), prestige: LAYERS.some(l => canPrestige(l.id)) } });
    },
    settings() { return JSON.stringify({ sfx: !!S.set.sfx, fx: !!S.set.fx }); },
    act, save() { save(); },
  };
})();
