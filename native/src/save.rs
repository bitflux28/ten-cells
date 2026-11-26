// Saves: a file in Documents/Ten Cells (save.txt, plus a backup), in the same format as the web version's localStorage,
// so a web export code can be imported and this build's export code works in the web version too.
use crate::game::S;
use std::path::PathBuf;

fn hash(s: &str) -> i32 { let mut h: i32 = 7; for c in s.encode_utf16() { h = h.wrapping_mul(31).wrapping_add(c as i32) } h }
pub fn dir() -> PathBuf {
    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let docs = home.join("Documents"); let base = if docs.is_dir() { docs } else { home };
    base.join("Ten Cells")
}
pub fn pack(s: &S) -> String { let d = serde_json::to_string(s).unwrap(); serde_json::json!({ "h": hash(&d), "v": 1, "d": d }).to_string() }
pub fn parse(blob: &str) -> Result<serde_json::Value, String> {
    let o: serde_json::Value = serde_json::from_str(blob.trim()).map_err(|e| e.to_string())?;
    let d = o["d"].as_str().ok_or("not a save")?;
    if o["h"].as_i64() != Some(hash(d) as i64) { return Err("checksum mismatch".into()) }
    serde_json::from_str(d).map_err(|e| e.to_string())
}
/// story fixes the web version applies to very old saves are not needed here: any save this old was moved forward by the web version already
pub fn hydrate(v: serde_json::Value) -> Result<S, String> {
    let mut s: S = serde_json::from_value(v).map_err(|e| e.to_string())?;
    s.boss.f = None; s.ev.buffs.retain(|b| b.m.is_finite() && b.t.is_finite());
    let laws: Vec<String> = crate::data::data().laws.iter().map(|l| l.id.clone()).collect(); s.laws.retain(|x| laws.contains(x));
    while s.gens.len() < crate::data::NG { s.gens.push(0.0) } while s.auto.gens.len() < crate::data::NG { s.auto.gens.push(0.0) }
    for d in &crate::data::data().dims { s.dims.entry(d.id.clone()).or_default(); }
    for d in s.dims.values_mut() { while d.g.len() < 3 { d.g.push(Default::default()) } }
    Ok(s)
}
pub fn save(s: &S) -> Result<(), String> {
    let dir = dir(); std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let f = dir.join("save.txt"); let blob = pack(s);
    if let Ok(prev) = std::fs::read_to_string(&f) { if parse(&prev).is_ok() { let _ = std::fs::write(dir.join("save.bak.txt"), prev); } }
    let tmp = dir.join("save.tmp"); std::fs::write(&tmp, blob).map_err(|e| e.to_string())?; std::fs::rename(&tmp, &f).map_err(|e| e.to_string())
}
fn read_any(t: &str) -> Result<S, String> { parse(t).and_then(hydrate).or_else(|_| import(t)) }
/// (state, loaded from backup or an older app?, something was unreadable?)
pub fn load() -> (Option<S>, bool, bool) {
    let dir = dir(); let mut corrupt = false;
    for (i, name) in ["save.txt", "save.bak.txt"].iter().enumerate() {
        if let Ok(t) = std::fs::read_to_string(dir.join(name)) {
            match read_any(&t) { Ok(s) => return (Some(s), i > 0, corrupt), Err(_) => {
                // never overwrite a save we can't read: keep a copy next to it
                corrupt = true; let _ = std::fs::write(dir.join(format!("{}.unreadable-{}.txt", name.trim_end_matches(".txt"), crate::game::now_ms() as u64)), &t);
            } }
        }
    }
    // first launch: bring over the newest save from the older Windows apps (WebView2 and Electron keep it in browser storage)
    let mut best: Option<S> = None;
    for blob in legacy_blobs() { if let Ok(s) = parse(&blob).and_then(hydrate) { if best.as_ref().map(|b| s.last > b.last).unwrap_or(true) { best = Some(s) } } }
    if best.is_some() { return (best, true, corrupt) }
    (None, false, corrupt)
}
fn legacy_blobs() -> Vec<String> {
    let Some(app) = std::env::var_os("APPDATA").map(PathBuf::from) else { return vec![] };
    let mut dirs = vec![app.join("Infinity Engine").join("Local Storage").join("leveldb")];
    // WebView2 keeps it under <data>/EBWebView/<profile>/Local Storage/leveldb
    fn walk(d: &std::path::Path, out: &mut Vec<PathBuf>, depth: u32) { if depth > 5 { return } for e in std::fs::read_dir(d).into_iter().flatten().flatten() { let p = e.path(); if p.is_dir() { if p.ends_with("Local Storage/leveldb") || p.ends_with("Local Storage\\leveldb") || (p.file_name().map(|f| f == "leveldb").unwrap_or(false) && p.parent().and_then(|q| q.file_name()).map(|f| f == "Local Storage").unwrap_or(false)) { out.push(p) } else { walk(&p, out, depth + 1) } } } }
    walk(&app.join("Ten Cells"), &mut dirs, 0);
    let mut out = vec![];
    for d in dirs { for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
        let p = e.path(); if !p.extension().map(|x| x == "log" || x == "ldb").unwrap_or(false) { continue }
        let Ok(raw) = std::fs::read(&p) else { continue };
        let latin: String = raw.iter().map(|&b| b as char).collect();
        let u16a = String::from_utf16_lossy(&raw.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect::<Vec<_>>());
        let u16b = if raw.len() > 1 { String::from_utf16_lossy(&raw[1..].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect::<Vec<_>>()) } else { String::new() };
        for text in [&latin, &u16a, &u16b] { let mut from = 0; while let Some(i) = text[from..].find("{\"h\":").map(|i| i + from) { if let Some(end) = json_end(&text[i..]) { out.push(text[i..i + end].to_string()) } from = i + 5; } }
    } }
    out
}
fn json_end(s: &str) -> Option<usize> {
    let (mut depth, mut in_str, mut esc) = (0i32, false, false);
    for (i, c) in s.char_indices() { if i > 4_000_000 { return None } if in_str { if esc { esc = false } else if c == '\\' { esc = true } else if c == '"' { in_str = false } continue } match c { '"' => in_str = true, '{' => depth += 1, '}' => { depth -= 1; if depth == 0 { return Some(i + 1) } } _ => {} } }
    None
}
// base64, for export/import codes (the web version uses btoa of the UTF-8 save)
const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
pub fn b64e(data: &[u8]) -> String { let mut o = String::new(); for c in data.chunks(3) { let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32; for i in 0..4 { if i <= c.len() { o.push(B64[(n >> (18 - 6 * i) & 63) as usize] as char) } else { o.push('=') } } } o }
pub fn b64d(s: &str) -> Option<Vec<u8>> {
    let mut o = vec![]; let mut buf = 0u32; let mut n = 0;
    for ch in s.bytes().filter(|c| !c.is_ascii_whitespace()) { if ch == b'=' { break } let v = B64.iter().position(|x| *x == ch)? as u32; buf = buf << 6 | v; n += 6; if n >= 8 { n -= 8; o.push((buf >> n & 255) as u8) } }
    Some(o)
}
pub fn export(s: &S) -> String { b64e(pack(s).as_bytes()) }
pub fn import(code: &str) -> Result<S, String> { let raw = b64d(code.trim()).ok_or("That isn't a save code")?; let t = String::from_utf8(raw).map_err(|_| "That isn't a save code")?; parse(&t).and_then(hydrate) }
