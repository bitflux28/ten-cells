# 0.19 checks: 25 makers that unlock by owning the previous tier, Fabricators V/VI, Buy max in every prestige shop,
# the reveal ETA, the new-maker note, old saves, and no JS errors.
import sys, subprocess, time, pathlib
from playwright.sync_api import sync_playwright
html, out = pathlib.Path(sys.argv[1]).resolve(), pathlib.Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
srv = subprocess.Popen([sys.executable, '-m', 'http.server', '8781', '-d', str(html.parent)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL); time.sleep(1)
errs = []
def ready(pg):
    pg.wait_for_function("typeof S!=='undefined' && S", timeout=60000); pg.wait_for_timeout(1200)
    if pg.query_selector('#tosck'): pg.evaluate("document.getElementById('tosck').click(); setTimeout(()=>document.getElementById('tosgo').click(),200)")
    pg.wait_for_function("!document.getElementById('load')", timeout=60000); pg.wait_for_timeout(600)
    pg.evaluate("document.querySelectorAll('.sheet .btn[data-a=close]').forEach(b=>b.click())")
def nav(pg, t): pg.evaluate(f"document.querySelector('[data-a=\"nav:{t}\"]').click()"); pg.wait_for_timeout(500)
try:
    with sync_playwright() as p:
        b = p.chromium.launch(executable_path='/opt/pw-browsers/chromium-1243/chrome-linux64/chrome')
        pg = b.new_page(viewport={'width': 390, 'height': 844}); pg.on('pageerror', lambda e: errs.append('JS: ' + str(e)))
        pg.goto('http://localhost:8781/' + html.name); ready(pg)
        r = pg.evaluate("[NG, GEN_SPR.length, GEN_SPR.every(n => SPR[n]), [...Array(NG)].map((_, i) => hue(i)).every(h => h !== undefined), UPG.length, ACH.filter(a => /^g\\d+_/.test(a.id)).length, GENS[24].n, D(GENS[24].c).toString()]")
        print('makers', r); assert r[0] == 25 and r[1] == 25 and r[2] and r[3] and r[5] == 125 and r[7] == '1e+400', r
        assert pg.evaluate("[rebReq().toString(), GENS[0].p, startEnergy().toString()]") == ['1000000000000', 0.2, '10']
        # tier 11 shows up as soon as you own a tier 10, no dimension needed
        pg.evaluate("S.gens = Array(NG).fill(0); S.gens.fill(1, 0, 10); S.energy = D('1e30'); calc(); render()"); pg.wait_for_timeout(300)
        assert pg.evaluate("[genVisible(10), genVisible(11), dimOn('fractal')]") == [True, False, False]
        sig = pg.evaluate("document.querySelector('.gen.locked .g-sub')?.textContent"); print('signal:', sig)
        assert sig and 'Fractal Lattice' in sig and 'dimension' not in sig, sig
        # buy along the whole ladder, including the three past 1e308
        pg.evaluate("S.energy = D('1e500'); for (let i = 10; i < NG; i++) buyGen(i, 1); calc()")
        r = pg.evaluate("[S.gens.slice(10).every(g => g === 1), C.prod.slice(14).every(p => p.gt(0)), S.energy.gt(0)]"); assert all(r), r
        # Fabricators V and VI turn on autobuyers 15-25
        pg.evaluate("for (const id of ['a4','a5','a6','a13','a14','a15']) S.upg[id] = 1; S.inf.root = 1; S.inf['break'] = 1; dirty(); fxCheck(); calc()")
        assert pg.evaluate('!!FX.flag.broken'), 'break'
        assert pg.evaluate("FX.ag.slice(14).every(Boolean) && FX.ag.length === NG")
        pg.evaluate("S.energy = D('1e500'); S.auto.gens = Array(NG).fill(1); const before = S.gens.slice(); for (let k = 0; k < 30; k++) tick(0.1); window.__auto = S.gens.slice(14).map((g, i) => g - before[14 + i])")
        a = pg.evaluate("__auto"); print('autobought 15-25', a); assert all(x > 0 for x in a), a
        # engine page lists all 25
        nav(pg, 'engine'); assert pg.evaluate("document.querySelectorAll('.gen:not(.locked)').length") == 25
        pg.evaluate("document.querySelectorAll('.gen')[24].scrollIntoView({block:'center'})"); pg.wait_for_timeout(300); pg.screenshot(path=str(out / 'engine-25.png'))
        # upgrade tree draws with 55 new nodes
        nav(pg, 'upgrades'); pg.wait_for_timeout(400); assert pg.evaluate("document.querySelectorAll('.tvw .node').length") >= 200
        assert pg.evaluate("['g24_4','a14','a15'].every(id => document.querySelector(`[data-a=\"node:${id}\"]`))"), 'new tree nodes missing'
        assert 'undefined' not in pg.evaluate("document.querySelector('#page').textContent"), 'tree shows undefined'
        assert pg.evaluate("UPG.filter(u => u.cat === 'gen').every(u => !/undefined/.test(nodeIcon(u)))")
        pg.screenshot(path=str(out / 'tree.png'))
        # Buy max: every shop
        pg.evaluate("""S.frag = D(1e6); S.sing = D(30); S.cores = D(5); S.ve = D(12); S.ip = D(1e6); S.up = D(1e9); S.perks = {}; S.supg = {}; S.aupg = {}; S.vupg = {}; S.inf = {}; S.uupg = {};
          Object.assign(S.cnt, { reb: 5, sing: 3, asc: 2, tra: 1, inf: 1, ult: 1 }); dirty(); fxCheck(); calc()""")
        res = {}
        for sub, k, cur in [('reb', 'perk', 'frag'), ('sing', 'su', 'sing'), ('asc', 'au', 'cores'), ('tra', 'vu', 've'), ('inf', 'inf', 'ip'), ('ult', 'uu', 'up')]:
            nav(pg, 'prestige'); pg.evaluate(f"ACT.sub('pre:{sub}') ?? 0") if False else pg.evaluate(f"UI.sub.pre = '{sub}'; render()"); pg.wait_for_timeout(300)
            has = pg.evaluate(f"!!document.querySelector('[data-a=\"smax:{k}\"]')")
            before = pg.evaluate(f"S.{cur}.toString()")
            can = pg.evaluate(f"TX.smaxcan('{k}')")
            pg.evaluate(f"document.querySelector('[data-a=\"smax:{k}\"]').click()"); pg.wait_for_timeout(300)
            left = pg.evaluate(f"TX.smaxcan('{k}')")
            res[k] = (has, can, before, pg.evaluate(f"S.{cur}.toString()"), left)
            if sub == 'reb': pg.screenshot(path=str(out / 'perks.png'))
            assert has and can and not left, (k, res[k])
        print('buy max', res)
        assert not pg.evaluate("!!S.uupg.omega"), 'Buy max must never buy the Omega Engine'
        assert pg.evaluate("Object.values(S.perks).reduce((a, b) => a + b, 0)") > 10
        # nothing affordable -> a note, no change
        pg.evaluate("window.__tt = []; const T0 = toast; toast = function (t) { __tt.push(t); return T0.apply(this, arguments); }; S.frag = D(0); UI.sub.pre = 'reb'; render()"); pg.wait_for_timeout(200)
        pg.evaluate("document.querySelector('[data-a=\"smax:perk\"]').click()"); pg.wait_for_timeout(300)
        assert 'Nothing affordable yet' in pg.evaluate("__tt"), pg.evaluate("__tt")
        # new-maker note: fresh tiers only
        pg.evaluate("S.auto.gens = Array(NG).fill(0); S.energy = D(0); S.gens = Array(NG).fill(0); S.gens.fill(1, 0, 15); for (const k of Object.keys(S.ach)) if (/^g1[5-9]_|^g2\\d_/.test(k)) delete S.ach[k]; calc()"); pg.wait_for_timeout(1300)
        pg.evaluate("__tt.length = 0; S.gens[15] = 1; calc()"); pg.wait_for_timeout(1300)
        t = pg.evaluate("__tt"); print('notes:', t); assert 'New energy maker: Spiral Dynamo' in t, t
        # an old 0.18 save with 14 makers loads into 25
        pg.evaluate("""S.gens = S.gens.slice(0, 14); const d = JSON.parse(JSON.stringify(S)); save(); location.reload()""")
        pg.wait_for_timeout(500); ready(pg)
        assert pg.evaluate("S.gens.length") == 25
        b.close()
finally: srv.terminate()
assert not errs, errs[:3]
print('ALL OK')
