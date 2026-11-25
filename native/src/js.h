// The rules engine: QuickJS running the web game's own data and logic scripts, plus the native bridge (api.js).
#pragma once
#include <string>

namespace js {
bool init(const std::string &saveDir, std::string &err);   // loads saves from disk, evaluates the scripts
std::string call(const char *fn, const std::string &args = ""); // NAT.<fn>(<args>) -> string result ("" on error)
std::string lastError();
void shutdown();
std::string quote(const std::string &s);                     // JSON string literal for passing args
}
