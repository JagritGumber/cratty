use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let zig_dir = manifest_dir.join("zig");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let zig_exe = env::var("ZIG_EXE").unwrap_or_else(|_| "zig".to_string());

    let zig_out = out_dir.join("zig-out");

    let mut cmd = Command::new(&zig_exe);
    cmd.current_dir(&zig_dir)
        .arg("build")
        .arg("-Doptimize=ReleaseFast")
        .arg("--prefix")
        .arg(&zig_out);

    // Only set target for cross-compilation. For native builds, let Zig auto-detect.
    if let Some(target) = get_zig_target() {
        cmd.arg(&format!("-Dtarget={target}"));
    }

    let output = cmd.output().expect(
        "Failed to run zig build. Make sure Zig is installed and available in PATH.",
    );

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Zig build failed:\n{stderr}");
    }

    // Tell cargo where to find the library.
    let lib_dir = zig_out.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=cratty_vt_zig");

    // Rerun if zig sources change.
    println!("cargo:rerun-if-changed=zig/src/");
    println!("cargo:rerun-if-changed=zig/build.zig");
}

fn get_zig_target() -> Option<String> {
    let target = env::var("TARGET").ok()?;
    let host = env::var("HOST").ok()?;

    // Don't set target for native builds.
    if target == host {
        return None;
    }

    let zig_target = match target.as_str() {
        "x86_64-pc-windows-msvc" => "x86_64-windows",
        "x86_64-unknown-linux-gnu" => "x86_64-linux-gnu",
        "x86_64-apple-darwin" => "x86_64-macos",
        "aarch64-apple-darwin" => "aarch64-macos",
        "aarch64-unknown-linux-gnu" => "aarch64-linux-gnu",
        _ => return None,
    };

    Some(zig_target.to_string())
}
