# Loads the real game WITH its Content-Security-Policy on (no eval from the test) and checks nothing is blocked.
import sys, pathlib, time
from playwright.sync_api import sync_playwright
html = pathlib.Path(sys.argv[1]).resolve()
bad = []
with sync_playwright() as p:
    b = p.chromium.launch(executable_path='/opt/pw-browsers/chromium-1243/chrome-linux64/chrome')
    for vp in ({'width': 390, 'height': 844}, {'width': 1280, 'height': 900}):
        pg = b.new_page(viewport=vp)
        pg.on('pageerror', lambda e: bad.append('JS: ' + str(e)))
        pg.on('console', lambda m: bad.append('CSP: ' + m.text) if 'Content Security Policy' in m.text or 'Refused' in m.text else None)
        pg.goto(html.as_uri()); time.sleep(3)
        pg.locator('input[type=checkbox]').first.click(force=True); pg.get_by_text('Start', exact=True).click(); time.sleep(4)
        n=0
        for tab in pg.locator('#nav button').all()[:14]:
            try: tab.click(timeout=2000, force=True); time.sleep(0.6); n+=1
            except Exception as e: print("click fail", str(e)[:160]); break
        print(vp['width'], 'tabs clicked', n); assert pg.locator('canvas').count() > 0, 'no canvas'
        pg.screenshot(path=f'/tmp/csp_{vp["width"]}.png'); pg.close()
    b.close()
print('\n'.join(bad) or 'CSP OK: no blocked resources, no JS errors')
sys.exit(1 if bad else 0)
