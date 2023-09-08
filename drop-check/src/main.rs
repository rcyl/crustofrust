#![feature(dropck_eyepatch)] // this is permanently unstable

mod one {
    // Boks is Box in norwegian
    // For any generic T, compiler assumes that dropping the T
    // will access the T
    pub struct Boks<T> {
        p: *mut T,
    }

    impl<T> Drop for Boks<T> {
        fn drop(&mut self) {
            // Just to show that it is possible to read the inner value in the 
            // drop implementation
            // let _: u8 = unsafe { std::ptr::read(self.p as *const u8)};

            // SAFETY: p was constructed from a Box in the first place
            // and has not been freed otherwise since self still exists
            unsafe {
                // Below calls destructor of T and deallocates the box
                Box::from_raw(self.p);
                // Below just drops the pointer without deallocating the box
                // std::ptr::drop_in_place(self.p);
            }
        }
    }

    impl<T> Boks<T> {
        pub fn new(t: T) -> Self {
            Boks {
                p: Box::into_raw(Box::new(t)),
            }
        }
    }

    impl<T> std::ops::Deref for Boks<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            // SAFETY: This is valid since Boks is constructed from a valid T and
            // was turned into a pointer through Box which creates aligned pointers
            // and hasn't been freed since self is alive
            unsafe {
                &*self.p 
            }
        }
    }

    impl<T> std::ops::DerefMut for Boks<T> {
        // Since DerefMut is a subtrait of Deref, so anything that is DerefMut
        // is also Deref, the compiler understands that the Self::Target is the 
        // associated type of Deref, which is the parent trait of DerefMut
        fn deref_mut(&mut self) -> &mut Self::Target {
            // SAFETY: This is valid since Boks is constructed from a valid T and
            // was turned into a pointer through Box which creates aligned pointers
            // and hasn't been freed since self is alive
            // Also, since we have &mut self, no other mutable reference has 
            // been given out to p 
            unsafe {
                &mut *self.p 
            }
        }
    }

    pub fn test() {
        let x = 42;
        let b = Boks::new(x);
        println!("{}", *b); // prints 42

        let mut y = 42;
        // Below does not compile because the compiler is trying to guard us from 
        // a drop implementation that might use the value
        let b = Boks::new(&mut y);
        //println!("{:?}", y);
    }
}

mod two {
    pub struct Boks<T> {
        p: *mut T,
    }

    // This notation tells the complier that we promise that the code
    // inside the drop does not access/use the T
    unsafe impl<#[may_dangle] T> Drop for Boks<T> {
        fn drop(&mut self) {
            unsafe {
                Box::from_raw(self.p);
            }
        }
    }

    impl<T> Boks<T> {
        pub fn new(t: T) -> Self {
            Boks {
                p: Box::into_raw(Box::new(t)),
            }
        }
    }

    impl<T> std::ops::Deref for Boks<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            unsafe {
                &*self.p 
            }
        }
    }

    impl<T> std::ops::DerefMut for Boks<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe {
                &mut *self.p 
            }
        }
    }

    use std::fmt::Debug;
    // Oissan means oopsies
    struct Oisann<T: Debug>(T);

    impl<T: Debug> Drop for Oisann<T> {
        fn drop(&mut self) {
            print!("{:?}", self.0);
        }
    }

    pub fn test() {
        let x = 42;
        let b = Boks::new(x);
        println!("{}", *b); // prints 42

        let mut y = 42;
        // This does compile after the no_mangle in T for drop trait
        let b = Boks::new(&mut y);
        println!("{:?}", y);

        let mut z = 42;
        // This compiles but really should not
        let b = Boks::new(Oisann(&mut z));
        println!("{:?}", z);
        // when b is dropped, it will access the mut reference
        // drop(b)
        //      *&mut z

        // with a standard Box, this does not compile
        // let b = Box::new(Oisann(&mut z));
        // println!("{:?}", z);
    }


}

mod three {
    use std::marker::PhantomData;

    pub struct Boks<T> {
        p: *mut T, // Boks is invariant because of *mut
        _t: PhantomData<T>
    }

    unsafe impl<#[may_dangle] T> Drop for Boks<T> {
        fn drop(&mut self) {
            unsafe {
                Box::from_raw(self.p);
            }
        }
    }

    impl<T> Boks<T> {
        pub fn new(t: T) -> Self {
            Boks {
                p: Box::into_raw(Box::new(t)),
                _t: PhantomData,
            }
        }
    }

    impl<T> std::ops::Deref for Boks<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            unsafe {
                &*self.p 
            }
        }
    }

    impl<T> std::ops::DerefMut for Boks<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe {
                &mut *self.p 
            }
        }
    }

    use std::fmt::Debug;
    // Oissan means oopsies
    struct Oisann<T: Debug>(T);

    impl<T: Debug> Drop for Oisann<T> {
        fn drop(&mut self) {
            print!("{:?}", self.0);
        }
    }

    pub fn test() {
        let mut z = 42;
        // With phantom data, this DOES NOT compile
        // let b = Boks::new(Oisann(&mut z));
        // println!("{:?}", z);

        // Below DOES not compile
        // s will live until the end of the function
        // which is shorter than 'static
        // So in theory, we should be able to 
        // assign boks1 to something that takes in 'static
        // but this is not the case right now. 

        // let s = String::from("hei");
        // let mut boks1 = Boks::new(&*s);
        // let boks2: Boks<&'static str> = Boks::new("heisann");
        // boks1 = boks2;
        
        // Above wil compile if we used std lib's Box
        // because it is covariant in the type T
        // and Boks is not 
    }


}


mod four {
    use std::marker::PhantomData;
    use std::ptr::NonNull; // this is kinda like a *mut, except that it is covariant

    pub struct Boks<T> {
        p: NonNull<T>,
        _t: PhantomData<T>
        
    }

    unsafe impl<#[may_dangle] T> Drop for Boks<T> {
        fn drop(&mut self) {
            unsafe {
                Box::from_raw(self.p.as_mut());
            }
        }
    }

    impl<T> Boks<T> {
        pub fn new(t: T) -> Self {
            Boks {
                // Safety: Box never creates a null pointer
                p: unsafe { 
                    NonNull::new_unchecked(Box::into_raw(Box::new(t)))
                },
                _t: PhantomData,
            }
        }
    }

    impl<T> std::ops::Deref for Boks<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            unsafe {
                &*self.p.as_ref() 
            }
        }
    }

    impl<T> std::ops::DerefMut for Boks<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe {
                &mut *self.p.as_mut() 
            }
        }
    }

    use std::fmt::Debug;
    // Oissan means oopsies
    struct Oisann<T: Debug>(T);

    impl<T: Debug> Drop for Oisann<T> {
        fn drop(&mut self) {
            print!("{:?}", self.0);
        }
    }
    struct Deserializer<T> {
        // The below notation is used for types that need to be covariant
        // over T but they don't drop a T because they don't contain a T
        // for example a Deserializer or Empty Iterator
        _t : PhantomData<fn() -> T>
        // we don't do _t: PhantomData<fn(T)> because this is contravariant
    }

    // If the inner type is PhantomData<T>, this would not compile
    // because Oisann accesses it own inner &mut i32 during the drop 
    // but the serializer's inner T is Oisann so by right it shouldn't care
    // about Oisann's inner type
    // Deserializer<Oisann<&mut i32>>

    struct EmptyIterator<T> {
        _t: PhantomData<fn() -> T>,
    }

    impl<T> Iterator for EmptyIterator<T> {
        type Item = T;
        fn next(&mut self) -> Option<Self::Item> {
            // we are returning None for next, so we don't want to drop our T
            None
        }
    }

    pub fn test() {
        // Below now compiles, because Boks is now covariant
        let s = String::from("hei");
        let mut boks1 = Boks::new(&*s);
        let boks2: Boks<&'static str> = Boks::new("heisann");
        boks1 = boks2;

        // let mut z = 42;
        // This compiles but really should not
        // let b = Box::new(Oisann(&mut z));
        // println!("{:?}", z);
    }

}


fn main() {
    // one::test();
    // two::test();
}


