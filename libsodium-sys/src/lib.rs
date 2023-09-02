#![feature(maybe_uninit_slice)]

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[no_mangle] // no mangle keeps the name exactly as it is in the binary
extern "C" fn this_is_rust(arg: std::os::raw::c_int) -> std::os::raw::c_char {
    b'x' as i8
}

// this is C:
// extern char this_is_rust(int);

use std::mem::MaybeUninit;

// static HAS_BEEN_INIT: OnceCell<bool> = OnceCell::new(false);
// above can be used to checked whether the sodium_init func has been called
// by using assert!(HAS_BEEN_INIT) in every calling func since it can only be set once
// problem is this is checked at runtime, not compile time

#[non_exhaustive]
// non exhaustive means external users cannot construct or destruct one of these
// struct, except by using our constructor and destructors
#[derive(Clone, Copy, Debug)]
pub struct Sodium;

impl Sodium {
    pub fn new() -> Result<Self, ()> {
        if unsafe { ffi::sodium_init() } < 0 {
            Err(())
        } else {
            Ok(Self)
        }
    }

    pub fn crypto_generichash<'a>(
        self,
        input: &[u8],
        key: Option<&[u8]>,
        out: &'a mut [MaybeUninit<u8>],
    ) -> Result<&'a mut [u8], ()> {
        // better to use 'usize::from' (or try_from) than 'as usize' for "casting"
        assert!(out.len() >= usize::try_from(ffi::crypto_generichash_BYTES_MIN).unwrap());
        assert!(out.len() <= usize::try_from(ffi::crypto_generichash_BYTES_MAX).unwrap());
        if let Some(key) = key {
            assert!(key.len() >= usize::try_from(ffi::crypto_generichash_KEYBYTES_MIN).unwrap());
            assert!(key.len() <= usize::try_from(ffi::crypto_generichash_KEYBYTES_MAX).unwrap());
        }
        let (key, keylen) = if let Some(key) = key {
            (key.as_ptr(), key.len())
        } else {
            (std::ptr::null(), 0)
        };
        // SAFETY: We've checked the requirements of the function of the function (MIN/MAX)
        // and the presence of self means that init has been called
        let res = unsafe { 
            ffi::crypto_generichash(
                MaybeUninit::slice_as_mut_ptr(out),
                out.len() as usize,
                input.as_ptr(),
                input.len() as u64,
                key,
                keylen as usize,
            )
        };

        if res < 0 {
            return Err(());
        }

        // SAFETY: crypto_generic_hash writes to (and thus all initializes all the bytes of out)
        Ok(unsafe { MaybeUninit::slice_assume_init_mut(out) })    
    }
}

pub use ffi::sodium_init;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        Sodium::new().unwrap();
    }

    #[test]
    fn it_hashes() {
        let s = Sodium::new().unwrap();
        let mut out = [MaybeUninit::uninit(); ffi::crypto_generichash_BYTES as usize];
        let bytes = s.crypto_generichash(b"Arbitary data to hash2", None, &mut out).unwrap();
        println!("{}", hex::encode(bytes));
    }
    
}

// can use AutoCfg for newer rust compiler options, for example for nightly features