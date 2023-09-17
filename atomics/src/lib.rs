
/*******************************************************************************

Atomics

load => get the value
store => set the value

Compare_and_swap is a single operation, whereas load and store are 2 different
operations and another thread can come in between both operations

fetch_add => loads a valud and adds to it
fetch_* => the rest of it, fairly self explanatory

*/

use std::sync::atomic::{AtomicBool, Ordering};
use std::cell::UnsafeCell;

const LOCKED: bool = true;
const UNLOCKED: bool = false;

mod one {
    use super::*;

    pub struct Mutex<T> {
        locked: AtomicBool,
        v: UnsafeCell<T>,
    }

    // we have to implement this ourselves since UnsafeCell is not Sync
    unsafe impl<T> Sync for Mutex<T> where T: Send {}

    impl<T> Mutex<T> {
        pub fn new(t: T) -> Self {
            Self {
                locked: AtomicBool::new(UNLOCKED),
                v: UnsafeCell::new(t),
            }
        }
        
        // spinlock impl, dont use spinlocks in general! implementing just for exercise
        pub fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
            while self.locked.load(Ordering::Relaxed) != UNLOCKED {}
            // in between the load and store here, another thread may run
            // std::thread::yield_now(); simulate thread getting pre-empted by OS
            self.locked.store(LOCKED, Ordering::Relaxed);

            // Safety: we hold the lock, therefore we can create a mutable reference
            let ret = f(unsafe { &mut *self.v.get() });
            self.locked.store(UNLOCKED, Ordering::Relaxed);
            ret
        }
    }
}

mod two {
    use super::*;

    pub struct Mutex<T> {
        locked: AtomicBool,
        v: UnsafeCell<T>,
    }

    unsafe impl<T> Sync for Mutex<T> where T: Send {}

    impl<T> Mutex<T> {
        pub fn new(t: T) -> Self {
            Self {
                locked: AtomicBool::new(UNLOCKED),
                v: UnsafeCell::new(t),
            }
        }

        /*  
        In general you want to use compare_exchange instead of compare_and_swap
        because comapare_exchange is more powerful as it allows you to specifiy
        the memory ordering differently depending if the operation succeeded or not

        compare_exchange() returns Error if the value is not updated and OK if it is
        compare_exchange() is fairly expensive, look at MESI protocol
        compare_exchange() will also never wait unlike a mutex, it just goes ahead and does it
        It also often used in a loop.
        compare_exchange() is only allowed to fail if the current value 
        does not match the value that is passed in, and not allowed to fail in any other condition
        
        compare_exchange_weak() is allowed to fail spuriously, even if the value you passed
        in matches the current value (usually wont, but allowed to)

        x86:    CAS (Compare and swap, kinda like compare_exchange)
                - compare_exchange_weak() is implemented as the same as compare_exchange() in x86
        ARM:    LDREX (Load exclusive) 
                    -> takes exclusive ownership of location in memory and loads the value
                STREX (Store exclusive) 
                    ->  only if i still have exclusive access to that location in memory
                        and no one has taken that from me, and only then i'll store
                        (if someone took exclusive access via LDREX just to read it, i'll still fail)
                    STREX is fairly cheap
                -   compare_exchange(): is implemented using a loop of LDREX and STREX in ARM
                    and therefore tend to be a nested loop and less generate less efficient code
                    because of registry pressure
                -   compare_exchange_weak(): LDREX STREX 
                    so if you already call it in a loop, you should use compare_exchange_weak()
        */
        
        pub fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
            while self.
                locked.compare_exchange_weak(
                    UNLOCKED, // what the current value should be in order for us to update it
                    LOCKED, // what is should be set to, the the first value matches the argument
                    Ordering::Relaxed, 
                    Ordering::Relaxed
                ).is_err()  // if it fails the locking, we loop and try again
            {
                // if we fail to take the lock, then we're just going to spin and just read the value
                // which allows the value to stay in the shared state, so we don't have 
                // ownership bouncing between CPUs
                // The moment the state changes because some core takes exclusive access to the value
                // only then we go back and do the expensive compare_exchange op
               
                // MESI protocol: stay in S when locked
                while self.locked.load(Ordering::Relaxed) == LOCKED {
                    // yield can be used to trigger race condition, letting othreads in here
                    // in order to show ordering::relaxed's problem
                    std::thread::yield_now();
                }
                std::thread::yield_now();
            } 
            // This still wouldn't work, though tests could be passing
            let ret = f(unsafe { &mut *self.v.get() });
            self.locked.store(UNLOCKED, Ordering::Relaxed);
            // The above could also be reordered, which is not fine!
            // self.locked.store(UNLOCKED, Ordering::Relaxed);
            // let ret = f(unsafe { &mut *self.v.get() });
            ret
        }
    }
}

mod three {
    use super::*;

    pub struct Mutex<T> {
        locked: AtomicBool,
        v: UnsafeCell<T>,
    }

    unsafe impl<T> Sync for Mutex<T> where T: Send {}

    impl<T> Mutex<T> {
        pub fn new(t: T) -> Self {
            Self {
                locked: AtomicBool::new(UNLOCKED),
                v: UnsafeCell::new(t),
            }
        }

        pub fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {

            // With Ordering::Acquire, no operation can be reodered before the load
            // ie, `let ret = f(unsafe { &mut *self.v.get() })`, cannot be moved before the  
            // load

            // There is also AcqRel (Acquire/Release) means do the load with Acquire
            // and the store with Release
            // If this case, we don't need AcqRel in our compare_exchange_weak 
            // since the Release is done at the end of our loop
            // AcqRel is more commonly used for a fetch_add or a single modication operation
            // where there is no critical section
            while self.
                locked.compare_exchange_weak( // LDREX STREX 
                    UNLOCKED, // what the current value should be in order for us to update it
                    LOCKED, // what it should be set to, the the first value matches the argument
                    Ordering::Acquire, 
                    Ordering::Relaxed 
                    // The argument above indicates what ordering should the load 
                    // have if the load indicates that the load shouln't store.
                    // In this case, it can be thought off as what is the ordering 
                    // of failing to take the lock
                ).is_err()  // if it fails the locking, we loop and try again
            {
                // if we fail to take the lock, then we're just going to spin and just read the value
                // which allows the value to stay in the shared state, so we don't have 
                // ownership bouncing between CPUs
                // The moment the state changes because some core takes exclusive access to the value
                // only then we go back and do the expensive compare_exchange op
               
                // MESI protocol: stay in S when locked

                // This load can still stay relaxed, because we don't need to 
                // establish a "happens before" relationship since we haven't 
                // taken the lock
                // in general can use Ordering::Relaxed if 
                // when it doesn't matter what each thread sees
                while self.locked.load(Ordering::Relaxed) == LOCKED {
                    std::thread::yield_now();
                }
                std::thread::yield_now();
            } 
            let ret = f(unsafe { &mut *self.v.get() });
            self.locked.store(UNLOCKED, Ordering::Release);

            // When we do the store with Ordering::Release, any load with the same value
            // that uses Ordering::Acquire or stronger must see all operations that happen
            // before the store as happening before the store

            // Also, no reads/write can be reordered after this store with Ordering::Release

            // If we use Ordering::Relaxed, the modification made to memory with
            // `let ret = f(unsafe { &mut *self.v.get() })`, may not be visible to
            // the next thread that takes the lock

            ret
        }
    }

}


 

// The fetch operations does not FAIL
// fetch_* operations returns the previous value

// Can think of fetch_add as a fetch_update that takes in a closure that adds 1
// Fetch_update can be thought of a compare_exchange loop that is implemented for you (can check source code)
// Can be used to give unique sequence numbers to a bunch of things that happens
// concurrently

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::spawn;

    #[test]
    fn one_does_not_work() {
        let l: &'static _ = Box::leak(Box::new(one::Mutex::new(0)));
        let handles: Vec<_> = (0..10)
            .map(|_| {
                spawn(move || {
                    for _ in 0..100 {
                        l.with_lock(|v| {
                            *v += 1;
                        })
                    }   
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
        assert_eq!(l.with_lock(|v| *v), 10 * 100);
    }

    #[test]
    fn too_relaxed() {
        use std::sync::atomic::AtomicUsize;
        let x: &'static _ = Box::leak(Box::new(AtomicUsize::new(0)));
        let y: &'static _ = Box::leak(Box::new(AtomicUsize::new(0)));
        let t1 = spawn(move || {
            let r1 = y.load(Ordering::Relaxed);
            x.store(r1, Ordering::Relaxed);
            r1
        });
        let t2 = spawn(move || {
            let r2 = x.load(Ordering::Relaxed);
            y.store(42, Ordering::Relaxed);
            r2
            /*
                It is possible for R2 to see the value 42! 
                Which is surpising, if the instructions are executed in order
                
                1. in t2, r2 is loaded with x which is 0
                2. in t2, y is stored with 42
                3. let's say t1 gets to run now, so r1 is loaded with the value of y which is 42
                4. in t1, x is stored the value of 42
                5. in t1, r1 is returned which 42 
                6. back in t2, r2 is returned which is 0   

                So by right r2 cannot be 42 ! even when executed in multiple threads

                The way this can happen is that the instructions are swapped by the CPU, ie
                    
                    y.store(42, Ordering::Relaxed)
                    let r2 = x.load(Ordering::Relaxed);
                
                Another mental mode for understanding this:
                
                MO -> Modification Order

                MO(x): 0 42
                MO(y): 0 42 
                
                When a value is loaded with Ordering::Relaxed, you can see any value
                written by any thread to that locaton.

                So the load of x is allowed to see any value ever stored to x 
                (even values that's only written in the future like 42)

             */
        });

        let r1 = t1.join().unwrap();
        let r2 = t2.join().unwrap();
        

    }

    #[test]
    fn test_seq_cst() {
        use std::sync::atomic::AtomicUsize;
        let x: &'static _ = Box::leak(Box::new(AtomicBool::new(false)));
        let y: &'static _ = Box::leak(Box::new(AtomicBool::new(false)));
        let z: &'static _ = Box::leak(Box::new(AtomicUsize::new(0)));
        
       let _tx = spawn(move || {
            x.store(true, Ordering::Release);
        });

        let _ty = spawn(move || {
            y.store(true, Ordering::Release);
        });

        let t1 = spawn(move || {
            // You should see all operations before the corresponding 
            // Ordering::Release store() in tx. There are no operations before that store
            while !x.load(Ordering::Acquire) {}
            // So the thread is allowed to see all values of y (true, false), even if ty has not ran
            if y.load(Ordering::Acquire) {
                z.fetch_add(1, Ordering::Relaxed);    
            }
            // so t1 must "see" that x is true because of the spin
            // but after that, it can see y as both true and false
            // It is also not the case where the operations is reordered as Acquire prevents that
        });

        let t2 = spawn(move || {
            // You should see all operations before the corresponding 
            // Ordering::Release store() in ty. There are no operations before that store
            while !y.load(Ordering::Acquire) {}
            // So the thread is allowed to see all values of x (true, false), even if tx has not ran
            if x.load(Ordering::Acquire) {
                z.fetch_add(1, Ordering::Relaxed);    
            }
            // so t2 must "see" that y is true because of the spin
            // but after that, it can see x as both true and false
        });

        t1.join().unwrap();
        t2.join().unwrap();

        let z = z.load(Ordering::SeqCst);
        /*
        What are the possible values for z?

        Is 0 possible?
            Restrictions: 
                - t1 must run "after" tx, because t1 is gonna spin in a loop until tx runs
                - t2 must run "after" ty, for the same reason as above
            Given that..
                .. tx .. t1 .. (t1 must run after tx)
                .. ty .. t2 .. (t2 must run after ty)

                ty t2 tx t1 -> t1 will increment z
                
                ty tx t2 t1 -> in this case t1 & t2 will increment z
                tx ty t2 t1 -> in this case t1 & t2 will increment z

                tx t1 ty t2 -> t2 will increment z
            
        So 0 seems impossible.., but is really not
        thread schedules are just human constructs to help us think about it!

        If all the Ordering for both store and load for x and y are changed 
        to Ordering::SeqCst, 0 is no longer possible

        MO -> Modification Order

        MO(x): false true
        MO(y): false true

        ---

        Is 1 possible?
            Yes: tx, t1, ty, t2

        ---
        Is 2 possible?
            Yes: tx, ty, t1, t2 (the ordering of the thread execution )
        */

    }


    /*
        Can use ThreadSanitizer to check for atomic bugs, even in Rust. 

        Use Loom for testing. in testing use Loom structs and in production use
        std lib structs. 

        std::sync::atomic::compiler_fence is only for the compiler, but the CPU
        can still execute out of order, so rarely used

        std::sync::atomic::fence is important, go read the docs. 

        std::ptr::read_volatile() tells the compiler that it must read from memory
        and that the operations cannot be reordered

        AtomicPtr is a specialized version of AtomicUsize that operates on pointers. 
    
     */

}

