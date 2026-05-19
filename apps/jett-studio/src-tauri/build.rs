fn main() {
    tauri_build::build();

    // Link to the bundled SimConnect import library
    println!("cargo:rustc-link-search=native=lib");
    println!("cargo:rustc-link-lib=dylib=SimConnect");
    println!("cargo:rerun-if-changed=lib/SimConnect.dll");

    // Copy SimConnect.dll next to the output binary so Windows can find it at runtime.
    // OUT_DIR = target/{profile}/build/jett-studio-{hash}/out  →  3 levels up = target/{profile}
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let binary_dir = std::path::Path::new(&out_dir)
        .ancestors()
        .nth(3)
        .expect("Could not determine binary output directory");

    let dll_src = std::path::Path::new("lib/SimConnect.dll");
    let dll_dst = binary_dir.join("SimConnect.dll");

    if dll_src.exists() {
        std::fs::copy(dll_src, &dll_dst)
            .expect("Failed to copy SimConnect.dll to output directory");
    }
}
