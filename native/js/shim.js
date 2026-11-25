// What the rules expect from a browser, provided by the native app (file-backed storage, base64, a notice queue).
var __store = {};
var localStorage = { getItem: k => (k in __store ? __store[k] : null), setItem: (k, v) => { __store[k] = String(v); __native_save(k, String(v)); }, removeItem: k => { delete __store[k]; } };
var __B64 = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
function btoa(s) { let o = '', i = 0; for (; i < s.length; i += 3) { const a = s.charCodeAt(i), b = s.charCodeAt(i + 1), c = s.charCodeAt(i + 2), n = (a << 16) | ((b || 0) << 8) | (c || 0); o += __B64[n >> 18 & 63] + __B64[n >> 12 & 63] + (i + 1 < s.length ? __B64[n >> 6 & 63] : '=') + (i + 2 < s.length ? __B64[n & 63] : '='); } return o; }
function atob(s) { s = s.replace(/[^A-Za-z0-9+/]/g, ''); let o = '', i = 0; for (; i < s.length; i += 4) { const n = (__B64.indexOf(s[i]) << 18) | (__B64.indexOf(s[i + 1]) << 12) | ((__B64.indexOf(s[i + 2]) & 63) << 6) | (__B64.indexOf(s[i + 3]) & 63); o += String.fromCharCode(n >> 16 & 255); if (s[i + 2] !== undefined && s[i + 2] !== '=') o += String.fromCharCode(n >> 8 & 255); if (s[i + 3] !== undefined && s[i + 3] !== '=') o += String.fromCharCode(n & 255); } return o; }
var window = globalThis; var self = globalThis;
