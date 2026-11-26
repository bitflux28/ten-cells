# Bakes the web game's art into image sheets for the native Rust build:
# sprites (12x12), 3D models (16-frame turntables), world pictures (animated), and one sky per world.
import json, time, base64, pathlib, sys
from playwright.sync_api import sync_playwright

OUT = pathlib.Path('/home/claude/port/assets'); OUT.mkdir(exist_ok=True)
d = json.load(open('/home/claude/port/data.json'))

def boot(pg):
    pg.goto('file:///home/claude/ten-cells/index.html'); time.sleep(3)
    pg.locator('input[type=checkbox]').first.click(force=True); pg.get_by_text('Start', exact=True).click(); time.sleep(2)
    pg.evaluate("() => { S.set.fx = 1; UI.tab = 'stats'; render(); }")

def save_png(name, data_url):
    (OUT / name).write_bytes(base64.b64decode(data_url.split(',', 1)[1]))

with sync_playwright() as p:
    b = p.chromium.launch(executable_path='/opt/pw-browsers/chromium-1243/chrome-linux64/chrome', args=['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'])
    pg = b.new_page(viewport={'width': 720, 'height': 960}); boot(pg)

    # ---- sprites: every SPR name plain, every rarity swap, every boss swap. One 12px row each.
    spr = pg.evaluate("""() => {
      const list = []; for (const k of Object.keys(SPR)) list.push([k, null]);
      RAR_SW.forEach((sw, r) => Object.values(ART_SPR).forEach(k => list.push([k, sw, 'r' + r])));
      BOSS_SPR.forEach(([k, sw], i) => list.push([k, sw, 'boss' + i]));
      const c = document.createElement('canvas'); c.width = 12 * list.length; c.height = 12; const x = c.getContext('2d');
      const idx = {}; list.forEach(([k, sw, tag], i) => { x.drawImage(sprCanvas(k, sw || undefined), i * 12, 0); idx[tag ? k + '@' + tag : k] = i; });
      return { url: c.toDataURL(), idx };
    }""")
    save_png('sprites.png', spr['url']); json.dump(spr['idx'], open(OUT / 'sprites.json', 'w'))
    print('sprites', len(spr['idx']))

    # ---- world pictures: 12 worlds at 50px, 96 frames over 6.4s, plus a greyed locked still
    worlds = pg.evaluate("""() => {
      const out = {}; const c = document.createElement('canvas'); c.width = c.height = 50;
      for (const d of DIMS) {
        const s = document.createElement('canvas'); s.width = 50 * 96; s.height = 100; const x = s.getContext('2d');
        for (let f = 0; f < 96; f++) { drawWorld(c, d.id, f * 1000 / 15, false); x.drawImage(c, f * 50, 0); }
        drawWorld(c, d.id, 0, true); x.drawImage(c, 0, 50);
        out[d.id] = s.toDataURL();
      }
      return out;
    }""")
    for k, v in worlds.items(): save_png(f'world_{k}.png', v)
    print('worlds', len(worlds))

    # ---- skies: the composed background for the default sky and each world, at 240x320 art pixels
    sky_ids = ['default'] + [x['id'] for x in d['DIMS']]
    for i, sid in enumerate(sky_ids):
        url = pg.evaluate("""k => {
          for (let j = 0; j < DIMS.length; j++) S.dims[DIMS[j].id].u = j < k ? 1 : 0;
          S.set.px = 3; PXS = 3; resize(); buildSpace(); SP.px = SP.py = SP.tx = SP.ty = 0; SP.scroll = 0;
          drawStars(1000); return document.getElementById('stars').toDataURL();
        }""", i)
        save_png(f'sky_{sid}.png', url)
    pg.evaluate("() => { for (const d of DIMS) S.dims[d.id].u = 0; }")
    print('skies', len(sky_ids))

    # ---- 3D models: 16-frame loops at 128px. Characters get an idle and a talking loop.
    specs = [f'char:{c}' for c in d['CHARS']] + [f'pet:{p["id"]}:{s}' for p in d['PETS'] for s in range(3)] + [f'boss:{i}' for i in range(len(d['BOSSES']))] + [f'egg:{r}' for r in range(5)] + [f'rew:{k}' for k in range(7)]
    jobs = [(s, False) for s in specs] + [(f'char:{c}', True) for c in d['CHARS']]
    for spec, talk in jobs:
        url = pg.evaluate("""([spec, talk]) => {
          const v = m3view(spec); if (!v) return null; const N = 128;
          M3.r.setSize(N, N, false); const s = document.createElement('canvas'); s.width = N * 16; s.height = N; const x = s.getContext('2d');
          for (let f = 0; f < 16; f++) {
            const t = 20000 + f * 125; v.model.userData.anim(t, { talk, happy: false, ready: true, hit: false });
            v.model.rotation.y = 0.5 * 0 + Math.sin(f / 16 * 2 * Math.PI) * 0.35; M3.r.render(v.scene, v.cam); x.drawImage(M3.r.domElement, f * N, 0);
          }
          return s.toDataURL();
        }""", [spec, talk])
        if not url: print('no model', spec); continue
        save_png(spec.replace(':', '_') + ('_talk' if talk else '') + '.png', url)
    print('models', len(jobs))
    b.close()
