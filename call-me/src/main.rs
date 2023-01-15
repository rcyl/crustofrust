mod one {
    pub fn one() {
        println!("Hello, world!");
        // x is actually a function item, not a function pointer
        //                  (fn item is term used by compiler)
        let x = bar::<i32>;
        // this prints zero, no pointer is held
        println!("{}", std::mem::size_of_val(&x));

        baz(bar::<u32>);
        // compiler coerces the function item type to a function pointer type
        // so that this function can be called
        baz(bar::<i32>);
    }

    fn bar<T>(_: u32) -> u32 {0}

    // This fn here is an function pointer while Fn (capital F) is a trait
    fn baz(f: fn(u32) ->  u32) {
        // this prints 8, because fn passed in are 2 different functions
        println!("{}", std::mem::size_of_val(&f));
    }
}

/*
    Function pointer/item has no state. They are standalone chunks of code
    that does not reference anything in other stack frames. They dont reference
    all memory that is stored outside of themselves. There is no lifetime associated
    with them. This means, they don't really care about self

    Therefore, function pointers implement all three Fn traits, Fn, FnMut, FnOnce

    According to function defintions
    Fn -> take a shared reference to self (&self)
    FnMut -> take a mut reference to self (&mut self)
    FnOnce -> take self (self)

    Since both FnMut and FnOnce are supertraits of Fn, 
    any instance of Fn can be used as a parameter where a FnMut or FnOnce is expected.

    Since FnOnce is a supertrait of FnMut, 
    any instance of FnMut can be used where a FnOnce is expected,
    and since Fn is a subtrait of FnMut, any instance of Fn can be used where FnMut is expected.

    Since both Fn and FnMut are subtraits of FnOnce, 
    any instance of Fn or FnMut can be used where a FnOnce is expected.
    
    FnOnce -> FnMut > Fn (in terms of how strigent)

*/

mod two {
    fn quox<F>(mut f: F)
    where 
        F: FnMut() 
    {
        (f)()
    }

    fn bar<T>() {}

    fn baz(f: fn()) {
        // this prints 8, because fn passed in are 2 different functions
        //println!("{}", std::mem::size_of_val(&f));
    }

    pub fn two() {
        // quox(bar::<i32>);
        // Non capturing closure can be coerced into function signatures
        let f = || ();
        baz(f);
    }

    pub fn three() {
        let mut z = String::new();
        // This closure is FnMut because of the mut z passed in it
        let f = || {
            z.clear();
        };
        quox(f);
    }
    
}

mod play {
    //
    fn quox<F>(f: F)
    where 
        F: FnOnce() 
    {
        (f)()
    }

    pub fn play() {
        let mut z = String::new();
        // This closure is Fn
        // We can pass a Fn closure into a function
        // that expects FnOnce but not the other way around
        let f = || {
            //z.clear();
        };
        quox(f);
    }
}

mod three {
    fn quox<F>(f: F)
    where 
        F: FnOnce() 
    {
        (f)()
    }

    pub fn three() {
        let mut z = String::new();
        // This closure is FnOnce because z is dropped
        let f = || {
            drop(z);
        };
        quox(f);
    }
}

mod drop {
    fn quox<F>(f: F)
    where 
        F: Fn() 
    {
        (f)()
    }

    pub fn bla_drop() {
        let mut z = String::new();
        let f = move || {
            // from this println itself, the compiler infers that z is a shared
            // reference.
            println!("{}", z);
            // When we add the "move" keyword, we are telling the compiler to
            // move it inside so that z will be dropped
            // z is dropped here 
        };
        quox(f);
    }

    fn make_fn() -> impl Fn() {
        let z = String::new();
        // this wouldn't work without move
        move || {
            println!("{}", z);
        }
    }

    fn move_example() -> impl Fn() {
        let x = String::new();
        let z = String::new();
        {
            //let x = &x; // Tell the compiler that you just wanna move the reference to x in the closure
            move || {
                println!("{}", x);
                println!("{}", z);
            }
        }
        
    }
}

mod dynamic_dispatch {
    fn quox<F>(mut f: F)
    where 
        F: FnMut() 
    {
        (f)()
    }

    fn hello(f: Box<dyn Fn()>) {
        f();
    }

    fn one() {
        let mut z = String::new();
        let f = || {
            println!("{}", z);
        };
        
        // For Fn, all you need is the shared reference
        let f: &dyn Fn() = &f;
        quox(f);
        
        // let f: &dyn FnMut() = &f;
        // The above doesn't implement FnMut because all you have is a shared
        // reference
    }

    fn two() {
        let mut z = String::new();
        let mut f = || {
            println!("{}", z);
        };
        
        // let f: &dyn FnMut() = &f;
        // The above doesn't implement FnMut because all you have is a shared
        // reference

        // For FnMut need to call it with mut shared reference
        let f: &mut dyn FnMut() = &mut f;
        quox(f);
    }

    fn quox_three<F>(mut f: F)
    where 
        F: FnOnce() 
    {
        (f)()
    }

    fn three() {
        let mut z = String::new();
        let mut f = || {
            println!("{}", z);
        };

        // For FnOnce, you need a wide pointer type that allows you to take ownership
        // This would also work with Box<dyn FnMut> and Box<dyn Fn>
        let f: Box<dyn FnOnce()> = Box::new(f);
        quox_three(f);
    }
}

mod const_mod {
    fn one() {
        // this closure is callable at compile time
        let x = || 0;
    }

    // This is experimental, wont compile
    // The tilde means that foo will be const if F is const
    // const fn foo<F: ~const FnOnce()>(f: F) {
    //     f()
    // }
}


fn main() {
    one::one();
}


