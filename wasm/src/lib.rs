// The Ten Cells nebula painter: the per-pixel loop of buildSpace(), moved out of JavaScript.
// Same maths as the JS version; the web page keeps the JS loop as a fallback.
#![no_std]
use core::ptr::addr_of_mut;

#[panic_handler] fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }

static mut BLOBS: [f64; 150] = [0.0; 150]; // 30 × (x, y, r, hue, weight)
static mut PALS: [u8; 36] = [0; 36];       // 3 palettes × 4 levels × rgb
static mut NZ: [f32; 4096] = [0.0; 4096];
const BAYER: [u8; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];

#[no_mangle] pub extern "C" fn blobs() -> *mut f64 { addr_of_mut!(BLOBS) as *mut f64 }
#[no_mangle] pub extern "C" fn pals() -> *mut u8 { addr_of_mut!(PALS) as *mut u8 }
#[no_mangle] pub extern "C" fn noise() -> *mut f32 { addr_of_mut!(NZ) as *mut f32 }

fn floor(x: f64) -> f64 { let t = x as i64 as f64; if t > x { t - 1.0 } else { t } }
fn exp(x: f64) -> f64 {
    if x < -700.0 { return 0.0 }
    let k = floor(x * core::f64::consts::LOG2_E + 0.5);
    let r = x - k * core::f64::consts::LN_2;
    let t = 1.0 + r * (1.0 + r * (0.5 + r * (0.16666666666666666 + r * (0.041666666666666664 + r * (0.008333333333333333 + r * (0.001388888888888889 + r * (0.0001984126984126984 + r * (2.48015873015873e-05 + r * (2.7557319223985893e-06 + r * (2.755731922398589e-07 + r * (2.505210838544172e-08 + r * (2.08767569878681e-09))))))))))));
    t * f64::from_bits(((k as i64 + 1023) as u64) << 52)
}
fn vn(nz: &[f32; 4096], x: f64, y: f64) -> f64 {
    let (xf0, yf0) = (floor(x), floor(y)); let (xi, yi) = (xf0 as i32, yf0 as i32);
    let (xf, yf) = (x - xf0, y - yf0);
    let u = xf * xf * (3.0 - 2.0 * xf); let v = yf * yf * (3.0 - 2.0 * yf);
    let g = |a: i32, b: i32| nz[(((b & 63) << 6) | (a & 63)) as usize] as f64;
    (g(xi, yi) * (1.0 - u) + g(xi + 1, yi) * u) * (1.0 - v) + (g(xi, yi + 1) * (1.0 - u) + g(xi + 1, yi + 1) * u) * v
}
/// round half to even, like a Uint8ClampedArray store
fn byte(v: f64) -> u8 {
    if !(v > 0.0) { return 0 } if v >= 255.0 { return 255 }
    let f = floor(v); let d = v - f;
    (if d > 0.5 || (d == 0.5 && (f as i64) & 1 == 1) { f + 1.0 } else { f }) as u8
}

/// paints nw×nh RGBA pixels at `out`. `scratch` holds 30 × (nw + nh) f64.
/// A blob is a Gaussian, e^-(dx²+dy²)/r² = e^-(dx²/r²) · e^-(dy²/r²), so each blob needs one exp per column and one per
/// row instead of one per pixel.
#[no_mangle] pub unsafe extern "C" fn fill(nw: i32, nh: i32, zs: f64, out: *mut u8, scratch: *mut f64) {
    let (bl, pl, nz) = (&*addr_of_mut!(BLOBS), &*addr_of_mut!(PALS), &*addr_of_mut!(NZ));
    let (nw_, nh_) = (nw as usize, nh as usize); let stride = nw_ + nh_;
    for k in 0..30 { let b = &bl[k * 5..k * 5 + 5]; let ir2 = 1.0 / (b[2] * b[2]); let row = scratch.add(k * stride);
        for x in 0..nw_ { let d = x as f64 - b[0]; *row.add(x) = exp(-d * d * ir2) }
        for y in 0..nh_ { let d = y as f64 - b[1]; *row.add(nw_ + y) = exp(-d * d * ir2) } }
    for y in 0..nh { for x in 0..nw {
        let (xf, yf) = (x as f64, y as f64);
        let mut dens = 0.0; let mut wg = [0.0f64; 3];
        for k in 0..30 { let v = bl[k * 5 + 4] * *scratch.add(k * stride + x as usize) * *scratch.add(k * stride + nw_ + y as usize);
            dens += v; wg[(bl[k * 5 + 3] as i64 % 3) as usize] += v }
        for g in 0..3 { wg[g] = wg[g] * wg[g] * wg[g] }
        let (cx, cy) = (xf / zs, yf / zs);
        let f = vn(nz, cx / 26.0, cy / 26.0) * 0.5 + vn(nz, cx / 11.0 + 17.0, cy / 11.0 + 5.0) * 0.32 + vn(nz, cx / 5.0 + 3.0, cy / 5.0 + 11.0) * 0.18;
        let dk = (0.4 - vn(nz, cx / 48.0 + 40.0, cy / 36.0 + 9.0)) * 5.0; let dark = if dk < 0.0 { 0.0 } else if dk > 1.0 { 1.0 } else { dk };
        let lf = (if dens * 3.2 < 4.4 { dens * 3.2 } else { 4.4 }) * (0.78 + f * 0.44) + (f - 0.5) * 1.4 - dark * 1.5;
        let mut l = floor(lf + BAYER[(((y & 3) << 2) | (x & 3)) as usize] as f64 / 16.0 - 0.5); if l > 4.0 { l = 4.0 }
        if l < 1.0 { continue }
        let li = (l as usize - 1) * 3; let mut ws = wg[0] + wg[1] + wg[2]; if ws == 0.0 { ws = 1.0 }
        let i = ((y * nw + x) * 4) as usize;
        for ch in 0..3 { *out.add(i + ch) = byte((pl[li + ch] as f64 * wg[0] + pl[12 + li + ch] as f64 * wg[1] + pl[24 + li + ch] as f64 * wg[2]) / ws) }
        *out.add(i + 3) = 255;
    } }
}
