use std::env;
use std::path::PathBuf;

fn main() {
    // Link against the system OpenAL Soft library.
    // On Linux this is typically `libopenal.so`, linked via `-lopenal`.
    println!("cargo:rustc-link-lib=openal");
 
    // If OpenAL is in a nonstandard location, uncomment and adjust:
    // println!("cargo:rustc-link-search=native=/usr/local/lib");
 
    println!("cargo:rerun-if-changed=wrapper.h");
 
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // Only pull in AL_/ALC_ prefixed items, keeps the output small
        // and avoids accidentally slurping unrelated system headers.
        .allowlist_function("al[A-Z].*")
        .allowlist_function("alc[A-Z].*")
        .allowlist_var("AL_.*")
        .allowlist_var("ALC_.*")
        .allowlist_type("AL[C]?.*")
        // Constants like AL_PITCH/AL_BUFFER come through as #define's;
        // this makes bindgen emit them as rust consts instead of trying
        // (and sometimes failing) to infer C enum types.
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .generate()
        .expect("Unable to generate OpenAL bindings - check that libopenal-dev \
                 (or equivalent) is installed and AL/al.h is on the include path");
 
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("al_bindings.rs"))
        .expect("Couldn't write bindings");
}
