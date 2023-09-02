/* MAIN.RS */

mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

fn main() {
    println!("Hello, world!");
    // env! macro read env variables as set at compile time not at run time
    println!("{}", env!("OUT_DIR"));

    println!("{}", concat!(env!("OUT_DIR"), "/bindings.rs"));

    // cfg! is used for compile time properties, to check if a feature is available or not
    // for example, to check whether feature hello exists #[cfg(feature = "hello")]
    // and do conditional compilation (for example)
    // println!("{}", cfg!("hello"));

    // side note: consider installing cargo expand to look at macros after expansion

}

/* MAIN.RS */
