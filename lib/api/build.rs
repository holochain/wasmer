// const WAMR_ZIP: &str = "https://github.com/bytecodealliance/wasm-micro-runtime/archive/refs/tags/WAMR-2.1.0.zip";
const WAMR_ZIP: &str = "https://github.com/mattyg/wasm-micro-runtime/archive/refs/tags/4.zip";
//const WAMR_DIR: &str = "wasm-micro-runtime-WAMR-2.1.0";
const WAMR_DIR: &str = "wasm-micro-runtime-4";

fn main() {
    #[cfg(feature = "wamr")]
    {
        use cmake::Config;
        use std::{env, path::PathBuf};

        let crate_root = env::var("CARGO_MANIFEST_DIR").unwrap();
        let wamr_dir = PathBuf::from(&crate_root).join("third_party").join("wamr");

        let zip = ureq::get(WAMR_ZIP).call().expect("failed to download wamr");

        let mut zip_data = Vec::new();
        zip.into_reader().read_to_end(&mut zip_data).expect("failed to download wamr");

        zip::read::ZipArchive::new(std::io::Cursor::new(zip_data))
            .expect("failed to open wamr zip file")
            .extract(&crate_root)
            .expect("failed to extract wamr zip file");

        let _ = std::fs::remove_dir_all(&wamr_dir);

        let zip_dir = PathBuf::from(&crate_root).join(WAMR_DIR);

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

        let mut cmake_config = Config::new(wamr_dir.clone());
        let mut dst_config = cmake_config
            .always_configure(true)
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
            .define("WAMR_BUILD_LIB_PTHREAD", "1")
            .define("WAMR_BUILD_LIB_WASI_THREADS", "0")
            .define("WAMR_BUILD_LIBC_WASI", "0")
            .define("WAMR_BUILD_LIBC_BUILTIN", "0")
            .define("WAMR_BUILD_SHARED_MEMORY", "1")
            .define("WAMR_BUILD_MULTI_MODULE", "0")
            .define("WAMR_DISABLE_HW_BOUND_CHECK", "1");
        if cfg!(not(target_os = "windows")) {
            dst_config = dst_config.generator("Unix Makefiles");
        }
        if cfg!(feature = "wamr-fast-interp") {
            dst_config = dst_config.define("WASM_ENABLE_FAST_INTERP", "1");
        }
        let dst = dst_config.build();

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
