#define _USE_MATH_DEFINES
// Sound effects, synthesized once at startup (no files): soft blips and chimes in the spirit of the web game's synth.
#include "audio.h"
#include "raylib.h"
#include <cmath>
#include <cstdlib>
#include <map>
#include <string>
#include <vector>

namespace audio {
static std::map<std::string, std::vector<Sound>> S;
static bool on = true, ready = false;
static const int SR = 44100;

struct Note { float f, f2, at, dur, vol; int wave; }; // wave: 0 sine, 1 triangle, 2 square-ish, 3 noise
static Sound make(const std::vector<Note> &notes, float total) {
    int n = (int)(SR * total); std::vector<short> buf(n, 0); std::vector<float> mix(n, 0.f);
    for (const Note &q : notes) {
        int s0 = (int)(q.at * SR), len = (int)(q.dur * SR); double ph = 0;
        for (int k = 0; k < len && s0 + k < n; k++) {
            float t = (float)k / len, f = q.f + (q.f2 - q.f) * t;
            ph += 2 * M_PI * f / SR;
            float w = q.wave == 0 ? sinf((float)ph) : q.wave == 1 ? (float)(2 / M_PI * asin(sin(ph))) : q.wave == 2 ? (sinf((float)ph) > 0 ? 0.6f : -0.6f) : ((rand() / (float)RAND_MAX) * 2 - 1);
            float env = fminf(1.f, k / (0.004f * SR)) * powf(1 - t, 2.2f);
            mix[s0 + k] += w * env * q.vol;
        }
    }
    for (int k = 0; k < n; k++) buf[k] = (short)(fmaxf(-1.f, fminf(1.f, mix[k])) * 32000);
    Wave w{(unsigned)n, SR, 16, 1, buf.data()};
    return LoadSoundFromWave(w);
}

void init() {
    InitAudioDevice(); if (!IsAudioDeviceReady()) return;
    for (int k = 0; k < 4; k++) { float b = 520 + k * 45; S["tap"].push_back(make({{b, b * 1.25f, 0, 0.07f, 0.22f, 1}}, 0.08f)); }
    S["crit"].push_back(make({{700, 1400, 0, 0.12f, 0.25f, 1}, {1400, 1800, 0.05f, 0.1f, 0.15f, 0}}, 0.16f));
    S["buy"].push_back(make({{660, 660, 0, 0.06f, 0.2f, 1}, {990, 990, 0.05f, 0.08f, 0.18f, 1}}, 0.14f));
    S["upg"].push_back(make({{523, 523, 0, 0.09f, 0.2f, 0}, {659, 659, 0.06f, 0.09f, 0.2f, 0}, {784, 784, 0.12f, 0.16f, 0.22f, 0}}, 0.3f));
    S["ui"].push_back(make({{880, 900, 0, 0.035f, 0.12f, 1}}, 0.04f));
    S["bad"].push_back(make({{220, 160, 0, 0.14f, 0.22f, 2}}, 0.15f));
    S["claim"].push_back(make({{784, 784, 0, 0.08f, 0.2f, 0}, {1047, 1047, 0.07f, 0.12f, 0.2f, 0}, {1319, 1319, 0.14f, 0.18f, 0.18f, 0}}, 0.34f));
    S["ach"].push_back(make({{988, 988, 0, 0.08f, 0.16f, 0}, {1319, 1319, 0.08f, 0.2f, 0.16f, 0}}, 0.3f));
    S["ab"].push_back(make({{300, 900, 0, 0.25f, 0.22f, 1}, {600, 1800, 0.05f, 0.2f, 0.1f, 0}}, 0.3f));
    S["well"].push_back(make({{180, 60, 0, 0.5f, 0.3f, 0}, {0, 0, 0, 0.4f, 0.06f, 3}}, 0.5f));
    S["prestige"].push_back(make({{130, 520, 0, 0.9f, 0.25f, 1}, {196, 784, 0.1f, 0.9f, 0.18f, 0}, {262, 1047, 0.25f, 0.8f, 0.14f, 0}, {0, 0, 0, 0.6f, 0.05f, 3}}, 1.1f));
    S["win"].push_back(make({{523, 523, 0, 0.12f, 0.2f, 0}, {659, 659, 0.1f, 0.12f, 0.2f, 0}, {784, 784, 0.2f, 0.12f, 0.2f, 0}, {1047, 1047, 0.3f, 0.3f, 0.22f, 0}}, 0.62f));
    ready = true;
}
void play(const std::string &name, float pitch) {
    if (!ready || !on) return; auto it = S.find(name); if (it == S.end() || it->second.empty()) return;
    Sound &s = it->second[rand() % it->second.size()]; SetSoundPitch(s, pitch); PlaySound(s);
}
void enable(bool v) { on = v; }
void shutdown() { if (!ready) return; for (auto &kv : S) for (Sound &s : kv.second) UnloadSound(s); CloseAudioDevice(); }
}
