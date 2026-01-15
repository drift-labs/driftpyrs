use std::env;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_DRIFT_FFI_PATH");

    let lib_dir = env::var("CARGO_DRIFT_FFI_PATH").unwrap_or_else(|_| {
        if cfg!(target_os = "macos") {
            "/usr/local/lib".to_string()
        } else {
            "".to_string()
        }
    });

    if lib_dir.is_empty() {
        panic!(
            "CARGO_DRIFT_FFI_PATH is not set. Set it to a directory containing libdrift_ffi_sys.{ext}",
            ext = if cfg!(target_os = "macos") { "dylib" } else { "so" }
        );
    }

    let lib_dir = Path::new(&lib_dir);
    if cfg!(target_os = "macos") {
        let dylib = lib_dir.join("libdrift_ffi_sys.dylib");
        if !dylib.exists() {
            panic!(
                "Missing {}. Either set CARGO_DRIFT_FFI_PATH to the drift-ffi-sys target dir, or install/symlink it into /usr/local/lib",
                dylib.display()
            );
        }
    }
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=dylib=drift_ffi_sys");

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
    }
}
