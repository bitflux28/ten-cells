// Ten Cells, native: raylib window + the web game's rules running in QuickJS (see js.cpp, js/api.js).
#include "raylib.h"
// our palette replaces raylib's colour macros
#undef BLUE
#undef GOLD
#undef GREEN
#undef RED
#undef WHITE
#include "audio.h"
#include "embed.h"
#include "js.h"
#include "json.h"
#include <algorithm>
#include <cmath>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <string>
#include <unordered_map>
#include <vector>

// ---------- palette (from the web game's CSS) ----------
static Color hexc(const char *h, unsigned char a = 255) {
    if (!h || h[0] != '#' || strlen(h) < 7) return {255, 255, 255, a};
    auto v = [&](int i) { char b[3] = {h[i], h[i + 1], 0}; return (unsigned char)strtol(b, nullptr, 16); };
    return {v(1), v(3), v(5), a};
}
static const Color INK = hexc("#0b0a1f"), BG = hexc("#12102e"), P1 = hexc("#1b1845"), P2 = hexc("#262056"), P3 = hexc("#3d3780"), SH = hexc("#110e30"),
                   TXT = hexc("#dfe6ff"), MIST = hexc("#a3a8d6"), FAINT = hexc("#6e72a8"), BLUE = hexc("#3466f0"), CYAN = hexc("#45c8f5"), GOLD = hexc("#ffcc4d"),
                   GREEN = hexc("#4fe89a"), RED = hexc("#ff4f73"), VIO = hexc("#8a63ff"), WHITE = {255, 255, 255, 255};
static Color withA(Color c, float a) { c.a = (unsigned char)std::max(0.f, std::min(255.f, a * 255)); return c; }
static Color mixc(Color a, Color b, float t) { return {(unsigned char)(a.r + (b.r - a.r) * t), (unsigned char)(a.g + (b.g - a.g) * t), (unsigned char)(a.b + (b.b - a.b) * t), (unsigned char)(a.a + (b.a - a.a) * t)}; }
static Color hsl(float h, float s, float l) {
    auto f = [&](float n) { float k = fmodf(n + h / 30.f, 12.f), a = s * std::min(l, 1 - l); return l - a * std::max(-1.f, std::min({k - 3, 9 - k, 1.f})); };
    return {(unsigned char)(f(0) * 255), (unsigned char)(f(8) * 255), (unsigned char)(f(4) * 255), 255};
}

// ---------- fonts and text ----------
static std::unordered_map<int, Font> FPX, FBOLD;
static Font &font(int size, bool bold) {
    auto &M = bold ? FBOLD : FPX; auto it = M.find(size); if (it != M.end()) return it->second;
    const unsigned char *d = bold ? EMB_font_bold : EMB_font_px; size_t n = bold ? EMB_font_bold_len : EMB_font_px_len;
    Font f = LoadFontFromMemory(".ttf", d, (int)n, size, EMB_codepoints, EMB_codepoint_count);
    SetTextureFilter(f.texture, TEXTURE_FILTER_BILINEAR); return M[size] = f;
}
// glyphs the pixel fonts don't have become plain text
static std::string clean(const std::string &s) {
    static const std::vector<std::pair<std::string, std::string>> R = {{"⁴", "^4"}, {"₀", "0"}, {"₁", "1"}, {"→", "->"}, {"∞", "inf"}, {"★", "*"}, {"☆", "-"}, {"…", "..."}, {"‘", "'"}, {"’", "'"}, {"“", "\""}, {"”", "\""}, {"›", ">"}, {"≥", ">="}, {"≤", "<="}};
    std::string o = s;
    for (auto &r : R) { size_t p = 0; while ((p = o.find(r.first, p)) != std::string::npos) { o.replace(p, r.first.size(), r.second); p += r.second.size(); } }
    return o;
}
static float tw(const std::string &s, int size, bool bold = false) { return MeasureTextEx(font(size, bold), s.c_str(), (float)size, 0).x; }
static void text(const std::string &s0, float x, float y, int size, Color c, bool bold = false, bool shadow = false) {
    std::string s = clean(s0);
    if (shadow) DrawTextEx(font(size, bold), s.c_str(), {roundf(x) + 2, roundf(y) + 2}, (float)size, 0, INK);
    DrawTextEx(font(size, bold), s.c_str(), {roundf(x), roundf(y)}, (float)size, 0, c);
}
static void textC(const std::string &s, float cx, float y, int size, Color c, bool bold = false, bool shadow = false) { text(s, cx - tw(clean(s), size, bold) / 2, y, size, c, bold, shadow); }
static std::unordered_map<std::string, std::vector<std::string>> WRAP;
static const std::vector<std::string> &wrap(const std::string &s0, float w, int size, bool bold = false) {
    std::string key = s0 + "\x01" + std::to_string((int)w) + ":" + std::to_string(size) + (bold ? "b" : "");
    auto it = WRAP.find(key); if (it != WRAP.end()) return it->second;
    if (WRAP.size() > 4000) WRAP.clear();
    std::string s = clean(s0); std::vector<std::string> lines; std::string cur;
    size_t i = 0;
    while (i <= s.size()) {
        size_t j = s.find_first_of(" \n", i); if (j == std::string::npos) j = s.size();
        std::string word = s.substr(i, j - i), cand = cur.empty() ? word : cur + " " + word;
        if (!cur.empty() && tw(cand, size, bold) > w) { lines.push_back(cur); cur = word; } else cur = cand;
        if (j < s.size() && s[j] == '\n') { lines.push_back(cur); cur.clear(); }
        i = j + 1;
    }
    if (!cur.empty()) lines.push_back(cur);
    return WRAP[key] = lines;
}
static float para(const std::string &s, float x, float y, float w, int size, Color c, bool center = false, bool bold = false) {
    const auto &L = wrap(s, w, size, bold); float lh = size * 1.32f;
    for (size_t k = 0; k < L.size(); k++) { if (center) textC(L[k], x + w / 2, y + k * lh, size, c, bold); else text(L[k], x, y + k * lh, size, c, bold); }
    return L.size() * lh;
}

// ---------- sprites ----------
static Texture2D SPR; static std::unordered_map<std::string, int> SPRI;
static void sprite(const std::string &name, float x, float y, float size, Color tint = WHITE) {
    auto it = SPRI.find(name); if (it == SPRI.end()) return;
    int k = it->second; Rectangle src{(float)(k % EMB_sprite_cols * EMB_sprite_w), (float)(k / EMB_sprite_cols * EMB_sprite_h), (float)EMB_sprite_w, (float)EMB_sprite_h};
    DrawTexturePro(SPR, src, {roundf(x), roundf(y), size, size}, {0, 0}, 0, tint);
}

// ---------- pixel panels and buttons ----------
static void panel(float x, float y, float w, float h, Color fill = P1, Color hi = P3, Color lo = SH) {
    x = roundf(x); y = roundf(y); w = roundf(w); h = roundf(h);
    DrawRectangle(x + 3, y, w - 6, h, fill); DrawRectangle(x, y + 3, w, h - 6, fill);   // cut corners like the web panels
    DrawRectangle(x + 3, y, w - 6, 3, hi); DrawRectangle(x, y + 3, 3, h - 6, hi);
    DrawRectangle(x + 3, y + h - 3, w - 6, 3, lo); DrawRectangle(x + w - 3, y + 3, 3, h - 6, lo);
}
struct Hit { Rectangle r; std::string act; bool hold = false; };
static std::vector<Hit> HITS; static std::string pressAct; static Vector2 pressPos; static bool dragging = false; static double pressT = 0, holdNext = 0; static bool held = false;
static void hit(Rectangle r, const std::string &act, bool hold = false) { if (!act.empty()) HITS.push_back({r, act, hold}); }
static bool hot(Rectangle r) { return CheckCollisionPointRec(GetMousePosition(), r); }
static void button(Rectangle r, const std::string &label, const std::string &st, const std::string &sub, const std::string &act, bool hold = false, Color col = BLUE) {
    bool down = !pressAct.empty() && pressAct == act && hot(r) && IsMouseButtonDown(MOUSE_BUTTON_LEFT) && !dragging;
    float y = r.y + (down ? 2 : 0);
    Color fill = P2, hi = P3, lo = SH, tc = FAINT;
    if (st == "on") { fill = col; hi = mixc(col, WHITE, 0.45f); lo = mixc(col, INK, 0.45f); tc = WHITE; }
    else if (st == "done") { fill = hexc("#17543f"); hi = hexc("#1f6b51"); lo = hexc("#0f3a2b"); tc = GREEN; }
    else if (st == "bad") { fill = RED; hi = hexc("#ffb3c3"); lo = hexc("#8f1d3c"); tc = WHITE; }
    if (st == "on" && hot(r) && !down) fill = mixc(fill, WHITE, 0.08f);
    panel(r.x, y, r.width, r.height, fill, hi, lo);
    bool shadow = st == "on" || st == "bad";
    if (!sub.empty()) { textC(sub, r.x + r.width / 2, y + r.height / 2 - 15, 12, withA(tc, 0.85f)); textC(label, r.x + r.width / 2, y + r.height / 2 - 1, 16, tc, false, shadow); }
    else textC(label, r.x + r.width / 2, y + r.height / 2 - 8, 16, tc, false, shadow);
    if (st != "done" && st != "off") hit(r, act, hold);
    else if (st == "off" && !act.empty()) hit(r, "!" + act, hold);  // off buttons still answer (with the 'bad' sound)
}

// ---------- state ----------
static std::string saveDir, tab = "engine", subUpg = "tree", cat = "gen", subPre = "reb", subSta = "stats";
static J VIEW; static double lastView = -1, lastSave = 0;
static float scrollY = 0, scrollV = 0, contentH = 0;
static std::unordered_map<std::string, float> scrollMem;
struct Toast { std::string text; Color col; double t; };
static std::vector<Toast> TOASTS;
struct Flash { std::string title, sub; Color col; double t = -100; } FLASH;
struct Confirm { bool open = false; std::string title, text, yes, label; bool bad = false; } CONFIRM;
struct Welcome { bool open = false; std::string text; } WELCOME;
struct Particle { float x, y, vx, vy, life, max, size; Color c; };
static std::vector<Particle> PARTS;
struct Float { float x, y, t; std::string s; Color c; };
static std::vector<Float> FLOATS;
static bool sfxOn = true, fxOn = true;
static float rxHit = 0, rxSpin = 0;
static Rectangle RX{0, 0, 0, 0};

static void toast(const std::string &s, Color c = CYAN) { TOASTS.push_back({s, c, GetTime()}); if (TOASTS.size() > 4) TOASTS.erase(TOASTS.begin()); }
static void refresh() {
    std::string sub = tab == "upgrades" ? subUpg : tab == "prestige" ? subPre : subSta;
    double t0 = GetTime();
    std::string s = js::call("view", js::quote(tab) + "," + js::quote(sub) + "," + js::quote(cat));
    if (getenv("TC_PROF")) TraceLog(LOG_WARNING, "view %s/%s %.2f ms, %d bytes", tab.c_str(), sub.c_str(), (GetTime() - t0) * 1000, (int)s.size());
    if (!s.empty()) VIEW = J::parse(s);
    lastView = GetTime();
    J set = J::parse(js::call("settings")); if (set.t == J::Obj) { sfxOn = set["sfx"].truthy(); fxOn = set["fx"].truthy(); }
    audio::enable(sfxOn);
}
static void go(const std::string &t) { scrollMem[tab] = scrollY; tab = t; scrollY = scrollMem[t]; scrollV = 0; refresh(); }
static void burst(float x, float y, Color c, int n, float sp = 140) {
    if (!fxOn) return;
    for (int k = 0; k < n; k++) { float a = GetRandomValue(0, 6283) / 1000.f, v = sp * (0.3f + GetRandomValue(0, 1000) / 1000.f); PARTS.push_back({x, y, cosf(a) * v, sinf(a) * v - 30, 0, 0.5f + GetRandomValue(0, 500) / 1000.f, (float)GetRandomValue(2, 4), c}); }
    if (PARTS.size() > 600) PARTS.erase(PARTS.begin(), PARTS.begin() + (PARTS.size() - 600));
}

// results of an action: sounds, notes, confirm dialogs, the prestige banner, tab changes, clipboard
static void handle(const std::string &res) {
    J r = J::parse(res); if (r.t != J::Obj) return;
    if (r.has("sfx")) { std::string s = r["sfx"].str(); audio::play(s == "prestige" ? "prestige" : s, s == "buy" ? 0.95f + GetRandomValue(0, 10) / 100.f : 1.f); }
    if (r.has("toast")) toast(r["toast"].str(), r["sfx"].str() == "bad" ? RED : CYAN);
    if (r.has("clip")) SetClipboardText(r["clip"].str().c_str());
    if (r.has("confirm")) { const J &c = r["confirm"]; CONFIRM = {true, c["title"].str(), c["text"].str(), c["yes"].str(), c["label"].str(), c["bad"].truthy()}; }
    if (r.has("flash")) { FLASH = {r["flash"].str(), r["sub"].str(), hexc(r["col"].str("#ffffff").c_str()), GetTime()}; burst(GetScreenWidth() / 2.f, GetScreenHeight() * 0.4f, FLASH.col, 120, 320); }
    if (r.has("go")) go(r["go"].str());
}
static void act(const std::string &a0) {
    if (a0.empty()) return;
    if (a0[0] == '!') { audio::play("bad"); return; }
    std::string a = a0;
    // navigation stays on this side
    if (a.rfind("sub:upg:", 0) == 0) { subUpg = a.substr(8); scrollY = 0; audio::play("ui"); refresh(); return; }
    if (a.rfind("sub:pre:", 0) == 0) { subPre = a.substr(8); scrollY = 0; audio::play("ui"); refresh(); return; }
    if (a.rfind("sub:sta:", 0) == 0) { subSta = a.substr(8); scrollY = 0; audio::play("ui"); refresh(); return; }
    if (a.rfind("cat:", 0) == 0) { cat = a.substr(4); scrollY = 0; audio::play("ui"); refresh(); return; }
    if (a.rfind("nav:", 0) == 0) { go(a.substr(4)); audio::play("ui"); return; }
    if (a == "folder") { OpenURL(saveDir.c_str()); return; }
    std::string arg = "\"\"";
    if (a == "import" || a.rfind("at:", 0) == 0) { const char *c = GetClipboardText(); arg = js::quote(c ? c : ""); }
    handle(js::call("act", js::quote(a) + "," + arg));
    refresh();
}

// ---------- background: a dithered nebula and three layers of twinkling stars ----------
static Texture2D NEB; static int nebW = 0, nebH = 0;
struct Star { float x, y, d, t; Color c; };
static std::vector<Star> STARS;
static void buildSky(int w, int h) {
    nebW = w / 3 + 1; nebH = h / 3 + 1;
    Image im = GenImageColor(nebW, nebH, BG);
    const int B[4][4] = {{0, 8, 2, 10}, {12, 4, 14, 6}, {3, 11, 1, 9}, {15, 7, 13, 5}};
    const Color PAL[3][4] = {{hexc("#1a1544"), hexc("#2a1f66"), hexc("#3d2a8f"), hexc("#5a3fbf")}, {hexc("#131a48"), hexc("#1c2a70"), hexc("#25409b"), hexc("#3466c8")}, {hexc("#2a1446"), hexc("#45206a"), hexc("#6a2a8f"), hexc("#8a3fb0")}};
    struct Blob { float x, y, r, w; int h; }; std::vector<Blob> bl;
    srand(7);
    for (int k = 0; k < 26; k++) { float u = rand() / (float)RAND_MAX; bl.push_back({nebW * (u * 1.2f - 0.1f) + (rand() % 100 - 50) / 300.f * nebW, nebH * (0.95f - u * 0.85f) + (rand() % 100 - 50) / 350.f * nebH, (0.06f + (rand() % 100) / 700.f) * std::max(nebW, nebH), 0.4f + (rand() % 100) / 200.f, rand() % 100 < 55 ? 0 : rand() % 100 < 60 ? 1 : 2}); }
    for (int y = 0; y < nebH; y++) for (int x = 0; x < nebW; x++) {
        float dens = 0, best = 0; int bh = 0;
        for (auto &b : bl) { float v = b.w * expf(-((x - b.x) * (x - b.x) + (y - b.y) * (y - b.y)) / (b.r * b.r)); dens += v; if (v > best) { best = v; bh = b.h; } }
        int L = std::min(4, (int)floorf(dens * 3.2f + B[y & 3][x & 3] / 16.f - 0.5f)); if (L < 1) continue;
        ImageDrawPixel(&im, x, y, PAL[bh][L - 1]);
    }
    if (NEB.id) UnloadTexture(NEB);
    NEB = LoadTextureFromImage(im); SetTextureFilter(NEB, TEXTURE_FILTER_POINT); UnloadImage(im);
    STARS.clear(); const Color SC[5] = {WHITE, hexc("#c4f3ff"), hexc("#9ab8ff"), hexc("#fff1b0"), hexc("#cdb8ff")};
    for (int L = 0; L < 3; L++) { int n = (int)(w * h / (L == 0 ? 1800.f : L == 1 ? 5400.f : 16000.f)); for (int k = 0; k < n; k++) STARS.push_back({(float)(rand() % w), (float)(rand() % h), L == 0 ? 0.12f : L == 1 ? 0.35f : 0.75f, (rand() % 600) / 100.f, SC[rand() % 5]}); }
}
static void drawSky(float t) {
    int w = GetScreenWidth(), h = GetScreenHeight();
    ClearBackground(INK);
    DrawTexturePro(NEB, {0, 0, (float)nebW, (float)nebH}, {roundf(sinf(t * 0.03f) * 6), 0, nebW * 3.f, nebH * 3.f}, {0, 0}, 0, WHITE);
    float mv = fxOn ? 1 : 0;
    for (auto &s : STARS) {
        float y = fmodf(s.y + t * 5 * s.d * mv + scrollY * 0.05f * s.d, (float)h); if (y < 0) y += h;
        float tw_ = sinf(t * (1 + s.d) + s.t); if (tw_ < -0.7f && mv) continue;
        Color c = tw_ > 0.6f ? WHITE : s.d < 0.2f ? hexc("#4f48a3") : s.c;
        int X = ((int)s.x / 3) * 3, Y = ((int)y / 3) * 3;
        DrawRectangle(X, Y, 3, 3, c);
        if (s.d > 0.5f && tw_ > 0.8f) { DrawRectangle(X - 3, Y, 9, 3, withA(c, 0.35f)); DrawRectangle(X, Y - 3, 3, 9, withA(c, 0.35f)); }
    }
}

// ---------- the reactor: a pixel atom that spins faster as your engine grows ----------
static void drawReactor(Rectangle r, float t, const J &b) {
    float cx = r.x + r.width / 2, cy = r.y + 108, eLog = (float)VIEW["hud"]["eLog"].num();
    float spd = 1 + std::min(4.f, eLog / 25.f); if (fxOn) rxSpin += GetFrameTime() * spd * (1 + rxHit * 3);
    rxHit *= 0.86f;
    float s = 1 + rxHit * 0.08f, R = 62 * s;
    for (int k = 6; k >= 1; k--) DrawCircle((int)cx, (int)cy, R + k * 9 + sinf(t * 2) * 3, withA(VIO, 0.035f * k));
    // three orbits and their electrons
    for (int o = 0; o < 3; o++) {
        float rot = o * 1.047f + rxSpin * 0.15f, ca = cosf(rot), sa = sinf(rot);
        for (int i = 0; i < 90; i++) { float a = i / 90.f * 6.283f, ex = cosf(a) * R * 1.25f, ey = sinf(a) * R * 0.45f;
            int X = (int)(cx + ex * ca - ey * sa), Y = (int)(cy + ex * sa + ey * ca); DrawRectangle(X / 2 * 2, Y / 2 * 2, 2, 2, withA(o == 0 ? CYAN : o == 1 ? VIO : hexc("#ff5fd0"), 0.7f)); }
        float a = rxSpin * (1.3f + o * 0.4f) + o * 2, ex = cosf(a) * R * 1.25f, ey = sinf(a) * R * 0.45f;
        DrawCircle((int)(cx + ex * ca - ey * sa), (int)(cy + ex * sa + ey * ca), 5, WHITE);
    }
    // the core, shaded in rings, with a highlight
    const Color CR[5] = {hexc("#1d3492"), hexc("#3466f0"), hexc("#45c8f5"), hexc("#c4f3ff"), WHITE};
    for (int k = 0; k < 5; k++) DrawCircle((int)(cx - k * 2.2f), (int)(cy - k * 2.2f), R * 0.52f * (1 - k * 0.16f), CR[k]);
    if (rxHit > 0.05f) DrawCircleLines((int)cx, (int)cy, R * (1.1f + (1 - rxHit) * 0.8f), withA(WHITE, rxHit));
    std::string info = b["notap"].truthy() ? "Tapping is disabled right now" : "Tap the reactor · +" + b["tap"].str() + " per tap · " + std::to_string((int)b["crit"].num()) + "% crit";
    textC(info, cx, r.y + r.height - 22, 14, b["notap"].truthy() ? RED : MIST);
}

// ---------- page blocks ----------
static float drawBlocks(const J &page, float x, float y0, float w, float t) {
    float y = y0;
    for (const J &b : page.a) {
        std::string ty = b["t"].str();
        if (ty == "reactor") { RX = {x, y, w, 250}; drawReactor(RX, t, b); y += 258; }
        else if (ty == "h") { text(b["text"].str(), x + 2, y + 6, 18, GOLD, true, true); y += 32; }
        else if (ty == "p") { float hh = para(b["text"].str(), x + 4, y, w - 8, 14, b["muted"].truthy() ? hexc("#8a8fc4") : MIST, b["center"].truthy()); y += hh + 8; }
        else if (ty == "chips") {
            float cx_ = x, cy_ = y; std::string base = b["act"].str();
            for (const J &c : b["items"].a) {
                std::string key = c[0].str(), lab = c[1].str(); bool lock = c[2].truthy(), sel = c[3].truthy();
                float cw = tw(lab, 14) + 22 + (lock ? 16 : 0); if (cx_ + cw > x + w) { cx_ = x; cy_ += 36; }
                Rectangle r{cx_, cy_, cw, 32};
                Color col = c[4].t == J::Str ? hexc(c[4].str().c_str()) : VIO;
                panel(r.x, r.y, r.width, r.height, sel ? withA(col, 1) : P2, sel ? mixc(col, WHITE, 0.4f) : P3, sel ? mixc(col, INK, 0.4f) : SH);
                if (lock) sprite("lock", r.x + 8, r.y + 9, 12);
                text(lab, r.x + 11 + (lock ? 14 : 0), r.y + 8, 14, lock ? FAINT : sel ? WHITE : MIST, false, sel);
                if (!lock && !sel) hit(r, base + ":" + key);
                cx_ += cw + 6;
            }
            y = cy_ + 42;
        }
        else if (ty == "btns") {
            if (b.has("title")) { text(b["title"].str(), x + 4, y, 14, MIST); y += 22; }
            size_t n = b["items"].size(); float bw = (w - (n - 1) * 6) / std::max<size_t>(1, n);
            for (size_t k = 0; k < n; k++) { const J &it = b["items"][k]; button({x + k * (bw + 6), y, bw, it["sub"].str().empty() ? 40.f : 48.f}, it["label"].str(), it["st"].str(), it["sub"].str(), it["act"].str()); }
            y += (b["items"][0]["sub"].str().empty() ? 40 : 48) + 10;
        }
        else if (ty == "big") { Color col = hexc(b["col"].str("#3466f0").c_str()); button({x, y, w, 56}, b["label"].str(), b["st"].str(), "", b["act"].str(), false, col); y += 64; }
        else if (ty == "toggle") {
            float tx = x + 14, tw_ = w - 100; float hh = 18 + (b["desc"].str().empty() ? 0 : wrap(b["desc"].str(), tw_, 13).size() * 17.f) + 18;
            panel(x, y, w, hh);
            text(b["title"].str(), tx, y + 12, 16, TXT);
            if (!b["desc"].str().empty()) para(b["desc"].str(), tx, y + 32, tw_, 13, MIST);
            Rectangle sw{x + w - 70, y + hh / 2 - 13, 56, 26}; bool on = b["on"].truthy();
            panel(sw.x, sw.y, sw.width, sw.height, on ? hexc("#17543f") : P2);
            DrawRectangle((int)(on ? sw.x + 32 : sw.x + 6), (int)sw.y + 5, 18, 16, on ? GREEN : FAINT);
            hit({x, y, w, hh}, b["act"].str()); y += hh + 6;
        }
        else if (ty == "row") {
            const J &bt = b["btn"]; bool hasB = bt.t == J::Obj; float bw = hasB ? 112 : 0, tx = x + 14 + (b.has("icon") ? 34 : 0), tw_ = w - (tx - x) - bw - (hasB ? 24 : 14);
            float hh = 14 + 20 + wrap(b["desc"].str(), tw_, 13).size() * 17.f + (b["note"].str().empty() ? 0 : wrap(b["note"].str(), tw_, 13).size() * 17.f + 2) + 12;
            hh = std::max(hh, hasB ? 68.f : 52.f);
            bool dim = b["dim"].truthy();
            panel(x, y, w, hh, dim ? hexc("#161339") : P1);
            if (b.has("icon")) sprite(b["icon"].str(), x + 12, y + 14, 24, dim ? withA(WHITE, 0.6f) : WHITE);
            text(b["title"].str(), tx, y + 12, 16, dim ? FAINT : TXT);
            if (b.has("tag")) { float tl = tx + tw(clean(b["title"].str()), 16) + 8; std::string tg = b["tag"].str(); panel(tl, y + 11, tw(tg, 12) + 12, 20, P2); text(tg, tl + 6, y + 14, 12, CYAN); }
            float dy = y + 34 + para(b["desc"].str(), tx, y + 34, tw_, 13, dim ? FAINT : MIST);
            if (!b["note"].str().empty()) para(b["note"].str(), tx, dy + 2, tw_, 13, b["good"].truthy() ? GREEN : hexc("#ff8c9f"));
            if (hasB) button({x + w - bw - 10, y + hh / 2 - 25, bw, 50}, bt["label"].str(), bt["st"].str(), bt["sub"].str(), bt["act"].str());
            y += hh + 6;
        }
        else if (ty == "gen") {
            float hh = 92; Color hue = hsl((float)b["hue"].num(), 0.7f, 0.55f); bool ban = b["ban"].truthy();
            panel(x, y, w, hh, ban ? hexc("#161339") : P1);
            panel(x + 8, y + 10, 62, 62, mixc(P2, hue, 0.25f), mixc(P3, hue, 0.35f), SH);
            sprite(b["spr"].str(), x + 15, y + 15, 48);
            text("T" + std::to_string((int)b["i"].num() + 1), x + 12, y + 60, 12, GOLD, false, true);
            float tx = x + 80, bw = 112, tw_ = w - 80 - bw - 20;
            text(b["n"].str(), tx, y + 10, 16, ban ? FAINT : TXT);
            std::string own = b["own"].str(); text(own, tx + tw_ - tw(own, 18), y + 8, 18, CYAN);
            text(b["prod"].str(), tx, y + 32, 13, MIST);
            float seg = (tw_ - 9) / 10.f, m = (float)b["mile"].num();
            for (int k = 0; k < 10; k++) DrawRectangle((int)(tx + k * (seg + 1)), (int)y + 52, (int)seg, 6, m * 10 >= k + 1 - 0.01f ? GOLD : P3);
            text(b["mileT"].str(), tx, y + 64, 12, FAINT);
            const J &bt = b["btn"]; button({x + w - bw - 10, y + 20, bw, 52}, bt["label"].str(), bt["st"].str(), bt["sub"].str(), bt["act"].str(), true);
            y += hh + 6;
        }
        else if (ty == "layer") {
            Color col = hexc(b["col"].str().c_str()); float hh = 130, cx = x + w / 2;
            for (int k = 5; k >= 1; k--) DrawCircle((int)cx, (int)(y + 50), 34.f + k * 7 + sinf(t * 2 + k) * 2, withA(col, 0.05f * k));
            DrawCircleLines((int)cx, (int)(y + 50), 40 + sinf(t * 1.5f) * 2, withA(col, 0.8f));
            for (int k = 0; k < 16; k++) { float a = t * 0.6f + k / 16.f * 6.283f; DrawRectangle((int)(cx + cosf(a) * 52) - 1, (int)(y + 50 + sinf(a) * 52) - 1, 3, 3, k % 4 ? withA(col, 0.6f) : WHITE); }
            sprite(b["spr"].str(), cx - 24, y + 26, 48);
            textC(b["amt"].str() + " " + b["cur"].str(), cx, y + 96, 20, WHITE, true, true);
            textC("production ×" + b["bonus"].str(), cx, y + hh - 12, 13, MIST);
            y += hh + 14;
        }
        else if (ty == "kv" && b["items"].size()) {
            float hh = 14 + b["items"].size() * 26.f; panel(x, y, w, hh);
            float ry = y + 9;
            for (const J &it : b["items"].a) { text(it[0].str(), x + 14, ry, 14, MIST); std::string v = it[1].str(); text(v, x + w - 14 - tw(v, 14), ry, 14, TXT); ry += 26; }
            y += hh + 8;
        }
    }
    return y - y0;
}

// ---------- HUD, nav and overlays ----------
static float drawHud(float t) {
    const J &h = VIEW["hud"]; int W = GetScreenWidth(); float x = 8, w = W - 16, y = 8;
    std::vector<std::pair<std::string, std::string>> curs; for (const J &c : h["curs"].a) curs.push_back({c[0].str(), c[1].str()});
    // currency chips wrap onto as many rows as they need
    float cx = x + 12, rows = curs.empty() ? 0 : 1; for (auto &c : curs) { float cw = 30 + tw(c.second, 14) + 12; if (cx + cw > x + w - 12) { rows++; cx = x + 12; } cx += cw + 6; }
    float hh = 92 + rows * 32 + (h["buffs"].size() ? 28 : 0);
    panel(x, y, w, hh);
    DrawRectangle((int)x + 14, (int)y + 15, 6, 6, fmodf(t, 1.2f) < 0.6f ? GREEN : withA(GREEN, 0.3f));
    text("Ten Cells", x + 26, y + 10, 14, MIST);
    Rectangle mute{x + w - 44, y + 8, 34, 30}; panel(mute.x, mute.y, mute.width, mute.height, P2); sprite(sfxOn && VIEW["hud"].t != J::Null ? "speaker" : "mute", mute.x + 5, mute.y + 3, 24); hit(mute, "tog:set:sfx");
    text(h["e"].str("0"), x + 12, y + 28, 38, WHITE, false, true);
    text(h["eps"].str("0"), x + 14, y + 70, 14, CYAN); text("energy per second", x + 18 + tw(h["eps"].str("0"), 14), y + 70, 14, MIST);
    cx = x + 12; float cy = y + 94;
    for (auto &c : curs) { float cw = 30 + tw(c.second, 14) + 12; if (cx + cw > x + w - 12) { cx = x + 12; cy += 32; } panel(cx, cy, cw, 28, P2); sprite(c.first, cx + 4, cy + 2, 24); text(c.second, cx + 32, cy + 6, 14, TXT); cx += cw + 6; }
    if (h["buffs"].size()) { float bx = x + 12, by = y + hh - 30; for (const J &bf : h["buffs"].a) { std::string s = bf[0].str() + " · " + std::to_string((int)bf[1].num()) + "s"; float bw = tw(s, 12) + 14; panel(bx, by, bw, 22, hexc("#3a1f5a")); text(s, bx + 7, by + 4, 12, GOLD); bx += bw + 6; } }
    float yy = y + hh + 6;
    if (!h["chal"].str().empty()) { panel(x, yy, w, 30, hexc("#3a1426")); sprite("swords", x + 8, yy + 3, 24); text(h["chal"].str(), x + 38, yy + 8, 13, hexc("#ffb3c3")); yy += 36; }
    return yy;
}
static void drawNav(float t) {
    int W = GetScreenWidth(), H = GetScreenHeight(); float y = H - 64;
    DrawRectangle(0, (int)y, W, 64, P1); DrawRectangle(0, (int)y, W, 3, INK); DrawRectangle(0, (int)y + 3, W, 3, P3);
    const char *T[4][3] = {{"engine", "Engine", "core"}, {"upgrades", "Upgrades", "tree"}, {"prestige", "Prestige", "star"}, {"stats", "Stats", "bars"}};
    float bw = W / 4.f;
    for (int k = 0; k < 4; k++) {
        Rectangle r{k * bw, y + 4, bw, 58}; bool sel = tab == T[k][0];
        if (sel) DrawRectangle((int)(r.x + 8), (int)r.y + 4, (int)r.width - 16, 50, P2);
        sprite(T[k][2], r.x + bw / 2 - 12, r.y + 8, 24, sel ? WHITE : withA(WHITE, 0.55f));
        textC(T[k][1], r.x + bw / 2, r.y + 36, 13, sel ? WHITE : FAINT);
        if (VIEW["dots"][T[k][0]].truthy() && !sel) DrawRectangle((int)(r.x + bw / 2 + 12), (int)r.y + 8, 6, 6, fmodf(t, 1.4f) < 0.9f ? GOLD : withA(GOLD, 0.4f));
        if (!sel) hit(r, std::string("nav:") + T[k][0]);
    }
}
static void drawOverlays(float t) {
    int W = GetScreenWidth(), H = GetScreenHeight();
    // particles and floating tap numbers
    float dt = GetFrameTime();
    for (auto &p : PARTS) { p.life += dt; p.x += p.vx * dt; p.y += p.vy * dt; p.vy += 60 * dt; float a = 1 - p.life / p.max; if (a > 0) DrawRectangle((int)p.x, (int)p.y, (int)p.size, (int)p.size, withA(p.c, a)); }
    PARTS.erase(std::remove_if(PARTS.begin(), PARTS.end(), [](const Particle &p) { return p.life >= p.max; }), PARTS.end());
    for (auto &f : FLOATS) { f.t += dt; f.y -= 50 * dt; float a = 1 - f.t / 1.1f; if (a > 0) text(f.s, f.x - tw(f.s, 18) / 2, f.y, 18, withA(f.c, a), false, true); }
    FLOATS.erase(std::remove_if(FLOATS.begin(), FLOATS.end(), [](const Float &f) { return f.t > 1.1f; }), FLOATS.end());
    // event orb
    const J &ev = VIEW["hud"]["ev"];
    if (ev.t == J::Obj && !CONFIRM.open) {
        Color c = hexc(ev["col"].str().c_str()); float r = 22 + sinf(t * 4) * 2, ox = W - 40, oy = H * 0.42f + sinf(t * 1.7f) * 12;
        for (int k = 4; k >= 1; k--) DrawCircle((int)ox, (int)oy, r + k * 6, withA(c, 0.08f * k));
        DrawCircle((int)ox, (int)oy, r, c); DrawCircle((int)(ox - 8), (int)(oy - 8), r * 0.35f, withA(WHITE, 0.6f));
        textC(ev["n"].str(), ox, oy + r + 8, 13, WHITE, false, true);
        hit({ox - r - 10, oy - r - 10, 2 * r + 20, 2 * r + 34}, "claim");
    }
    // notes
    float ty = H - 64 - 10;
    for (int k = (int)TOASTS.size() - 1; k >= 0; k--) {
        auto &q = TOASTS[k]; float age = (float)(GetTime() - q.t), a = std::min(1.f, std::min(age * 6, (3.6f - age) * 3)); if (a <= 0) continue;
        float tw_ = std::min((float)W - 40, tw(clean(q.text), 14) + 30), hh = wrap(q.text, tw_ - 28, 14).size() * 18.f + 16, tx = W / 2.f - tw_ / 2;
        ty -= hh + (age < 0.15f ? (0.15f - age) * -60 : 0);
        panel(tx, ty, tw_, hh, withA(hexc("#221d55"), a), withA(q.col, a), withA(SH, a));
        para(q.text, tx + 14, ty + 8, tw_ - 28, 14, withA(TXT, a)); ty -= 6;
    }
    TOASTS.erase(std::remove_if(TOASTS.begin(), TOASTS.end(), [](const Toast &q) { return GetTime() - q.t > 3.6; }), TOASTS.end());
    // the prestige banner
    float fa = (float)(GetTime() - FLASH.t);
    if (fa < 2.4f) { float a = std::min(1.f, std::min(fa * 5, (2.4f - fa) * 2.5f)), by = H * 0.36f;
        DrawRectangle(0, (int)by, W, 110, withA(INK, 0.85f * a)); DrawRectangle(0, (int)by, W, 3, withA(FLASH.col, a)); DrawRectangle(0, (int)by + 107, W, 3, withA(FLASH.col, a));
        textC(FLASH.title, W / 2.f, by + 18, 40, withA(WHITE, a), true, true); textC(FLASH.sub, W / 2.f, by + 70, 18, withA(FLASH.col, a)); }
    // dialogs
    auto dialog = [&](const std::string &title, const std::string &body, const std::string &yesLabel, const std::string &yesAct, bool bad, const std::string &noLabel) {
        DrawRectangle(0, 0, W, H, withA(INK, 0.7f)); HITS.clear();
        float w = std::min(380.f, W - 32.f), x = W / 2.f - w / 2, th = wrap(body, w - 36, 15).size() * 20.f, h = 100 + th + 60, y = H / 2.f - h / 2;
        panel(x, y, w, h); textC(title, W / 2.f, y + 18, 24, WHITE, true, true); para(body, x + 18, y + 58, w - 36, 15, MIST, true);
        float bw = noLabel.empty() ? w - 36 : (w - 42) / 2;
        if (!noLabel.empty()) button({x + 18, y + h - 64, bw, 46}, noLabel, "on", "", "cancel", false, P3);
        button({x + w - 18 - bw, y + h - 64, bw, 46}, yesLabel, bad ? "bad" : "on", "", yesAct);
    };
    if (CONFIRM.open) dialog(CONFIRM.title, CONFIRM.text, CONFIRM.label, "yes", CONFIRM.bad, "Cancel");
    else if (WELCOME.open) dialog("Welcome back", WELCOME.text, "Nice", "welcome", false, "");
}

// ---------- input ----------
static void input(float pageTop, float pageBottom) {
    Vector2 m = GetMousePosition(); bool inPage = m.y > pageTop && m.y < pageBottom;
    float wheel = GetMouseWheelMove(); if (wheel != 0 && inPage && !CONFIRM.open && !WELCOME.open) scrollV -= wheel * 900;
    if (IsMouseButtonPressed(MOUSE_BUTTON_LEFT)) {
        pressPos = m; dragging = false; pressAct.clear(); pressT = GetTime(); holdNext = pressT + 0.4; held = false;
        for (auto it = HITS.rbegin(); it != HITS.rend(); ++it) if (CheckCollisionPointRec(m, it->r)) { pressAct = it->act; break; }
        // the reactor fires on press so fast tapping feels right
        if (pressAct.empty() && !CONFIRM.open && !WELCOME.open && tab == "engine" && CheckCollisionPointRec(m, RX)) {
            J r = J::parse(js::call("act", "\"tap\",\"\""));
            if (r.has("tap")) { rxHit = 1; bool crit = r["crit"].truthy(); FLOATS.push_back({m.x + GetRandomValue(-20, 20), m.y - 10, 0, (crit ? "CRIT +" : "+") + r["tap"].str(), crit ? GOLD : WHITE}); burst(m.x, m.y, crit ? GOLD : CYAN, crit ? 26 : 10); audio::play(crit ? "crit" : "tap"); }
            else audio::play("bad");
            pressAct = "~tap";
        }
    }
    if (IsMouseButtonDown(MOUSE_BUTTON_LEFT)) {
        Vector2 d = GetMouseDelta();
        if (!dragging && fabsf(m.y - pressPos.y) > 8 && pressAct != "~tap" && inPage && !CONFIRM.open && !WELCOME.open) { dragging = true; }
        if (dragging) { scrollY -= d.y; scrollV = -d.y / std::max(GetFrameTime(), 0.001f) * 0.6f; }
        // hold a buy button to keep buying
        Hit *hh = nullptr; for (auto &h : HITS) if (h.act == pressAct && h.hold && CheckCollisionPointRec(m, h.r)) hh = &h;
        if (hh && !dragging && GetTime() > holdNext) { act(pressAct); held = true; holdNext = GetTime() + 0.09; }
    }
    if (IsMouseButtonReleased(MOUSE_BUTTON_LEFT)) {
        if (!dragging && !held && !pressAct.empty() && pressAct != "~tap") {
            bool still = false; for (auto &h : HITS) if (h.act == pressAct && CheckCollisionPointRec(m, h.r)) still = true;
            if (still) {
                if (pressAct == "cancel") { CONFIRM.open = false; audio::play("ui"); }
                else if (pressAct == "yes") { CONFIRM.open = false; act(CONFIRM.yes); }
                else if (pressAct == "welcome") { WELCOME.open = false; audio::play("claim"); }
                else act(pressAct);
            }
        }
        pressAct.clear(); dragging = false;
    }
    // keys: space taps, 1-4 switch tabs, M mutes, F11 full screen
    if (!CONFIRM.open && !WELCOME.open) {
        if (IsKeyPressed(KEY_SPACE) && tab == "engine") { J r = J::parse(js::call("act", "\"tap\",\"\"")); if (r.has("tap")) { rxHit = 1; burst(RX.x + RX.width / 2, RX.y + 108, CYAN, 10); audio::play(r["crit"].truthy() ? "crit" : "tap"); } }
        const char *tabs[4] = {"engine", "upgrades", "prestige", "stats"};
        for (int k = 0; k < 4; k++) if (IsKeyPressed(KEY_ONE + k) && tab != tabs[k]) go(tabs[k]);
        if (IsKeyPressed(KEY_M)) act("tog:set:sfx");
        if (IsKeyPressed(KEY_G)) act("buyall");
    } else if (IsKeyPressed(KEY_ESCAPE)) { CONFIRM.open = false; WELCOME.open = false; }
    if (IsKeyPressed(KEY_F11)) ToggleBorderlessWindowed();
}

static std::string dataDir() {
#ifdef TC_WINDOWS
    // wide getenv so a user name with accents or non-Latin letters still works; kept as UTF-8 from here on
    const wchar_t *a = _wgetenv(L"APPDATA"); return (std::filesystem::path(a ? a : L".") / L"Ten Cells").u8string();
#else
    const char *x = getenv("XDG_DATA_HOME"), *h = getenv("HOME"); return x ? std::string(x) + "/tencells" : std::string(h ? h : ".") + "/.local/share/tencells";
#endif
}

int main(int argc, char **argv) {
    // test hooks: --data <dir> uses another save folder, --shots <prefix> saves screenshots of every tab and quits
    std::string shots; for (int k = 1; k + 1 < argc; k++) { if (!strcmp(argv[k], "--data")) saveDir = argv[k + 1]; if (!strcmp(argv[k], "--shots")) shots = argv[k + 1]; }
    if (saveDir.empty()) saveDir = dataDir();
    SetConfigFlags(FLAG_WINDOW_RESIZABLE | FLAG_MSAA_4X_HINT | FLAG_VSYNC_HINT);
    InitWindow(460, 900, "Ten Cells");
    SetWindowMinSize(360, 600); SetExitKey(KEY_NULL); SetTargetFPS(60);
    { Image ic = LoadImageFromMemory(".png", EMB_icon_png, (int)EMB_icon_png_len); SetWindowIcon(ic); UnloadImage(ic); }
    { Image im = LoadImageFromMemory(".png", EMB_sprites_png, (int)EMB_sprites_png_len); SPR = LoadTextureFromImage(im); SetTextureFilter(SPR, TEXTURE_FILTER_POINT); UnloadImage(im); for (int k = 0; k < EMB_sprite_count; k++) SPRI[EMB_sprite_names[k]] = k; }
    audio::init();
    std::string err;
    if (!js::init(saveDir, err)) {
        while (!WindowShouldClose()) { BeginDrawing(); ClearBackground(INK); para("Ten Cells couldn't start its rules engine:\n" + err, 20, 20, GetScreenWidth() - 40, 14, RED); EndDrawing(); }
        CloseWindow(); return 1;
    }
    J boot = J::parse(js::call("boot"));
    if (boot["corrupt"].truthy()) toast(boot["ok"].truthy() ? "Your last save was damaged. Restored the backup." : "Your save could not be read. Started a new game.", GOLD);
    if (boot["off"].t == J::Obj) { const J &o = boot["off"]; WELCOME = {true, "You were away for " + o["away"].str() + (o["capped"].truthy() ? " (your engine ran for the first " + o["used"].str() + ")" : "") + ".\nYour engine made " + o["gained"].str() + " energy while you were gone."}; }
    refresh(); buildSky(GetScreenWidth(), GetScreenHeight());
    int lastW = GetScreenWidth(), lastH = GetScreenHeight(); double acc = 0;
    int frame = 0;

    while (!WindowShouldClose()) {
        double now = GetTime(); float dt = GetFrameTime();
        if (GetScreenWidth() != lastW || GetScreenHeight() != lastH) { lastW = GetScreenWidth(); lastH = GetScreenHeight(); buildSky(lastW, lastH); WRAP.clear(); }
        // rules: tick every 100 ms like the web game; a long stall (window dragged, laptop asleep) counts as offline time
        acc += dt;
        if (acc >= 0.1) {
            std::string off = js::call("step", std::to_string(acc)); acc = 0;
            if (!off.empty()) { J o = J::parse(off); WELCOME = {true, "You were away for " + o["away"].str() + ".\nYour engine made " + o["gained"].str() + " energy."}; }
            J ns = J::parse(js::call("notices"));
            int achs = 0; for (const J &q : ns.a) if (q["type"].str() == "ach" || q["type"].str() == "badge") achs++;
            if (achs > 2) { toast(std::to_string(achs) + " achievements and badges unlocked", GOLD); audio::play("ach"); }
            for (const J &q : ns.a) {
                std::string ty = q["type"].str(), tx = q["text"].str();
                if ((ty == "ach" || ty == "badge") && achs > 2) continue;
                Color c = ty == "ach" || ty == "badge" ? GOLD : ty == "bad" ? RED : ty == "boss" ? hexc("#ff9d5c") : ty == "chal" ? GREEN : CYAN;
                if (ty == "prestige") { toast("Auto " + tx, VIO); audio::play("claim"); continue; }
                if (ty == "chal") { FLASH = {"Challenge complete", tx.substr(tx.find(": ") == std::string::npos ? 0 : tx.find(": ") + 2), GREEN, GetTime()}; audio::play("win"); }
                if (ty == "beat") { FLASH = {"The End", "You finished the climb. Thank you for playing", hexc("#ff5fd0"), GetTime()}; audio::play("prestige"); }
                toast(ty == "ach" ? "Achievement: " + tx : tx, c);
                if (ty == "ach" || ty == "badge") audio::play("ach"); else if (ty == "event" || ty == "gain") audio::play("claim"); else if (ty == "bad") audio::play("bad");
            }
            refresh();
        }
        if (now - lastSave > 5) { js::call("save"); lastSave = now; }
        // settings mirror the save

        int W = GetScreenWidth(), H = GetScreenHeight(); float t = (float)now;
        // smooth scrolling with a little momentum
        if (!dragging) { scrollY += scrollV * dt; scrollV *= powf(0.02f, dt); if (fabsf(scrollV) < 5) scrollV = 0; }
        BeginDrawing();
        drawSky(t);
        HITS.clear();
        float top = drawHud(t), bottom = H - 64.f, maxW = std::min((float)W - 16, 560.f), px = W / 2.f - maxW / 2;
        float maxScroll = std::max(0.f, contentH - (bottom - top) + 16);
        scrollY = std::max(0.f, std::min(scrollY, maxScroll));
        BeginScissorMode(0, (int)top, W, (int)(bottom - top));
        size_t hits0 = HITS.size();
        contentH = drawBlocks(VIEW["page"], px, top + 4 - scrollY, maxW, t) + 12;
        // only what's on screen can be clicked
        for (size_t k = hits0; k < HITS.size(); k++) { Rectangle &r = HITS[k].r; float y1 = std::max(r.y, top), y2 = std::min(r.y + r.height, bottom); if (y2 <= y1) r.height = 0; else { r.height = y2 - y1; r.y = y1; } }
        if (RX.y + RX.height < top || RX.y > bottom || tab != "engine") RX = {0, 0, 0, 0};
        EndScissorMode();
        if (maxScroll > 0) { float bh = (bottom - top) * (bottom - top) / (contentH + 16), by = top + (scrollY / maxScroll) * (bottom - top - bh); DrawRectangle(W - 5, (int)by, 3, (int)bh, withA(P3, 0.9f)); }
        drawNav(t);
        drawOverlays(t);
        EndDrawing();
        input(top, bottom);
        if (!shots.empty()) {
            frame++;
            // every screen in turn: tab, sub page
            static const char *SH[][2] = {{"engine", ""}, {"upgrades", "tree"}, {"upgrades", "research"}, {"upgrades", "auto"}, {"prestige", "reb"}, {"prestige", "sing"}, {"prestige", "asc"}, {"prestige", "tra"}, {"prestige", "inf"}, {"prestige", "ult"}, {"prestige", "chal"}, {"stats", "stats"}, {"stats", "set"}};
            static int si = 0;
            if (frame % 40 == 20) { Image sc = LoadImageFromScreen(); ExportImage(sc, (shots + "-" + std::to_string(si) + ".png").c_str()); UnloadImage(sc); }
            if (frame % 40 == 39) { if (++si >= 13) break; std::string t2 = SH[si][0], s2 = SH[si][1]; if (t2 == "upgrades") subUpg = s2; if (t2 == "prestige") subPre = s2; if (t2 == "stats") subSta = s2; go(t2); }
        }
    }
    js::call("save");
    js::shutdown(); audio::shutdown(); CloseWindow();
    return 0;
}
