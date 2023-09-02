/* BUILD.RS */

/* 
fn main() {
    // build script output is not printed to the terminal unless
    // the build script fails
    dbg!(std::env::var("OUT_DIR"));
    //panic!();

    // side note: can consider using CARGO_TARGET_DIR for all your projects
    // in a single host, might save compilation times and space

    // can cat the build folder to see the stderr from the build script
    // target/debug/build/build-and-ffi-da846e62c4976855/stderr

    // prints out user stuff as warning
    println!("cargo:warning=blabla");

    // in toml file, if you want to link against a shared library, please do below
    // you still have to use rustc- stuff to link it, but this prevents other
    // crates from linking to it
    /*
     [package]
     links = "foo"
     */

    // take a look at libgit2 for build.rs example

    // in the open source world, the suffix -sys is for pure binding and explose
    // the headers via FFI, ie, libsodium-sys
    // other crates can use the -sys stuff to generate nicer stuff
    
    // for example if you are doing openssl and the var OPEN_SSL_DIR is changed
    // then you need to rebuild
    // println!("cargo:rerun-if-changed=VAR")

    // println!("cargo:rustc-cfg=libgit2_vendored");
    // this line means they can be used #cfg(libgit2_vendored) in their source files

    // pub enum git_blob {} 
    // pub enum means that this is a opaque type that we are only ever going to 
    // use to pass around, we don't care about the internals 
    // why it is an enum instead of an empty struct? because we don't want
    // someone to be able to construct an instance of it. 
    // empty enums are impossible to create in rust

    // bindgen takes the c header file and generates the equivalent rust bindings
    // though, new revisions of bindgen may not generate the same bindings given the same header file
    // so you may have to do a new release and get a new version, especially if it is a -sys crate
    // which makes the user of the -sys crate update their versions as well

}
*/

use std::env;
use std::path::PathBuf;

fn main() {

    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    println!("cargo:rustc-link-lib=sodium");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

/* BUILD.RS */