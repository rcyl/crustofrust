// Think of Send as sending the data to another thread and 
// the other thread can do whatever it wants with it, like a primitive type like
// bool or i32

// A type T is Sync, if and only if &T (reference to T) is Send

// We cannot give out a MutexGuard (to another thread), but we can give out 
// reference to a MutexGuard, that's why it is not Send (!Send) but is Sync

// Cell is Send but not Sync
// Cell does not give out references to the inner type (check the API)

struct Rc<T> {
    inner: *mut Inner<T>,
}

struct Inner<T> {
    count: usize,
    value: T,
}

impl<T> Rc<T> {
    pub fn new(v: T) -> Self {
        Rc {
            inner: Box::into_raw(Box::new(Inner {count: 1, value: v})),
        }
    }
}

// NOTE: T does not need to implement Clone
impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        unsafe { &mut *self.inner}.count += 1;
        Rc { inner: self.inner }
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        let cnt = &mut unsafe { &mut *self.inner }.count;
        if *cnt == 1 {
            // Makes the box own the inner pointer and 
            // immediatedly go out of scope, effectively
            // dropping the pointer
            let _ = unsafe { Box::from_raw(self.inner) };
        } else {
            *cnt -= 1;
        }
    }
}

impl<T> std::ops::Deref for Rc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &unsafe { &*self.inner }.value
    }
}

fn caller() {
    let x = Rc::new(1);
    let y = x.clone();
    // Below DOES not compile, because our Rc is not Send
    // both threads will be racing to drop the inner value
    // std::thread::spawn(move || {
    //     drop(y);
    // });
    drop(x);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
