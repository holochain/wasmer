const WAMR_ZIP: &str = "https://github.com/bytecodealliance/wasm-micro-runtime/archive/refs/tags/WAMR-2.1.0.zip";
const WAMR_DIR: &str = "wasm-micro-runtime-WAMR-2.1.0";

fn main() {
    #[cfg(feature = "wamr")]
    {
        use cmake::Config;
        use std::{env, path::PathBuf};

        let crate_root = env::var("CARGO_MANIFEST_DIR").unwrap();

        // Read target os from cargo env
        // Transform from cargo value to valid wasm-micro-runtime os
        let target_os = match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
            "linux" => "linux",
            "windows" => "windows",
            "macos" => "darwin",
            "freebsd" => "freebsd",
            "android" => "android",
            "ios" => "ios",
            other => panic!("Unsupported CARGO_CFG_TARGET_OS: {}", other),
        };

        // Read target arch from cargo env
        // Transform from cargo value to valid wasm-micro-runtime WAMR_BUILD_TARGET
        let target_arch = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
            "x86" => "X86_32",
            "x86_64" => "X86_64",
            "arm" => "ARM",
            "aarch64" => "AARCH64",
            "mips" => "MIPS",
            "powerpc" => "POWERPC",
            "powerpc64" => "POWERPC64",
            other => panic!("Unsupported CARGO_CFG_TARGET_ARCH: {}", other),
        };
        
        // Cleanup tmp data from prior builds      
        let wamr_dir = PathBuf::from(&crate_root).join("third_party/wamr");
        let zip_dir = PathBuf::from(&crate_root).join(WAMR_DIR);
        let _ = std::fs::remove_dir_all(&wamr_dir);
        let _ = std::fs::remove_dir_all(&zip_dir);

        // Fetch & extract wasm-micro-runtime source
        let zip = ureq::get(WAMR_ZIP).call().expect("failed to download wamr");
        let mut zip_data = Vec::new();
        zip.into_reader().read_to_end(&mut zip_data).expect("failed to download wamr");
        zip::read::ZipArchive::new(std::io::Cursor::new(zip_data))
            .expect("failed to open wamr zip file")
            .extract(&crate_root)
            .expect("failed to extract wamr zip file");
        let _ = std::fs::remove_dir_all(&wamr_dir);
        std::fs::rename(zip_dir, &wamr_dir).expect("failed to rename wamr dir");

        /*
        let mut fetch_submodules = std::process::Command::new("git");
        fetch_submodules
            .current_dir(crate_root)
            .arg("submodule")
            .arg("update")
            .arg("--init");

        let res = fetch_submodules.output();

        if let Err(e) = res {
            panic!("fetching submodules failed: {e}");
        }
        */

        let wamr_platform_dir = wamr_dir.join("product-mini/platforms").join(target_os);
        let dst = Config::new(wamr_platform_dir.as_path())
            .always_configure(true)
            .generator("Unix Makefiles")
            .no_build_target(true)
            .define(
                "CMAKE_BUILD_TYPE",
                if cfg!(debug_assertions) {
                    "RelWithDebInfo"
                } else {
                    "Release"
                },
            )
            .define("WAMR_BUILD_AOT", "0")
            //.define("WAMR_BUILD_TAIL_CALL", "1")
            //.define("WAMR_BUILD_DUMP_CALL_STACK", "1")
            // .define("WAMR_BUILD_CUSTOM_NAME_SECTION", "1")
            // .define("WAMR_BUILD_LOAD_CUSTOM_SECTION", "1")
            .define("WAMR_BUILD_BULK_MEMORY", "1")
            .define("WAMR_BUILD_REF_TYPES", "1")
            .define("WAMR_BUILD_SIMD", "1")
            .define("WAMR_BUILD_FAST_INTERP", "1")
            .define("WAMR_BUILD_LIB_PTHREAD", "1")
            .define("WAMR_BUILD_LIB_WASI_THREADS", "0")
            .define("WAMR_BUILD_LIBC_WASI", "0")
            .define("WAMR_BUILD_LIBC_BUILTIN", "0")
            .define("WAMR_BUILD_SHARED_MEMORY", "1")
            .define("WAMR_BUILD_MULTI_MODULE", "0")
            .define("WAMR_DISABLE_HW_BOUND_CHECK", "1")
            .define("WAMR_BUILD_TARGET", target_arch)
            .build();

        // Check output of `cargo build --verbose`, should see something like:
        // -L native=/path/runng/target/debug/build/runng-sys-abc1234/out
        // That contains output from cmake
        println!(
            "cargo:rustc-link-search=native={}",
            dst.join("build").display()
        );
        println!("cargo:rustc-link-lib=vmlib");

        let bindings = bindgen::Builder::default()
            .header(
                wamr_dir
                    .join("core/iwasm/include/wasm_c_api.h")
                    .to_str()
                    .unwrap(),
            )
            .header(
                wamr_dir
                    .join("core/iwasm/include/wasm_c_api.h")
                    .to_str()
                    .unwrap(),
            )
            .header(
                wamr_dir
                    .join("core/iwasm/include/wasm_export.h")
                    .to_str()
                    .unwrap(),
            )
            .derive_default(true)
            .derive_debug(true)
            .generate()
            .expect("Unable to generate bindings");
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings");
    }
}
