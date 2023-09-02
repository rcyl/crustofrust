use std::env;
use std::path::PathBuf;

// the crate ctor allows a function to be ran before main
// useful for functions that has to be ran once
// the downside is there is no good way to report an error to user with this
// to the user
// #[ctor]
// fn foo() {
//     println!("Hello world")
// }

fn main() {
    // don't need to tell compiler where to find the library with pkg-config
    pkg_config::Config::new()
        .print_system_libs(false) // supresses '-L /usr/lib' from being emitted in the build process std output
        .atleast_version("1.0.18")
        .probe("libsodium")
        .unwrap();

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .allowlist_function("sodium_init")
        .allowlist_function("crypto_generichash")
        .allowlist_var("crypto_generichash_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
