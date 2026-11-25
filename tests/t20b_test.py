# 0.20 part 2: permanent upgrades, PC tabs (Bosses, Pets), wider PC column, tree view reset after prestige.
import sys, subprocess, time, pathlib
from playwright.sync_api import sync_playwright
html, out = pathlib.Path(sys.argv[1]).resolve(), pathlib.Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
srv = subprocess.Popen([sys.executable, '-m', 'http.server', '8785', '-d', str(html.parent)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL); time.sleep(1)
errs = []
def ready(pg):
    pg.wait_for_function("typeof S!=='undefined' && S", timeout=60000); pg.wait_for_timeout(1200)
    if pg.query_selector('#tosck'): pg.evaluate("document.getElementById('tosck').click(); setTimeout(()=>document.getElementById('tosgo').click(),200)")
    pg.wait_for_function("!document.getElementById('load')", timeout=60000); pg.wait_for_timeout(600)
    pg.evaluate("document.querySelectorAll('.sheet .btn[data-a=close]').forEach(b=>b.click())")
try:
    with sync_playwright() as p:
        b = p.chromium.launch(executable_path='/opt/pw-browsers/chromium-1243/chrome-linux64/chrome')
        # phone first: no extra tabs, narrow column
        ph = b.new_page(bypass_csp=True, viewport={'width': 390, 'height': 844}, is_mobile=True, has_touch=True); ph.on('pageerror', lambda e: errs.append('JS phone: ' + str(e)))
        ph.goto('http://localhost:8785/' + html.name); ready(ph)
        assert not ph.evaluate("TABS.some(t => t[0] === 'bosses')"), 'phone should not get PC tabs'
        ph.close()
        pg = b.new_page(bypass_csp=True, viewport={'width': 1400, 'height': 900}); pg.on('pageerror', lambda e: errs.append('JS: ' + str(e)))
        pg.goto('http://localhost:8785/' + html.name); ready(pg)
        # permanent upgrades
        pg.evaluate("S.energy = D('1e25'); S.gens[0] = 50; S.gens[1] = 20; calc(); UI.tab = 'engine'; render()"); pg.wait_for_timeout(400)
        assert pg.evaluate("!!document.querySelector('.permbar')"), 'engine bar missing'
        e0 = pg.evaluate("C.eps.toNumber()")
        for _ in range(4): pg.evaluate("document.querySelector('[data-a=\"permbuy\"]').click()"); pg.wait_for_timeout(150)
        r = pg.evaluate("[Object.keys(S.perm).sort().join(), permProd(), FX.cost < 1, C.eps.toNumber() / " + str(e0) + "]"); print('perm after 4 buys:', r)
        assert r[0] == 'pm1,pm2,pm3,pm4' and r[1] == 3 and r[2] and r[3] > 2.9, r
        pg.screenshot(path=str(out / 'engine-perm.png'))
        assert any(p[0] == 'Permanent upgrades' for p in pg.evaluate("multParts().map(p => [p[0], p[1].toString()])"))
        # survives Rebirth and the Ultra wipe
        pg.evaluate("S.runE = D('1e15'); prestige('reb')"); assert pg.evaluate("Object.keys(S.perm).length") == 4
        pg.evaluate("ultraReset()"); assert pg.evaluate("Object.keys(S.perm).length") == 4, 'perm lost on Ultra'
        assert pg.evaluate("permProd() === 3 && FX.cost < 1")
        # upgrades tab chip + page
        pg.evaluate("UI.tab = 'upgrades'; UI.sub.upg = 'tree'; render()"); pg.wait_for_timeout(300)
        assert pg.evaluate("!!document.querySelector('.btn.permbtn')")
        pg.evaluate("document.querySelector('.btn.permbtn').click()"); pg.wait_for_timeout(300)
        assert pg.evaluate("UI.sub.upg === 'perm' && document.querySelectorAll('[data-a^=\"perm:\"]').length === 8")
        pg.screenshot(path=str(out / 'perm-page.png'))
        # tree view resets after a prestige
        pg.evaluate("UI.sub.upg = 'tree'; render()"); pg.wait_for_timeout(300)
        pg.evaluate("UI.cam.upg.x += 500; UI.cam.upg.z = 1.4; applyCam()"); pg.evaluate("S.runE = D('1e15'); prestige('reb'); render()"); pg.wait_for_timeout(300)
        assert pg.evaluate("UI.cam.upg.z") != 1.4, 'tree view did not reset'
        # PC tabs and width
        r = pg.evaluate("[TABS.map(t => t[0]).join(), document.getElementById('app').getBoundingClientRect().width]"); print('pc:', r)
        assert 'bosses' in r[0] and 'pets' in r[0] and 650 < r[1] <= 700, r
        pg.evaluate("S.cnt.reb = Math.max(1, S.cnt.reb); document.querySelector('[data-a=\"nav:bosses\"]').click()"); pg.wait_for_timeout(600)
        r = pg.evaluate("[UI.tab, !!document.querySelector('#page .chips'), document.querySelectorAll('#page canvas.m3, #page .boss, #page .row').length]"); print('bosses tab:', r)
        assert r[0] == 'bosses' and not r[1] and r[2] > 0, r
        pg.screenshot(path=str(out / 'pc-bosses.png'))
        pg.evaluate("document.querySelector('[data-a=\"nav:pets\"]').click()"); pg.wait_for_timeout(600)
        assert pg.evaluate("UI.tab === 'pets' && document.getElementById('page').textContent.length > 50")
        # shrink to a phone width: tabs go away and the page falls back
        pg.set_viewport_size({'width': 420, 'height': 900}); pg.wait_for_timeout(700)
        r = pg.evaluate("[UI.tab, TABS.some(t => t[0] === 'pets')]"); print('narrow:', r); assert r == ['crew', False], r
        b.close()
finally: srv.terminate()
assert not errs, errs[:3]
print('ALL OK')
