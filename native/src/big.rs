// Big numbers as mantissa × 10^exponent, the same model as break_infinity.js in the web version,
// so saves move between the two unchanged and the maths gives the same results.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Big { pub m: f64, pub e: f64 }

const MAX_DIGITS: f64 = 17.0;

impl Big {
    pub const ZERO: Big = Big { m: 0.0, e: 0.0 };
    pub const ONE: Big = Big { m: 1.0, e: 0.0 };
    pub fn new(x: f64) -> Big {
        if x == 0.0 || !x.is_finite() { return if x.is_infinite() { Big { m: x.signum() * 9.99, e: 9e15 } } else { Big::ZERO } }
        let e = x.abs().log10().floor();
        let mut b = Big { m: x / 10f64.powf(e), e };
        if e < -300.0 { b.m = x * 10f64.powf(-e) } // keep subnormals sane
        b.norm()
    }
    pub fn from_me(m: f64, e: f64) -> Big { Big { m, e }.norm() }
    fn norm(mut self) -> Big {
        if self.m == 0.0 || !self.m.is_finite() { return Big::ZERO }
        if self.m.abs() >= 1.0 && self.m.abs() < 10.0 { return self }
        let t = self.m.abs().log10().floor();
        self.m /= 10f64.powf(t); self.e += t;
        if self.m.abs() >= 10.0 { self.m /= 10.0; self.e += 1.0 } else if self.m.abs() < 1.0 { self.m *= 10.0; self.e -= 1.0 }
        self
    }
    pub fn parse(s: &str) -> Option<Big> {
        let s = s.trim();
        if s.is_empty() { return None }
        if let Ok(v) = s.parse::<f64>() { if v.is_finite() { return Some(Big::new(v)) } }
        let (a, b) = s.split_once(['e', 'E'])?;
        let m: f64 = if a.is_empty() { 1.0 } else { a.parse().ok()? };
        let e: f64 = b.trim_start_matches('+').parse().ok()?;
        if !m.is_finite() || !e.is_finite() { return None }
        Some(Big::from_me(m, e))
    }
    pub fn is_zero(&self) -> bool { self.m == 0.0 }
    pub fn to_f64(&self) -> f64 { if self.m == 0.0 { 0.0 } else if self.e > 308.0 { f64::INFINITY * self.m.signum() } else { self.m * 10f64.powf(self.e) } }
    pub fn log10(&self) -> f64 { if self.m <= 0.0 { f64::NEG_INFINITY } else { self.e + self.m.log10() } }
    pub fn from_log10(l: f64) -> Big { if !l.is_finite() { return if l > 0.0 { Big { m: 9.99, e: 9e15 } } else { Big::ZERO } } let e = l.floor(); Big::from_me(10f64.powf(l - e), e) }
    pub fn add(self, o: Big) -> Big {
        if self.m == 0.0 { return o } if o.m == 0.0 { return self }
        let (big, small) = if self.e >= o.e { (self, o) } else { (o, self) };
        if big.e - small.e > MAX_DIGITS { return big }
        Big::from_me(big.m + small.m * 10f64.powf(small.e - big.e), big.e)
    }
    pub fn sub(self, o: Big) -> Big { self.add(o.neg()) }
    pub fn neg(self) -> Big { Big { m: -self.m, e: self.e } }
    pub fn mul(self, o: Big) -> Big { if self.m == 0.0 || o.m == 0.0 { return Big::ZERO } Big::from_me(self.m * o.m, self.e + o.e) }
    pub fn mulf(self, x: f64) -> Big { self.mul(Big::new(x)) }
    pub fn div(self, o: Big) -> Big { if o.m == 0.0 { return Big { m: 9.99, e: 9e15 } } if self.m == 0.0 { return Big::ZERO } Big::from_me(self.m / o.m, self.e - o.e) }
    pub fn divf(self, x: f64) -> Big { self.div(Big::new(x)) }
    pub fn addf(self, x: f64) -> Big { self.add(Big::new(x)) }
    pub fn pow(self, p: f64) -> Big {
        if self.m == 0.0 { return if p == 0.0 { Big::ONE } else { Big::ZERO } }
        if p == 0.0 { return Big::ONE }
        if self.m < 0.0 { return Big::from_log10(self.neg().log10() * p).neg() }
        // exact for small exponents (matches break_infinity's fast path)
        if self.e.abs() < 300.0 && (self.to_f64().powf(p)).is_finite() && self.to_f64().powf(p) != 0.0 { return Big::new(self.to_f64().powf(p)) }
        Big::from_log10(self.log10() * p)
    }
    pub fn floor(self) -> Big { if self.e >= MAX_DIGITS { self } else if self.e < -1.0 { if self.m < 0.0 { Big::new(-1.0) } else { Big::ZERO } } else { Big::new(self.to_f64().floor()) } }
    pub fn ceil(self) -> Big { if self.e >= MAX_DIGITS { self } else { Big::new(self.to_f64().ceil()) } }
    pub fn cmp(&self, o: &Big) -> Ordering {
        let (a, b) = (self, o);
        if a.m == 0.0 && b.m == 0.0 { return Ordering::Equal }
        if a.m == 0.0 { return if b.m > 0.0 { Ordering::Less } else { Ordering::Greater } }
        if b.m == 0.0 { return if a.m > 0.0 { Ordering::Greater } else { Ordering::Less } }
        if a.m > 0.0 && b.m < 0.0 { return Ordering::Greater } if a.m < 0.0 && b.m > 0.0 { return Ordering::Less }
        let sign = if a.m > 0.0 { 1.0 } else { -1.0 };
        let r = if a.e != b.e { (a.e - b.e) * sign } else { (a.m - b.m) * 1.0 };
        if r > 0.0 { Ordering::Greater } else if r < 0.0 { Ordering::Less } else { Ordering::Equal }
    }
    pub fn gt(&self, o: Big) -> bool { self.cmp(&o) == Ordering::Greater }
    pub fn gte(&self, o: Big) -> bool { self.cmp(&o) != Ordering::Less }
    pub fn lt(&self, o: Big) -> bool { self.cmp(&o) == Ordering::Less }
    pub fn lte(&self, o: Big) -> bool { self.cmp(&o) != Ordering::Greater }
    pub fn gtf(&self, x: f64) -> bool { self.gt(Big::new(x)) }
    pub fn gtef(&self, x: f64) -> bool { self.gte(Big::new(x)) }
    pub fn ltf(&self, x: f64) -> bool { self.lt(Big::new(x)) }
    pub fn max(self, o: Big) -> Big { if self.gte(o) { self } else { o } }
    pub fn min(self, o: Big) -> Big { if self.lte(o) { self } else { o } }
    pub fn maxf(self, x: f64) -> Big { self.max(Big::new(x)) }
    /// cost of buying n more when you own `owned`, prices start*ratio^k (break_infinity sumGeometricSeries)
    pub fn sum_geo(n: f64, start: Big, ratio: f64, owned: f64) -> Big {
        start.mul(Big::new(ratio).pow(owned)).mul(Big::ONE.sub(Big::new(ratio).pow(n))).div(Big::new(1.0 - ratio))
    }
    /// how many you can afford (break_infinity affordGeometricSeries)
    pub fn afford_geo(res: Big, start: Big, ratio: f64, owned: f64) -> f64 {
        let actual = start.mul(Big::new(ratio).pow(owned));
        let v = res.div(actual).mulf(ratio - 1.0).addf(1.0).log10() / ratio.log10();
        if v.is_finite() { v.floor().max(0.0) } else { 0.0 }
    }
    /// break_infinity's toString: plain digits under 1e21, else "m e+x"
    pub fn to_js_string(&self) -> String {
        if self.m == 0.0 { return "0".into() }
        if self.e < 21.0 && self.e > -7.0 { return js_num(self.to_f64()) }
        format!("{}e{}{}", js_num(self.m), if self.e >= 0.0 { "+" } else { "" }, self.e as i64)
    }
}
/// format an f64 the way JavaScript's Number#toString does for ordinary values
pub fn js_num(x: f64) -> String {
    if x == x.trunc() && x.abs() < 1e21 { return format!("{}", x as i128) }
    let s = format!("{}", x); s
}
impl From<f64> for Big { fn from(x: f64) -> Big { Big::new(x) } }
impl Serialize for Big { fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> { s.serialize_str(&self.to_js_string()) } }
impl<'de> Deserialize<'de> for Big {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Big, D::Error> {
        let v = serde_json::Value::deserialize(d)?;
        Ok(match v { serde_json::Value::Number(n) => Big::new(n.as_f64().unwrap_or(0.0)), serde_json::Value::String(s) => Big::parse(&s).filter(|b| b.m >= 0.0).unwrap_or(Big::ZERO), _ => Big::ZERO })
    }
}
pub fn b(x: f64) -> Big { Big::new(x) }
pub fn bs(s: &str) -> Big { Big::parse(s).unwrap_or(Big::ZERO) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basics() {
        assert_eq!(b(5.0).add(b(7.0)).to_f64(), 12.0);
        assert!((bs("1e1000").mul(bs("1e1000")).log10() - 2000.0).abs() < 1e-9);
        assert_eq!(bs("1e+30").to_js_string(), "1e+30");
        assert_eq!(b(123.0).to_js_string(), "123");
        assert!(b(1e12).div(b(1e12)).pow(0.33).to_f64() == 1.0);
        let c = Big::sum_geo(1.0, b(10.0), 1.15, 0.0); assert!((c.to_f64() - 10.0).abs() < 1e-9);
        assert_eq!(Big::afford_geo(b(100.0), b(10.0), 1.15, 0.0), 6.0);
        assert!(bs("1.79e308").gt(b(1e308)));
        assert_eq!(b(9.7).floor().to_f64(), 9.0);
        assert!(b(3.0).sub(b(5.0)).lt(Big::ZERO));
    }
}
