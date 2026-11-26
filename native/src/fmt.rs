// Number and time formatting, same output as the web version.
use crate::big::{b, Big};
use crate::data::data;
use std::sync::atomic::{AtomicU8, Ordering};

/// 0 suffix (1.23M), 1 scientific (1.23e6), 2 engineering (1.23e6 in steps of 3)
pub static NOTATION: AtomicU8 = AtomicU8::new(0);
pub fn set_notation(s: &str) { NOTATION.store(match s { "sci" => 1, "eng" => 2, _ => 0 }, Ordering::Relaxed) }

fn fixed(x: f64, dp: usize) -> String { format!("{:.*}", dp, x) }
pub fn fmt(x: Big) -> String { fmt_dp(x, 2) }
pub fn fmt_dp(x: Big, dp: usize) -> String {
    if x.m < 0.0 { return format!("-{}", fmt_dp(x.neg(), dp)) }
    if x.ltf(1000.0) {
        let n = x.to_f64();
        return if n < 10.0 && dp > 0 { fixed(n, if n % 1.0 != 0.0 { 2 } else { 0 }) } else if n < 100.0 && dp > 0 { fixed(n, if n % 1.0 != 0.0 { 1 } else { 0 }) } else { format!("{}", n.floor() as i64) };
    }
    let e = x.e; let not = NOTATION.load(Ordering::Relaxed);
    if e >= 1e6 { return format!("e{}", fmt_dp(b(e), 2)) }
    let suf = &data().suf;
    if not == 0 && e < (suf.len() * 3) as f64 { let k = (e / 3.0).floor(); return format!("{}{}", fixed(x.m * 10f64.powf(e - k * 3.0), 2), suf[k as usize]) }
    if not == 2 { let k = (e / 3.0).floor() * 3.0; return format!("{}e{}", fixed(x.m * 10f64.powf(e - k), 2), k as i64) }
    format!("{}e{}", fixed(x.m, 2), group(e as i64))
}
pub fn fmtf(x: f64) -> String { fmt(b(x)) }
pub fn group(n: i64) -> String { let s = n.abs().to_string(); let mut o = String::new(); for (i, c) in s.chars().enumerate() { if i > 0 && (s.len() - i) % 3 == 0 { o.push(',') } o.push(c) } if n < 0 { format!("-{o}") } else { o } }
pub fn fmt_t(s: f64) -> String {
    let s = s.max(0.0).floor() as i64; let (d, h, m, x) = (s / 86400, (s / 3600) % 24, (s / 60) % 60, s % 60);
    if d > 0 { format!("{d}d {h}h") } else if h > 0 { format!("{h}h {m}m") } else if m > 0 { format!("{m}m {x}s") } else { format!("{x}s") }
}
