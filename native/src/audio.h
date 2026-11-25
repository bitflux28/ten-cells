#pragma once
#include <string>
namespace audio {
void init();
void play(const std::string &name, float pitch = 1.f);
void enable(bool on);
void shutdown();
}
