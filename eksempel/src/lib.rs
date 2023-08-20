mod one {
    pub fn strlen(s: impl AsRef<str>) -> usize {
        s.as_ref().len()
    }

    /*
    for the above, at compile time, complier generates 2 types of functions
    pub fn strlen_refstr(s: &str) -> usize {
        s.len()
    }

    pub fn strlen_string(s: String) -> usize {
        s.len()
    }
    */

    /*
    if you have 2 versions of the same function (because of the genertic types),
    instruction caching may not be efficient, because there's a possibility of a consumer
    calling the function with different types that will make the program jump to different
    parts of the instruction cache
    */

    pub fn strlen2<S: AsRef<str>>(s: S) -> usize
    {
        s.as_ref().len()
    }

    pub fn foo() {
        strlen("hello world"); // &'static str
        strlen(String::from("hei verden")); // String: implements AsRef<str>
    }

    // pub fn strlen_dyn(s: dyn AsRef<str>). This does not COMPILE!
    pub fn strlen_dyn(s: &dyn AsRef<str>) -> usize {
        s.as_ref().len()
    }

    // can put it inside a Box as well 
    pub fn strlen_dynbox(s: Box<dyn AsRef<str>>) -> usize {
        s.as_ref().as_ref().len()
    }

    pub fn caller() {
        let x: Box<dyn AsRef<str>> = Box::new(String::from("hello"));
        strlen_dynbox(x);

        let y: &dyn AsRef<str> = &"world";
        strlen_dyn(y);
    }

    /*
    struct Box<T: ?Sized>. ?Sized means T does not have to be sized

    when to close Box over &?
    Box is static so can keep using it
    */
}

mod two {
    pub trait Hei {
        fn hei(&self);
    }

    impl Hei for &str {
        fn hei(&self) {
            println!("hei {}", self);
        }
    }  

    impl Hei for String {
        fn hei(&self) {
            println!("hei {}", self);
        }
    }

    fn foo2() {
        "J".hei();
    }

    // func below is equivalent to 
    // bar_old`<H: Hei>(h: H)
    fn bar_old(h: impl Hei) {
        h.hei();
    }

    fn foo() {
        bar_static(&["J", "Jon"]);
        bar_static(&[String::from("J"), String::from("Jon")]);
        // The below doesn't compile, can't have another type
        // bar(&[String::from("J"), "Jon"]);
    }

    // impl is a shortcut for
    // bar<H: Hei>(s: &[H])
    // also, bar is generic over one type
    fn bar_static(s: &[impl Hei]) {
        for h in s {
            h.hei();
        }
    }

    // static dispatch gives concrete types
    fn bar_static_str(h: &str) {
        h.hei();
    }

    // this results in compiler error:
    // size of values of type dyn Hei cannot be known at compilation time
    // fn bar_dynamic(s: &[dyn Hei]) {
    //     for h in s {
    //         h.hei();
    //     }
    // }

    /* these are not sized
    struct Foo { s: [u8] } // notice there's no reference, NOT &[u8]. Becase we don't know how long u8 is
    struct Foo { s: str } // notice there's no reference, NOT &[str]. Becase we don't know how long str is
    */

    // &dyn Hei
    // stored in &
    // 1. a pointer to the actual, concrete implementing type
    // 2. a pointer to the vtable for the referenced trait
    // 
    // what is a vtable? 
    // 
    // dyn Hei, vtable:
    //
    // struct HeiVtable {
    //    hei: *mut Fn(&mut ()),
    // }
    // each type has its own distint vtable
    
    // &str -> &dyn Hei (wide/fat pointer)
    // 1. pointer to the &str
    // 2. &HeiVtable {
    //      hei: &<str as Hei>::hei // Points to Line 65
    //}

    // 1 & 2 are known at fat pointers because they have 2 pointers and not just one
    // 2 will be known statically

    // &String -> &dyn Hei 
    // 1. pointer to the String
    // 2. &HeiVtable {
    //      hei: &<String as Hei>::hei // Points to Line 71
    //}

    // s.hei()
    // -> s.vtable.hei(s.pointer)

    fn say_hei_dynamic(s: &dyn Hei) {
        s.hei(); // Call ??
    }

    // The below DOES not compile. Only auto traits can be used as additional traits
    // fn baz(s: &(dyn Hei + AsRef<str>)) {
    //     s.hei();
    //     let x = s.as_ref().len();
    //     s.len();
    // }

    trait HeiAsRef: Hei + AsRef<str> {}
    
    fn baz(s: &dyn HeiAsRef) {
        s.hei();
        let x = s.as_ref().len();
    }
}

mod three {
    pub trait Hei {
        fn hei(&self);
        // NOTE does not have self parameter!
        //fn weird(); 
    }

    impl Hei for &str {
        fn hei(&self) {
            println!("hei {}", self);
        }
    }  

    impl Hei for String {
        fn hei(&self) {
            println!("hei {}", self);
        }
    }

    fn say_hei(s: &dyn Hei) {
        // s.weird();
        // this is like calling  (dyn Hei)::weird();
        // which doenst really make sense, because
        // which type are we calling weird on?
    }   
}

// Resolves mod three
mod three_plus {
    pub trait Hei {
        fn hei(&self);
        // NOTE does not have self parameter!
        //fn weird();

        // This is telling the compiler that this function shouldnt be placed
        // in the vtable 
        // and shouldn't be callable through a trait object
        fn weird(&self) where Self: Sized {

        }
    }

    impl Hei for &str {
        fn hei(&self) {
            println!("hei {}", self);
        }
    }  

    impl Hei for String {
        fn hei(&self) {
            println!("hei {}", self);
        }
    }

    fn say_hei(s: &dyn Hei) {
        s.hei();
        // DOES NOT compile. The weird method cannot be invoked on a trait object
        //s.weird();
    }   
}

mod four {
    use std::iter::Extend;
    // The extend discussion of generics over iterator type is complicated
    // TODO: revisit when there's bottom up context

    // This DOES NOT compile. The trait cannot be made into an object!
    // fn clone(&self) -> Self
    // takes a reference and returns the concrete type
    // so the clone func below is returning dyn Clone (not &dynClone)
    // which is not Sized, therefore function does not compile
    // pub fn clone(v: &dyn Clone) {
    //     let x = v.clone();
    //     
    // }

    fn drop(v: &mut dyn Drop) {
        // when v goes out of scope, Drop::drop is called
    }

    fn say_hei(s: Box<dyn AsRef<str>>) {
        // what happens when s goes out of scope?
        // by right we'll have to free the memory, but the only thing we can do
        // is as_ref() because of the trait

        // every vtable includes Drop!
        // think of it as an implicit drop,ie Box<dyn AsRef<str> + Drop
    }
}

mod five{
    // * means when turned into a pointer
    // fat pointers have metadata
    // dynTrait -> * -> (*mut data, *mut vtable)
    // [u8]     -> * -> (*mut data, usize length) 
    // [str]     -> * -> (*mut data, usize length)

    // below does not COMPILE, so use a reference
    //fn foo(s: [u8]) {}
    fn foo(s: &[u8]) {}

    // below does not COMPILE, so use a Box
    // fn bar() -> [u8] {
    //     [][..]
    // }
    fn bar() -> Box<[u8]> {
        Box::new([]) as Box<[u8]>
    }

    // below does not COMPILE, but if [u8] is at the end, it is ok
    // struct Foo {
    //     f: bool,
    //     t: [u8],
    //     x: bool,
    // }
    struct Foo {
        f: bool,
        x: bool,
        t: [u8],
    }

    /*
        dyn Fn != fn
        fn is a function pointer, can't pass closure in to a func that expects fn
        fn foo2(f: &dyn Fn()) {}
        fn bar2(f: fn()) {}

        fn caller() {
            let x = "Hello";
            foo2(&|| {
                let _ = &x;
            });
            // below DOES not compile
            // bar2(&|| {
            //     let _ = &x;
            // });
        }

        TODO: revisit discussion on &dynFn() vs impl Fn(), when bottom up context appears
    */

    fn say_his(v: &[&dyn AsRef<str>]) {
        for s in v {
            s.as_ref();
        }
    }


}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
