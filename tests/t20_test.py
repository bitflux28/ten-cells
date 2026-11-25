# 0.20 checks: Graphics panel (fps + pixel slider + preview), sky extras, 30 fps keeps animation speed, hard achievements.
import sys, subprocess, time, pathlib
from playwright.sync_api import sync_playwright
html, out = pathlib.Path(sys.argv[1]).resolve(), pathlib.Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
srv = subprocess.Popen([sys.executable, '-m', 'http.server', '8782', '-d', str(html.parent)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL); time.sleep(1)
errs = []
def ready(pg):
    pg.wait_for_function("typeof S!=='undefined' && S", timeout=60000); pg.wait_for_timeout(1200)
    if pg.query_selector('#tosck'): pg.evaluate("document.getElementById('tosck').click(); setTimeout(()=>document.getElementById('tosgo').click(),200)")
    pg.wait_for_function("!document.getElementById('load')", timeout=60000); pg.wait_for_timeout(600)
    pg.evaluate("document.querySelectorAll('.sheet .btn[data-a=close]').forEach(b=>b.click())")
try:
    with sync_playwright() as p:
        b = p.chromium.launch(executable_path='/opt/pw-browsers/chromium-1243/chrome-linux64/chrome')
        pg = b.new_page(bypass_csp=True, viewport={'width': 390, 'height': 844}, device_scale_factor=2); pg.on('pageerror', lambda e: errs.append('JS: ' + str(e)))
        pg.goto('http://localhost:8782/' + html.name); ready(pg)
        assert pg.evaluate("[S.set.fps, S.set.px, PXS, TCAPP]") == [60, 3, 3, False]
        pg.screenshot(path=str(out / 'sky.png'))
        # extras exist and draw
        assert pg.evaluate("!!(SP.ex && SP.ex.gal.length === 2 && SP.ex.ast.length === 7 && SP.ex.ring)")
        # graphics panel
        pg.evaluate("UI.tab = 'stats'; UI.sub.sta = 'set'; render()"); pg.wait_for_timeout(500)
        assert pg.evaluate("!!document.getElementById('pxr') && !!document.getElementById('pxprev') && !!document.querySelector('[data-a=\"fps:30\"]')")
        assert not pg.evaluate("!!document.querySelector('[data-a=\"collapse\"]')"), 'collapse only in the PC app'
        pg.evaluate("document.getElementById('pxr').scrollIntoView({block:'center'})"); pg.wait_for_timeout(300)
        lw3 = pg.evaluate("LW")
        # drag the slider like a person
        q = pg.evaluate("(() => { const r = document.getElementById('pxr').getBoundingClientRect(); return [r.left, r.top, r.width, r.height]; })()")
        pg.mouse.click(q[0] + q[2] - 4, q[1] + q[3] / 2); pg.wait_for_timeout(500)
        r = pg.evaluate("[S.set.px, PXS, LW, document.querySelector('[data-t=\"pxl\"]').textContent, document.getElementById('pxprev').width]"); print('slider max:', r, 'LW was', lw3)
        assert r[0] == 6 and r[1] == 6 and r[2] < lw3 and '6' in r[3], r
        pg.screenshot(path=str(out / 'px6.png'))
        pg.mouse.click(q[0] + 4, q[1] + q[3] / 2); pg.wait_for_timeout(500)
        r = pg.evaluate("[S.set.px, LW]"); print('slider min:', r); assert r[0] == 2 and r[1] > lw3
        pg.screenshot(path=str(out / 'px2.png'))
        pg.evaluate("S.set.px = 3; resize(); render()"); pg.wait_for_timeout(300)
        # pixel size survives a reload
        pg.evaluate("S.set.px = 4; save(); location.reload()"); pg.wait_for_timeout(500); ready(pg)
        assert pg.evaluate("[S.set.px, PXS]") == [4, 4]
        pg.evaluate("S.set.px = 3; resize()")
        # 30 fps: half the drawn frames, same animation speed
        def rate(fps):
            pg.evaluate(f"S.set.fps = {fps}; window.__fb = 0; window.__rx0 = UI.rxA || 0; if (!window.__wrapped) {{ const F0 = frameBody; frameBody = function (t) {{ __fb++; return F0(t); }}; window.__wrapped = 1; }}")
            n0 = pg.evaluate("(() => { let n = 0; const f = () => { n++; if (performance.now() < window.__end) requestAnimationFrame(f); }; window.__end = performance.now() + 3000; window.__rafN = () => n; requestAnimationFrame(f); return 0; })()")
            pg.wait_for_timeout(3200)
            return pg.evaluate("[__fb, (UI.rxA || 0) - __rx0]")
        a60 = rate(60); a30 = rate(30); print('60 fps steps/spin', a60, '30 fps steps/spin', a30)
        assert abs(a30[1] / a60[1] - 1) < 0.25, 'reactor speed changed'
        pg.evaluate("S.set.fps = 60")
        # achievements
        r = pg.evaluate("[ACH.length, ACH.filter(a => a.id.startsWith('l_')).length, ACH.find(a => a.id === 'upall').d]"); print('achievements', r)
        assert r[1] == 30 and str(r[2]).startswith('Own all 213'), r
        pg.evaluate("S.st.taps = 1e7; S.cnt.reb = 1000; for (let k = 0; k < 12; k++) tick(0.1)"); pg.wait_for_timeout(300)
        assert pg.evaluate("!!S.ach.l_tap10m && !!S.ach.l_reb1k && !S.ach.l_all")
        pg.evaluate("for (const a of ACH) if (a.id !== 'l_all') S.ach[a.id] = 1; for (let k = 0; k < 12; k++) tick(0.1)"); assert pg.evaluate("!!S.ach.l_all")
        b.close()
finally: srv.terminate()
assert not errs, errs[:3]
print('ALL OK')
