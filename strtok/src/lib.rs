mod one {
    pub fn strtok<'a>(s: &'a mut &'a str, delimeter: char) -> &'a str {
        if let Some(i) = s.find(delimeter) {
            let prefix = &s[..i];
            let suffix = &s[(i + delimeter.len_utf8())..];
            *s = suffix;
            prefix
        } else {
            let prefix = *s;
            *s = "";
            prefix
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // #[test]
        // fn it_does_not_work() {
        //     let mut x = "hello world"; // mut x: &str
        //                                // as long as hello lives, x continues to be mutable borrowed
        //                                // since x (the arg) and hello (the return value) have the same lifetime
        //                                // so, try to put it in separate scope, see test it_works2()
        //     let hello = strtok(&mut x, ' '); // hello: &str
        //     assert_eq!(hello, "hello");
        //     assert_eq!(x, "world");
        // }

        // #[test]
        // fn it_does_not_work2() {
        //     let mut x = "hello world"; // mut x: &str
        //     {
        //         let hello = strtok(&mut x, ' '); // hello: &str
        //         assert_eq!(hello, "hello");
        //         // hello, is gone out of scope at the end of this
        //         // so x should not be borrowed as mut anymore
        //         // but yet still can't compile
        //     }
        //     assert_eq!(x, "world");
        // }
    }
}

mod two {
    pub fn strtok<'a>(s: &'a mut &'a str, delimeter: char) -> &'static str {
        if let Some(i) = s.find(delimeter) {
            let prefix = &s[..i];
            let suffix = &s[(i + delimeter.len_utf8())..];
            *s = suffix;
            // prefix
            ""
        } else {
            let prefix = *s;
            *s = "";
            // prefix
            ""
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // #[test]
        // fn it_does_not_work() {
        //     let mut x = "hello world"; // mut x: &str
        //     {
        //         let hello = strtok(&mut x, ' '); // hello: &str
        //         assert_eq!(hello, "hello");
        //         // strtok still fails to compile though strtok is now returning 'static
        //         // so it is not related to the lifetime
        //         // if fact, even if the the func does not return anything,
        //         // it still does not compile
        //     }
        //     assert_eq!(x, "world");
        // }

        // #[test]
        // fn it_does_not_work2() {
        //     let mut x = "hello world"; 
        //     // &'a mut &'a      str (this is our func prototype)
        //     // &   mut &'static str (this is the type of x)
        //     // the compiler deduces that 'a is 'static 
        //     // the complier deduces that x should have a 'static lifetime
        //     // x cannot have a 'static lifetime since it is a stack variable
        //     // therefor compilation error 
        //     let hello = strtok(&mut x, ' '); 
        //     assert_eq!(x, "world");
        // }

        #[test]
        fn it_works() {
            let mut x = "hello world"; 
            let hello = strtok(&mut x, ' '); 
            // assert_eq!(x, "world");
            
            // commenting the above will make the code compile
            // which is weird considering that "hello world" is still 'static
            // this is because the compiler can use 'static in place of anything
            // that has a shorter lifetime reference
            // ie, anything that has a static lifetime is valid in any context
            // that takes a non static lifetime, because the static lifetime is longer
            // than any non static lifetime
        }

        // #[test]
        // fn it_does_notwork3() {
        //     fn check_is_static(_: &'static str) {}
            
        //     let mut x = "hello world"; 
        //     check_is_static(x);
        //     let hello = strtok(&mut x, ' '); 
        //     // assert_eq!(x, "world");

        //     // it does not compile now, because the compiler 
        //     // can no longer shorten the liftime of x to be 'it_does_notwork3
        //     // because we passed it into a func that forces x to be 'static
        // }

        fn subtype_example() {
            let s = String::new();
            let x: &'static str = "hello world";
            let mut y /* : &'a */ = &*s; // lifetime of y is &'a
            y = x; // y is assigned to x which has 'static lifetime

            // This works because 'static is a subtype of any lifetime 'a
            // T is a subtype of U (T: U)
            // T is at least as useful as U (not formal definion)
         }

        // 3 types of variance: 
        // 1. covariance -> most things
        // 2. contravariance
        // 3. invariance

        fn covariance<'a>(_:&'a str) {
            // usage example below, both would work
            // covariance(&'a str)
            // covariance(&'static str)
        }

        // Revisit when have bottom up context
        fn contravariance() {
            // fn foo(bar: Fn(&'a str) -> ()) {
            //     bar("hello world")
            // }
            // let x: Fn(&'a str) -> ()
            // foo(fn(&'static str) {})
            
            // &'static str (more useful)
            // &'a str
            // 'static <: 'a (static a subtype of 'a)
            // &'static T <: &'a T 

            // Fn(&'static str) (stricter requirements)
            // Fn(&'a str) (more useful since takes a shorter lifetime)
            // Therefore Fn is contravariant
            // 'static <: 'a (static a subtype of 'a)
            // Fn(&'a T) <: Fn(&'static T)
        }

        fn invariance() {
            // fn foo(s: &mut &'s str, x: &'a str) {
            //     *s = x // s which is a refence to 's str is made to point to 'a str
            // }

            // let mut x: &'static str = "Hello world";
            // let z = String::new();
            // foo(&mut x, &z); // we made the pointer to 'static str point to a 'a str
            // foo(&mut &'a      str, &'a str)
            // foo(&mut &'static str, &'a str)
            // mutable references are invariant in T (in this case, reference to a str) 
            // so the lifetimes must be the equal for T and the code DOES NOT compile
            // because z is only valid for the context of the func and the compiler
            // infers the lifetimes to be 'static
            // drop(z);
            // println!("{}", x);

            // let mut y = true;
            // let mut z /* &'y mut bool */ = &mut y; // z is a mut reference to y
            
            // let x = Box::new(true);
            // let x: &'static mut bool = Box::leak(x); // Box leak used to return &'static mut bool

            // z = x; // &'y mut bool = &'static mut bool 
            // This is ok because mutable references are covariant in their lifetimes, 
            // but only invariant in their T (in this case bool)
        }
    }
}

mod three {
    pub fn strtok<'a>(s: &'a mut &'a str, delimeter: char) -> &'a str {
        if let Some(i) = s.find(delimeter) {
            let prefix = &s[..i];
            let suffix = &s[(i + delimeter.len_utf8())..];
            *s = suffix;
            prefix
        } else {
            let prefix = *s;
            *s = "";
            prefix
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // #[test]
        // fn it_does_not_work() {
        //     pub fn strtok<'a>(s: &'a mut &'a str, delimeter: char) -> &'a str

        //     fn check_is_static(_: &'static str) {}

        //     let mut x = "hello world"; 
        //     check_is_static(x);     
        //     strtok(&mut x, ' '); 

        //     (arg to stroktok) &'a mut &'a      str 
        //                       (arg passed into trtok)
        //     (&mut x)          &'x mut &'static str 
        //                       (args of &mut x, we forced the x to be 'static 
        //                        when we passed it into check_is_static)
        //     lifetime of the arg passed into strtok 
        //     is &'static mut &'static str (no fancy compiler stuff, this is because of the signature)
        //     
        //     since mut x lives for liftime 'x, 
        //     the compiler would like to change the lifetime of T to match since the signature 
        //     is the same but it can't do so for the T because of invariance of mutable references, 
        //     but it can do so for the lifetime which is covariant for mutable references
        //     which is why the code does not compile
        // }
    }
}

mod four {
    // Add another lifetime so that the compiler is able to choose
    // them separately when calling the function
    pub fn strtok<'a, 'b>(s: &'a mut &'b str, delimeter: char) -> &'b str {
        if let Some(i) = s.find(delimeter) {
            let prefix = &s[..i];
            let suffix = &s[(i + delimeter.len_utf8())..];
            *s = suffix;
            prefix
        } else {
            let prefix = *s;
            *s = "";
            prefix
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn it_works() {
            let mut x = "hello world";

            // this still compiles though mut x is borrowed later
            // because compiler is allowed to shorten the lifetimes
            let z = &mut x; // &'x mut -> &'until-ZZZ mut
                            // until-ZZZ: borrow of x stops here

            // strtok<'a, 'b> &'a mut &'b      str (func arg of strtok)
            //                &'a mut &'b str 
            // compiler can assign 'b to 'static and 'a to 'it_works
            let hello = strtok(&mut x, ' '); 

            assert_eq!(hello, "hello");
            assert_eq!(x, "world");
        }
    }
}

mod five {
    // You use PhantomData because you have a type that is generic over T
    // but doesn't actually contain a T, for example a deserializer
    use std::marker::PhantomData;
    
    struct Deserializer<T> {
        // drop check will drop the T
        _t: PhantomData<T>,
    }

    struct Deserializer2<T> {
        // signature indicates that a T is not owned, so 
        // drop check can't drop the T
        // this is covariant
        _t: PhantomData<fn() -> T>,
        // Below is another way to specifying covariance
        // but people get scared when they see raw pointers
        // so they use the first defintion instead
        // also certain traits, such as send/sync are not 
        // implemented for raw pointers
        _t2: PhantomData<*const T>,
    }

    struct Deserializer3<T> {
        // this is contravariant, so it won't be able to shorten lifetimes
        _t1: PhantomData<fn(T)>,
    }

    struct Deserializer4<T> {
        // The function signature includes that struct is 
        // both covariant and contravariant but no type can be both
        // therefore compiler concludes that this is invariant
        _t1: PhantomData<fn(T)>,
        _t2: PhantomData<fn() -> T>,

        // You could also do this for invariance, 
        // but now this requires a lifetime
        // _t3: PhantomData<&'a mut T>,

        // This works as well to indicate that it is invariant
        _t4: PhantomData<*mut T>,

        // This works as well to indicate that it is invariant
        _t5: PhantomData<fn(T) -> T>,
    }

    // There are different function signatures
    // because of the drop check, compiler needs to know
    // whether T can be dropped as well

    fn five_main() {
        let x = String::new();
        let z = vec![&x];
        drop(x);
        //drop(z); // z is never accessed again, so it is ok to drop the x
    }

    // This way is fine
    pub fn strtok<'s>(s: &mut &'s str, delimeter: char) -> &'s str {
        if let Some(i) = s.find(delimeter) {
            let prefix = &s[..i];
            let suffix = &s[(i + delimeter.len_utf8())..];
            *s = suffix;
            prefix
        } else {
            let prefix = *s;
            *s = "";
            prefix
        }
    }

    // But jon prefers to be explicit with inferred lifetimes
    pub fn strtok2<'s>(s: &'_ mut &'s str, delimeter: char) -> &'s str {
        if let Some(i) = s.find(delimeter) {
            let prefix = &s[..i];
            let suffix = &s[(i + delimeter.len_utf8())..];
            *s = suffix;
            prefix
        } else {
            let prefix = *s;
            *s = "";
            prefix
        }
    }

}