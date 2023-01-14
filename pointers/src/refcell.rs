// Refcell
// runtime checked borrow rules, as opposed to compile time check
// safe, dynamically checked borrowing, good thing for graphs and trees
use std::cell::{UnsafeCell};

mod one {
    use super::*;
    enum RefState {
        Unshared,
        Shared(usize),
        Exclusive,
    }

    pub struct RefCell<T> {
        value: UnsafeCell<T>,
        state: RefState,
    }

    /*
    impl<T> !Sync for RefCell<T> {}
    // Don't allow RefCell to be shared across threads
    // Implied by UnsafeCell
    */

    impl<T> RefCell<T> {
        pub fn new(value: T) -> Self {
            Self {
                value: UnsafeCell::new(value),
                state: RefState::Unshared,
            }
        }

        // If you have a shared reference, there are no exclusive references
        pub fn borrow(&self) -> Option<&T> {
            match self.state {
                RefState::Unshared => {
                    // Can't mutate self when the the API takes in reference to self
                    //self.state = RefState::Shared(1);
                    Some(unsafe { &*self.value.get()})
                }
                RefState::Shared(n) => {
                    // Can't mutate self when the the API takes in reference to self
                    //self.state = RefState::Shared(n + 1);
                    Some(unsafe { &*self.value.get()})
                }
                RefState::Exclusive => None
            }
        }

        // If you have an exclusive reference, there are no shared references
        pub fn borrow_mut(&self) -> Option<&mut T> {
            if let RefState::Unshared = self.state {
                // Can't mutate self when the the API takes in reference to self
                // So use Cell instead
                // self.state = RefState::Exclusive;
                Some(unsafe { &mut *self.value.get()})
            } else {
                None
            }
        }
    }
}


// If type is not Sync and need some way to mutate sync, can use Cell
mod two {
    use super::*;
    // cell::two is because the Cell implementation is under module two
    use crate::cell::two::Cell;

    // Cell doesnt allow to use to get the reference to thing that inside
    // but that's fine, still the enum can be cheaply cloned
    #[derive(Copy, Clone)]
    enum RefState {
        Unshared,
        Shared(usize),
        Exclusive,
    }

    pub struct RefCell<T> {
        value: UnsafeCell<T>,
        state: Cell<RefState>,
    }

    /*
    impl<T> !Sync for RefCell<T> {}
    // Don't allow RefCell to be shared across threads
    // Implied by UnsafeCell
    */

    impl<T> RefCell<T> {
        pub fn new(value: T) -> Self {
            Self {
                value: UnsafeCell::new(value),
                state: Cell::new(RefState::Unshared),
            }
        }

        // If you have a shared reference, there are no exclusive references
        pub fn borrow(&self) -> Option<&T> {
            match self.state.get() {
                RefState::Unshared => {
                    self.state.set(RefState::Shared(1));
                    // SAFETY: no exclusive references have been given out since state
                    // would be Exclusive.
                    Some(unsafe { &*self.value.get()})
                }
                RefState::Shared(n) => {
                    self.state.set(RefState::Shared(n + 1));
                    // SAFETY: no exclusive references have been given out since state
                    // would be Exclusive
                    Some(unsafe { &*self.value.get()})
                }
                RefState::Exclusive => None
            }
        }

        // If you have an exclusive reference, there are no shared references
        pub fn borrow_mut(&self) -> Option<&mut T> {
            if let RefState::Unshared = self.state.get() {
                // SAFETY: no other references have been given out since state
                // would be Shared or Exclusive
                self.state.set(RefState::Exclusive);
                Some(unsafe { &mut *self.value.get()})
            } else {
                None
            }
        }
        // PROBLEM: Reference counts are never decreased! The moment you exclusively
        // borrow something, you can never borrow it again, making this a pretty useless
        // structure
    }
}


mod three {
    use super::*;
    // cell::two is because the Cell implementation is under module two
    use crate::cell::two::Cell;

    // Cell doesnt allow to use to get the reference to thing that inside
    // but that's fine, still the enum can be cheaply cloned
    #[derive(Copy, Clone)]
    enum RefState {
        Unshared,
        Shared(usize),
        Exclusive,
    }

    pub struct RefCell<T> {
        value: UnsafeCell<T>,
        state: Cell<RefState>,
    }

    // Just reference to refcell
    pub struct Ref<'refcell, T> {
        refcell: &'refcell RefCell<T>,
    }

    impl<T> Drop for Ref<'_, T> {
        fn drop(&mut self) {
            match self.refcell.state.get() {
                RefState::Exclusive | RefState::Unshared => unreachable!(),
                RefState::Shared(1) => {
                    self.refcell.state.set(RefState::Unshared);
                }
                RefState::Shared(n) => {
                    self.refcell.state.set(RefState::Shared(n - 1));
                }
            }
        }
    }
    // Deref is the trait that gets invoked evertime you used the dot operator
    // If you do T.some_method() and if T doesn't have that method, but derefs to something 
    // that does, then deref trait gets called
    // A way to automatically following deeper into a type
    impl<T> std::ops::Deref for Ref <'_, T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            // SAFETY
            // A Ref is only created if no exclusive references have been given out
            // Once it is given out, state is set to Shared so no exclusive references
            // are given out. So deferencing into a shared reference is fine
            unsafe { &*self.refcell.value.get() }
        }
    }

    pub struct RefMut<'refcell, T> {
        refcell: &'refcell RefCell<T>,
    }

    impl<T> Drop for RefMut<'_, T> {
        fn drop(&mut self) {
            match self.refcell.state.get() {
                RefState::Shared(_) | RefState::Unshared => unreachable!(),
                RefState::Exclusive => {
                    self.refcell.state.set(RefState::Unshared);
                }
            }
        }
    }

    impl<T> std::ops::Deref for RefMut<'_, T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            // SAFETY
            // See safety for DerefMut
            unsafe { &*self.refcell.value.get() }
        }
    }

    impl<T> std::ops::DerefMut for RefMut<'_, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            // SAFETY
            // A RefMut is only created if no other references have been given out
            // Once it is given out, state is set to Exclusive so no future references
            // are given out. So we have an exclusive lease on the innter value
            // so mutably deferencing is fine
            unsafe { &mut *self.refcell.value.get() }
        }
    }

    /*
    impl<T> !Sync for RefCell<T> {}
    // Don't allow RefCell to be shared across threads
    // Implied by UnsafeCell
    */

    impl<T> RefCell<T> {
        pub fn new(value: T) -> Self {
            Self {
                value: UnsafeCell::new(value),
                state: Cell::new(RefState::Unshared),
            }
        }

        // If you have a shared reference, there are no exclusive references
        pub fn borrow(&self) -> Option<Ref<'_, T>> {
            match self.state.get() {
                RefState::Unshared => {
                    self.state.set(RefState::Shared(1));
                    Some(Ref{refcell: self})
                }
                RefState::Shared(n) => {
                    self.state.set(RefState::Shared(n + 1));
                    Some(Ref{refcell: self})
                }
                RefState::Exclusive => None
            }
        }

        // If you have an exclusive reference, there are no shared references
        pub fn borrow_mut(&self) -> Option<RefMut<'_, T>> {
            if let RefState::Unshared = self.state.get() {
                // SAFETY: no other references have been given out since state
                // would be Shared or Exclusive
                self.state.set(RefState::Exclusive);
                Some(RefMut { refcell: self })
            } else {
                None
            }
        }
    }
}