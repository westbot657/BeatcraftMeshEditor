use std::env;
use std::path::PathBuf;


fn main() {
    println!("cargo:rustc-link-lib=openal");

    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .allowlist_function("al[A-Z].*")
        .allowlist_function("alc[A-Z].*")
        .allowlist_var("AL_.*")
        .allowlist_var("ALC_.*")
        .allowlist_type("AL[C]?.*")
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .generate()
        .expect(
            "Unable to generate OpenAL bindings - check that libopenal-dev \
                 (or equivalent) is installed and AL/al.h is on the include path",
        );

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("al_bindings.rs"))
        .expect("Couldn't write bindings");



}
