// Rc does not provide mutability
// It just allows multiple shared references to a thing
// Rc is not thread safe!
use crate::cell::two::Cell;
use std::ptr::NonNull;
use std::marker::PhantomData;

struct RcInner<T> {
    value: T,
    refcount: Cell<usize>,
}

pub struct Rc<T> {
    // Rc is !Send because NonNull is !Send
    inner: NonNull<RcInner<T>>,
    // This doesn't work because closing the struct clones the refcount as well 
    //refcount: usize
    _marker: PhantomData<RcInner<T>>,
}

impl<T> Rc<T> {
    pub fn new(v: T) -> Self {
        // Box gives us a pointer on the heap
        let inner = Box::new(RcInner {
            value: v,
            refcount: Cell::new(1)
        });
        Rc {
            // Box::into_raw returns the raw pointer
            // If we didnt do it, then inner would go out
            // of scope and box will be deallocated

            // SAFETY: Box does not give us a null pointer
            inner: unsafe { NonNull::new_unchecked(Box::into_raw(inner)) }, 
            _marker: PhantomData,
        }
    }
}

// T doesnt have to implement Clone
// because we are not copying the inner value
impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        let inner = unsafe { self.inner.as_ref() };
        let c = inner.refcount.get();
        inner.refcount.set(c + 1); 
        Rc { 
            inner: self.inner,
            _marker: PhantomData,
        }
    }
}

impl<T> std::ops::Deref for Rc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY self.inner is a Box that is only deallocated when the last Rc 
        // goes away. We have an Rc therefore the Box has not been deallocated,
        // so deref is fine
        &unsafe { &*self.inner.as_ref() }.value
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.inner.as_ref() };
        let c = inner.refcount.get();
        if *c == 1 {
            //SAFETY: we are the only Rc left and we are being dropped
            // therefore after us, there will be no Rc's and no references to T
            drop(inner);
            let _ = unsafe {
                Box::from_raw(self.inner.as_ptr())
            };
        } else {
            // there are other Rcs so don't drop the Box!
            inner.refcount.set(c -1);
        }
    }
}

// Check video at 1:30+ for PhantomData and DropCheck
// !Sized means not Sized