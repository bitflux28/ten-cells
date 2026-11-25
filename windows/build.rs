// Compiles the exe icon with zig's resource compiler and links it in.
use std::{env, path::PathBuf, process::Command};
fn main() {
    println!("cargo:rerun-if-changed=res/app.rc");
    println!("cargo:rerun-if-changed=res/icon.ico");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") { return; }
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("app.res");
    let ok = Command::new("python3").args(["-m", "ziglang", "rc", "res/app.rc"]).arg(&out).status().map(|s| s.success()).unwrap_or(false);
    if ok { println!("cargo:rustc-link-arg={}", out.display()); } else { println!("cargo:warning=icon resource skipped"); }
}
