#include "js.h"
#include "embed.h"
extern "C" {
#include "quickjs.h"
}
#include <cstdio>
#include <filesystem>
#include <fstream>
#include <sstream>

namespace fs = std::filesystem;
namespace js {
static JSRuntime *rt; static JSContext *ctx; static std::string dir, err_;

// localStorage keys map to files: "infinityEngine" -> save.txt, "infinityEngine_bak" -> save_bak.txt
static fs::path fileFor(const std::string &key) { return fs::u8path(dir) / (key == "infinityEngine" ? "save.txt" : key == "infinityEngine_bak" ? "save_bak.txt" : key + ".txt"); }

static JSValue nativeSave(JSContext *c, JSValueConst, int argc, JSValueConst *argv) {
    if (argc < 2) return JS_UNDEFINED;
    const char *k = JS_ToCString(c, argv[0]); size_t n = 0; const char *v = JS_ToCStringLen(c, &n, argv[1]);
    if (k && v) {
        // write to a temp file, then rename over the old one, so a crash mid-write can't leave a half save
        fs::path p = fileFor(k), tmp = p; tmp += ".tmp";
        { std::ofstream o(tmp, std::ios::binary); o.write(v, (std::streamsize)n); }
        std::error_code ec; fs::rename(tmp, p, ec);
        if (ec) { fs::remove(p, ec); fs::rename(tmp, p, ec); }
    }
    JS_FreeCString(c, k); JS_FreeCString(c, v); return JS_UNDEFINED;
}

static std::string exc() {
    JSValue e = JS_GetException(ctx); const char *s = JS_ToCString(ctx, e); std::string out = s ? s : "?";
    JSValue st = JS_GetPropertyStr(ctx, e, "stack"); if (!JS_IsUndefined(st)) { const char *ss = JS_ToCString(ctx, st); if (ss) { out += "\n"; out += ss; } JS_FreeCString(ctx, ss); }
    JS_FreeValue(ctx, st); JS_FreeCString(ctx, s); JS_FreeValue(ctx, e); return out;
}

static bool evalSrc(const unsigned char *src, size_t len, const char *name) {
    std::string code((const char *)src, len);
    JSValue v = JS_Eval(ctx, code.c_str(), code.size(), name, JS_EVAL_TYPE_GLOBAL);
    if (JS_IsException(v)) { err_ = std::string(name) + ": " + exc(); return false; }
    JS_FreeValue(ctx, v); return true;
}

bool init(const std::string &saveDir, std::string &err) {
    dir = saveDir; std::error_code ec; fs::create_directories(fs::u8path(dir), ec);
    rt = JS_NewRuntime(); JS_SetMaxStackSize(rt, 4 << 20); ctx = JS_NewContext(rt);
    JSValue g = JS_GetGlobalObject(ctx);
    JS_SetPropertyStr(ctx, g, "__native_save", JS_NewCFunction(ctx, nativeSave, "__native_save", 2));
    JS_FreeValue(ctx, g);
    if (!evalSrc(EMB_shim_js, EMB_shim_js_len, "shim.js")) { err = err_; return false; }
    // load the save files into the localStorage stand-in before the game reads them
    JSValue store = JS_Eval(ctx, "__store", 7, "<s>", JS_EVAL_TYPE_GLOBAL);
    for (const char *k : {"infinityEngine", "infinityEngine_bak"}) {
        std::ifstream f(fileFor(k), std::ios::binary); if (!f) continue;
        std::stringstream ss; ss << f.rdbuf(); std::string v = ss.str();
        JS_SetPropertyStr(ctx, store, k, JS_NewStringLen(ctx, v.data(), v.size()));
    }
    JS_FreeValue(ctx, store);
    if (!evalSrc(EMB_decimal_js, EMB_decimal_js_len, "decimal.js") || !evalSrc(EMB_data_js, EMB_data_js_len, "data.js") ||
        !evalSrc(EMB_logic_js, EMB_logic_js_len, "logic.js") || !evalSrc(EMB_api_js, EMB_api_js_len, "api.js")) { err = err_; return false; }
    return true;
}

std::string call(const char *fn, const std::string &args) {
    std::string code = std::string("NAT.") + fn + "(" + args + ")";
    JSValue v = JS_Eval(ctx, code.c_str(), code.size(), "<call>", JS_EVAL_TYPE_GLOBAL);
    if (JS_IsException(v)) { err_ = std::string(fn) + ": " + exc(); fprintf(stderr, "JS error %s\n", err_.c_str()); return ""; }
    std::string out; if (JS_IsString(v)) { size_t n; const char *s = JS_ToCStringLen(ctx, &n, v); out.assign(s, n); JS_FreeCString(ctx, s); }
    JS_FreeValue(ctx, v);
    // run pending promise jobs, if any
    JSContext *c2; while (JS_ExecutePendingJob(rt, &c2) > 0) {}
    return out;
}

std::string lastError() { return err_; }
void shutdown() { JS_FreeContext(ctx); JS_FreeRuntime(rt); }

std::string quote(const std::string &s) {
    std::string o = "\"";
    for (unsigned char c : s) {
        if (c == '"' || c == '\\') { o += '\\'; o += (char)c; }
        else if (c == '\n') o += "\\n"; else if (c == '\r') o += "\\r"; else if (c == '\t') o += "\\t";
        else if (c < 0x20) { char b[8]; snprintf(b, sizeof b, "\\u%04x", c); o += b; }
        else o += (char)c;
    }
    return o + "\"";
}
}
