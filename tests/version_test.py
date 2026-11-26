# the game's BUILD_V, version.json, the README, the download copy and the newest CHANGELOG entry must all say the same version
import json, re
s = open('index.html').read(); v = re.search(r'BUILD_V = "([\d.]+)"', s).group(1)
assert json.load(open('version.json'))['v'] == v, 'version.json'
assert f'build {v}' in open('README.md').read() or f'**{v}**' in open('README.md').read(), 'README'
assert f'BUILD_V = "{v}"' in open('downloads/Ten Cells.html').read(), 'downloads copy'
assert re.search(r'CHANGELOG = \[\{"v": "%s"' % re.escape(v), s), 'CHANGELOG'
print('version OK', v)
