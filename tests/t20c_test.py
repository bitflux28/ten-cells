# 0.20 part 3: daily rewards (claim, cycle, Double Day boost, models), automation rules, Auto tab, story time missions.
import sys, subprocess, time, pathlib
from playwright.sync_api import sync_playwright
html, out = pathlib.Path(sys.argv[1]).resolve(), pathlib.Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
srv = subprocess.Popen([sys.executable, '-m', 'http.server', '8786', '-d', str(html.parent)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL); time.sleep(1)
errs = []
def ready(pg):
    pg.wait_for_function("typeof S!=='undefined' && S", timeout=60000); pg.wait_for_timeout(1200)
    if pg.query_selector('#tosck'): pg.evaluate("document.getElementById('tosck').click(); setTimeout(()=>document.getElementById('tosgo').click(),200)")
    pg.wait_for_function("!document.getElementById('load')", timeout=60000); pg.wait_for_timeout(600)
    pg.evaluate("document.querySelectorAll('.sheet .btn[data-a=close]').forEach(b=>b.click())")
try:
    with sync_playwright() as p:
        b = p.chromium.launch(executable_path='/opt/pw-browsers/chromium-1243/chrome-linux64/chrome', args=['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'])
        pg = b.new_page(bypass_csp=True, viewport={'width': 1300, 'height': 900}); pg.on('pageerror', lambda e: errs.append('JS: ' + str(e)))
        pg.goto('http://localhost:8786/' + html.name); ready(pg)
        # story: no 'different days' missions, later time missions shorter
        r = pg.evaluate("[CHAPTERS.some(c => c.m.some(m => /different days/.test(m[0]))), CHAPTERS[9].m.map(m => m[0]).join(' | '), CHAPTERS[23].m.map(m => m[0]).join(' | ')]"); print('story', r)
        assert not r[0] and '2 hours' in r[1] and '4.3 hours' in r[2], r
        # daily rewards
        pg.evaluate("S.gens[0] = 100; S.gens[1] = 40; calc(); UI.tab = 'engine'; render()"); pg.wait_for_timeout(1500)
        assert pg.evaluate("!!document.querySelector('.daily.rdy')"), 'daily card not ready'
        pg.screenshot(path=str(out / 'daily-card.png'), clip={'x': 300, 'y': 120, 'width': 700, 'height': 300})
        pg.evaluate("document.querySelector('.daily').click()"); pg.wait_for_timeout(2500)
        lit = pg.evaluate("[...document.querySelectorAll('.dgrid canvas.m3')].map(c => { const d = c.getContext('2d').getImageData(0, 0, c.width, c.height).data; let n = 0; for (let i = 3; i < d.length; i += 4) if (d[i]) n++; return n; })"); print('reward models drawn (pixels each):', lit)
        assert len(lit) == 7 and all(n > 150 for n in lit), lit
        pg.screenshot(path=str(out / 'daily-sheet.png'))
        e0 = pg.evaluate("S.energy.toNumber()")
        pg.evaluate("document.querySelector('[data-a=\"dailyclaim\"]').click()"); pg.wait_for_timeout(500)
        r = pg.evaluate("[S.daily.day, S.daily.last === new Date().toDateString(), S.energy.toNumber() > " + str(e0) + ", dailyReady()]"); print('claim day 1', r)
        assert r == [1, True, True, False], r
        assert pg.evaluate("claimDaily()") is None, 'claimed twice in one day'
        # skip ahead to Double Day (day 5) on a later date
        pg.evaluate("S.daily.day = 4; S.daily.last = 'yesterday'; window.__e0 = C.eps.toNumber(); claimDaily(); calc()")
        r = pg.evaluate("[boostOn(), S.daily.bm, Math.round((S.daily.bt - Date.now()) / 3600e3), C.eps.toNumber() / __e0, multParts().some(p => /Daily reward/.test(p[0]))]"); print('double day', r)
        assert r[0] and r[1] == 2 and r[2] == 24 and abs(r[3] - 2) < 0.05 and r[4], r
        pg.evaluate("S.daily.day = 6; S.daily.last = 'x'; const eggs = S.pets.eggs.length; claimDaily(); window.__eggs = S.pets.eggs.length - eggs"); r = pg.evaluate("[S.daily.bm, __eggs]"); print('crown', r); assert r[0] == 3 and r[1] == 1
        # survives Ultra
        pg.evaluate("ultraReset()"); assert pg.evaluate("S.daily.day") == 7
        # automation: Auto tab on PC, autopilot perk, timer and multiple rules really fire
        assert pg.evaluate("TABS.some(t => t[0] === 'auto')")
        pg.evaluate("S.frag = D(1000); S.cnt.reb = 3; buyPerk('autoR'); fxCheck(); calc()")
        assert pg.evaluate("!!FX.flag.autoReb"), 'autopilot perk did not unlock Auto-Rebirth'
        pg.evaluate("document.querySelector('[data-a=\"nav:auto\"]').click()"); pg.wait_for_timeout(500)
        assert pg.evaluate("!!document.querySelector('[data-a=\"amode:reb:time\"]')")
        pg.evaluate("document.querySelector('[data-a=\"amode:reb:time\"]').click()"); pg.wait_for_timeout(300)
        pg.evaluate("S.auto.reb = 1; S.auto.rebT = '1'; render()"); pg.wait_for_timeout(300)
        pg.screenshot(path=str(out / 'auto-tab.png'))
        n0 = pg.evaluate("S.cnt.reb")
        pg.evaluate("S.runE = D('1e14'); S.st.run = 30; for (let k = 0; k < 20; k++) tick(0.1)"); assert pg.evaluate("S.cnt.reb") == n0, 'timer fired early'
        pg.evaluate("S.runE = D('1e14'); S.st.run = 61; for (let k = 0; k < 20; k++) tick(0.1)"); assert pg.evaluate("S.cnt.reb") == n0 + 1, 'timer rule did not fire'
        pg.evaluate("S.auto.rebM = 'mult'; S.auto.rebX = '1000'; S.frag = D(1e9); S.runE = D('1e14'); for (let k = 0; k < 20; k++) tick(0.1)"); assert pg.evaluate("S.cnt.reb") == n0 + 1, 'x-have rule fired too early'
        pg.evaluate("S.frag = D(1); S.runE = D('1e30'); for (let k = 0; k < 20; k++) tick(0.1)"); assert pg.evaluate("S.cnt.reb") == n0 + 2, 'x-have rule did not fire'
        print('automation rules OK; anext:', pg.evaluate("TX.anext('reb')"))
        # new automations start off, with a note; old choices stick after a prestige
        pg.evaluate("window.__tt = []; const T0 = toast; toast = function (t) { __tt.push(t); return T0.apply(this, arguments); }; S.upg.a4 = 1; dirty(); fxCheck(); calc(); autoWatch()")
        r = pg.evaluate("[S.auto.gens.slice(0, 3).join(), __tt.some(t => /start off/.test(t))]"); print('new autobuyers:', r); assert r == ['0,0,0', True], r
        pg.evaluate("S.auto.gens[0] = 1; S.runE = D('1e15'); prestige('reb'); S.upg.a4 = 1; dirty(); fxCheck(); calc(); autoWatch()"); assert pg.evaluate("S.auto.gens[0]") == 1, 'choice lost after prestige'
        # dimensions explain how to unlock
        pg.evaluate("closeSheet(); S.cnt.asc = 1; UI.tab = 'worlds'; UI.sub.wor = 'dims'; render()"); pg.wait_for_timeout(400)
        t = pg.evaluate("document.querySelector('.dunlock')?.textContent || ''"); print('how-to:', t[:200], pg.evaluate("[UI.tab, UI.sub.wor, S.cnt.asc, document.getElementById('page').textContent.slice(0, 200)]")); assert 'How to unlock a dimension' in t and 'Next up' in t
        assert pg.evaluate("document.querySelectorAll('.world .rqh').length") > 0
        pg.screenshot(path=str(out / 'dims.png'))
        b.close()
finally: srv.terminate()
assert not errs, errs[:3]
print('ALL OK')
