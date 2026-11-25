// Minimal JSON reader for the views the rules engine sends (objects, arrays, strings, numbers, booleans, null).
#pragma once
#include <map>
#include <memory>
#include <string>
#include <vector>

struct J {
    enum T { Null, Bool, Num, Str, Arr, Obj } t = Null;
    bool b = false; double n = 0; std::string s;
    std::vector<J> a; std::map<std::string, J> o;
    const J &operator[](const char *k) const { static const J none; auto it = o.find(k); return it == o.end() ? none : it->second; }
    const J &operator[](size_t i) const { static const J none; return i < a.size() ? a[i] : none; }
    const J &operator[](int i) const { return (*this)[(size_t)i]; }
    size_t size() const { return t == Arr ? a.size() : t == Obj ? o.size() : 0; }
    std::string str(const char *d = "") const { return t == Str ? s : t == Num ? std::to_string(n) : d; }
    double num(double d = 0) const { return t == Num ? n : t == Bool ? (b ? 1 : 0) : d; }
    bool truthy() const { return t == Bool ? b : t == Num ? n != 0 : t == Str ? !s.empty() : t == Arr || t == Obj; }
    bool has(const char *k) const { return t == Obj && o.count(k); }
    static J parse(const std::string &src) { size_t i = 0; J v; try { v = val(src, i); } catch (...) { v = J(); } return v; }

  private:
    static void ws(const std::string &s, size_t &i) { while (i < s.size() && (s[i] == ' ' || s[i] == '\n' || s[i] == '\r' || s[i] == '\t')) i++; }
    static void utf8(std::string &o, unsigned c) {
        if (c < 0x80) o += (char)c;
        else if (c < 0x800) { o += (char)(0xC0 | (c >> 6)); o += (char)(0x80 | (c & 63)); }
        else if (c < 0x10000) { o += (char)(0xE0 | (c >> 12)); o += (char)(0x80 | ((c >> 6) & 63)); o += (char)(0x80 | (c & 63)); }
        else { o += (char)(0xF0 | (c >> 18)); o += (char)(0x80 | ((c >> 12) & 63)); o += (char)(0x80 | ((c >> 6) & 63)); o += (char)(0x80 | (c & 63)); }
    }
    static std::string strv(const std::string &s, size_t &i) {
        std::string o; i++;
        while (i < s.size() && s[i] != '"') {
            if (s[i] == '\\' && i + 1 < s.size()) {
                char e = s[++i];
                if (e == 'n') o += '\n'; else if (e == 't') o += '\t'; else if (e == 'r') o += '\r'; else if (e == 'b') o += '\b'; else if (e == 'f') o += '\f';
                else if (e == 'u' && i + 4 < s.size()) {
                    unsigned c = std::stoul(s.substr(i + 1, 4), nullptr, 16); i += 4;
                    if (c >= 0xD800 && c < 0xDC00 && i + 6 < s.size() && s[i + 1] == '\\' && s[i + 2] == 'u') { unsigned lo = std::stoul(s.substr(i + 3, 4), nullptr, 16); c = 0x10000 + ((c - 0xD800) << 10) + (lo - 0xDC00); i += 6; }
                    utf8(o, c);
                } else o += e;
                i++;
            } else o += s[i++];
        }
        i++; return o;
    }
    static J val(const std::string &s, size_t &i) {
        ws(s, i); J v; if (i >= s.size()) return v;
        char c = s[i];
        if (c == '{') {
            v.t = Obj; i++; ws(s, i);
            if (s[i] == '}') { i++; return v; }
            while (i < s.size()) { ws(s, i); std::string k = strv(s, i); ws(s, i); i++; v.o[k] = val(s, i); ws(s, i); if (s[i] == ',') { i++; continue; } i++; break; }
        } else if (c == '[') {
            v.t = Arr; i++; ws(s, i);
            if (s[i] == ']') { i++; return v; }
            while (i < s.size()) { v.a.push_back(val(s, i)); ws(s, i); if (s[i] == ',') { i++; continue; } i++; break; }
        } else if (c == '"') { v.t = Str; v.s = strv(s, i); }
        else if (c == 't') { v.t = Bool; v.b = true; i += 4; }
        else if (c == 'f') { v.t = Bool; v.b = false; i += 5; }
        else if (c == 'n') { i += 4; }
        else { size_t j = i; while (j < s.size() && (isdigit((unsigned char)s[j]) || s[j] == '-' || s[j] == '+' || s[j] == '.' || s[j] == 'e' || s[j] == 'E')) j++; v.t = Num; v.n = std::stod(s.substr(i, j - i)); i = j; }
        return v;
    }
};
