use std::cell::UnsafeCell;

mod one {
    use super::*;

    pub struct Cell<T> {
        value: UnsafeCell<T>,
    }

    /*
    impl<T> !Sync for Cell<T> {}
    // Don't allow Cell to be shared across threads
    // Implied by UnsafeCell
    */
   
    impl<T> Cell<T> {
        fn new(value: T) -> Self {
            Cell { 
                value: UnsafeCell::new(value),
            }
        }

        pub fn set(&self, value: T) {
            // SAFETY: We know no one else is concurrently mutating self.value (because !Sync)
            // SAFETY: we know we're not invalidating any references, because we never give any out
            unsafe { *self.value.get() = value };
        }

        pub fn get(&self) -> T
            where T: Copy 
        {
            // SAFETY: We know no one else is modifying this value, this only this thread can mutate
            // (because !Sync), and is executing this function instead (of set)
            unsafe { *self.value.get() }
        } 
    }

    #[cfg(test)]
    mod test {
        use super::Cell;
        use std::sync::Arc;

        // Both tests should not compile
        // #[test]
        // fn bad() {
        //     let x = Arc::new(Cell::new(42));
        //     let x1 = Arc::clone(&x);
        //     std::thread::spawn(|| {
        //         x1.set(43);
        //     });
        //     let x2 = Arc::clone(&x);
        //     std::thread::spawn(|| {
        //         x2.set(44);
        //     });
        // }

        // // Single threaded example of why this doesnt compile
        // #[test]
        // fn bad2() {
        //     let x = Cell::new(vec![42]);
        //     let first = &x.get()[0];
        //     x.set(vec![]);
        //     eprintln!("{}", first);
        // }
    }
}

pub mod two {
    use super::*;
    pub struct Cell<T> {
        value: UnsafeCell<T>,
    }

    // purposely implent Sync trait for Cell trait so that it complies
    // but the result are wrong
    unsafe impl<T> Sync for Cell<T> {}
   
    impl<T> Cell<T> {
        pub fn new(value: T) -> Self {
            Cell { 
                value: UnsafeCell::new(value),
            }
        }

        pub fn set(&self, value: T) {
            unsafe { *self.value.get() = value };
        }

        pub fn get(&self) -> &T
        {
            unsafe { &*self.value.get() }
        } 
    }

    #[cfg(test)]
    mod test {
        use super::Cell;
        use std::sync::Arc;

        #[test]
        fn bad() {
            let x = Arc::new(Cell::new(0));
            let x1 = Arc::clone(&x);
            let jh1 = std::thread::spawn(move || {
                for _ in 0..1_000_000 {
                    let x = x1.get();
                    x1.set(x + 1);
                }
            });
            let x2 = Arc::clone(&x);
            let jh2 = std::thread::spawn(move || {
                for _ in 0..1_000_000 {
                    let x = x2.get();
                    x2.set(x + 1);
                }
            });
            jh1.join().unwrap();
            jh2.join().unwrap();

            // This should fail, as the threads are racing to increment the values
            // at the same time
            assert_eq!(*x.get(), 2_000_000)
        }

        #[test]
        fn bad2() {
            let x = Cell::new(String::from("hello"));
            let first = x.get();
            x.set(String::new());
            x.set(String::from("world"));
            // This should print "world" instead of "hello", since it has been modified
            eprintln!("{}", first);
        }
    }
}