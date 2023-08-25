use std::sync::{Arc, Mutex, Condvar};
use std::collections::VecDeque;

mod one {
    use super::*;

    struct Inner<T> {
        queue: Mutex<VecDeque<T>>,
        available: Condvar,
    }
    pub struct Sender<T> {
        inner: Arc<Inner<T>>,
    }

    // NOTE: notice the signature is not T: Clone, because our inner type
    // is already an Arc, which is cloneable. we could use #[derive(Clone)] but this imlements
    // makes T: Clone, which we are trying to avoid
    impl<T> Clone for Sender<T> {
        fn clone(&self) -> Self {
            Sender {
                // The compiler deferences the inner type and it doesn't know
                // if you are trying to clone the Arc or the inner type so use
                // Arc::clone instead
                //inner: self.inner.clone();
                inner: Arc::clone(&self.inner),
            }
        }
    }

    pub struct Receiver<T> {
        inner: Arc<Inner<T>>,

    }

    impl<T> Sender<T> {
        pub fn send(&mut self, t: T) {
            let mut queue = self.inner.queue.lock().unwrap();
            queue.push_back(t);
            // need to drop the lock so whoever we notify can wake up and grab it
            // though the lock will be dropped at the end, we want to other thread
            // to be able to get it immediatedly
            drop(queue); 
            self.inner.available.notify_one();
        }
    }

    impl<T> Receiver<T> {
        pub fn recv(&mut self) -> T {
            let mut queue = self.inner.queue.lock().unwrap();
            loop { // NOTE: this is not a "spin" loop because of the wait on condvar
                match queue.pop_front() {
                    Some(t) => return t,
                    None => {
                        // NOTE: need to take the guard while waiting. wait gives up the lock before it goes to sleep
                        // We'll get the lock back upon waking up 
                        queue = self.inner.available.wait(queue).unwrap();
                    }
                }   
            }
        }
    }

    pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let inner = Inner {
            // NOTE:: below uses the default implementation of the inner type
            queue: Mutex::default(),
            available: Condvar::new(),
        };
        let inner = Arc::new(inner);
        (
            Sender {
                inner: inner.clone(),
            },
            Receiver {
                inner: inner.clone(),
            },
        )
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn ping_pong() {
            let (mut tx, mut rx) = channel();
            tx.send(42);
            assert_eq!(rx.recv(), 42);
        }

        #[test]
        fn closed() {
            let (tx, mut rx) = channel::<()>();
            // NOTE, below does not drop the tx immediatedly, do drop(tx) like in mod two
            let _ = tx;
            // Below hangs forever, because the sender has been dropped 
            // and there are no future senders that can be created
            // from cloning since they are already dropped
            //let _ = rx.recv();
        }
    }
}

mod two {
    use super::*;

    struct Inner<T> {
        queue: VecDeque<T>,
        senders: usize,
    }

    struct Shared<T> {
        shared: Mutex<Inner<T>>,
        available: Condvar,
    }
    pub struct Sender<T> {
        shared: Arc<Shared<T>>,
    }

    impl<T> Clone for Sender<T> {
        fn clone(&self) -> Self {
            let mut inner = self.shared.shared.lock().unwrap();
            inner.senders += 1;
            drop(inner); 
            Sender {
                shared: Arc::clone(&self.shared),
            }
        }
    }

    impl<T> Drop for Sender<T> {
        fn drop(&mut self) {
            let mut inner = self.shared.shared.lock().unwrap();
            inner.senders -= 1;
            // check if we are the last sender by checking number of senders
            let was_last = inner.senders == 0;
            drop(inner);
            if was_last {
                self.shared.available.notify_one();
            }
            
        }
    }

    pub struct Receiver<T> {
        shared: Arc<Shared<T>>,

    }

    impl<T> Sender<T> {
        pub fn send(&mut self, t: T) {
            let mut inner = self.shared.shared.lock().unwrap();
            inner.queue.push_back(t);
            // need to drop the lock so whoever we notify can wake up and grab it
            // though the lock will be dropped at the end, we want to other thread
            // to be able to get it immediatedly
            drop(inner); 
            self.shared.available.notify_one();
        }
    }

    impl<T> Receiver<T> {
        pub fn recv(&mut self) -> Option<T> {
            let mut inner = self.shared.shared.lock().unwrap();
            loop { 
                match inner.queue.pop_front() {
                    Some(t) => return Some(t),
                    None if inner.senders == 0 => return None,
                    None => {
                        inner = self.shared.available.wait(inner).unwrap();
                    }
                }   
            }
        }
    }

    pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let inner = Inner {
            queue: VecDeque::default(),
            senders: 1,
        };
        let shared = Shared {
            shared: Mutex::new(inner),
            available: Condvar::new(),
        };
        let shared = Arc::new(shared);
        (
            Sender {
                shared: shared.clone(),
            },
            Receiver {
                shared: shared.clone(),
            },
        )
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn ping_pong() {
            let (mut tx, mut rx) = channel();
            tx.send(42);
            assert_eq!(rx.recv(), Some(42));
        }

        #[test]
        fn closed_tx() {
            let (tx, mut rx) = channel::<()>();
            drop(tx);
            // Doesn't hang anymore after tracking number of senders
            // so it should return something that we can assert
            // let _ = rx.recv();
            assert_eq!(rx.recv(), None);
        }

        #[test]
        fn closed_rx() {
            let (mut tx, mut rx) = channel();
            drop(rx);
            // NOTE, not exactly wrong here but you would want some kind of
            // notification that the rx is closed instead of blindly sending
            tx.send(42);
        }
    }
}

// In "async" std::sync::mpsc which is Sender, if the sender if sending too fast,
// and the receiver can't keep up, the internal buffer will just expand inifinitely.
// with SyncSender, the channel is bounded and so if the buffer is full, the 
// SyncSender will block until the Receiver can read to free up space in buffer

// Because there is only only one receiver, we can "steal" all the items
// in the queue instead of just one and store it in our "cache" when we acquire the lock.
// Then, if our cache still has the item we don't have to take the lock.
// This minimizes lock contention
mod three {
    use super::*;

    struct Inner<T> {
        queue: VecDeque<T>,
        senders: usize,
    }

    struct Shared<T> {
        shared: Mutex<Inner<T>>,
        available: Condvar,
    }
    pub struct Receiver<T> {
        shared: Arc<Shared<T>>,
        stolen_buffer: VecDeque<T>,
    }

    impl<T> Receiver<T> {
        pub fn recv(&mut self) -> Option<T> {
            // If we already have it, dont' need to take lock
            if let Some(t) = self.stolen_buffer.pop_front() {
                return Some(t);
            }

            let mut inner = self.shared.shared.lock().unwrap();
            loop { 
                match inner.queue.pop_front() {
                    Some(t) => {
                        if inner.queue.is_empty() {
                            // there are still items in the inner queue, so
                            // swap the contents of inner queue into our stolen buffer
                            std::mem::swap(&mut self.stolen_buffer, &mut inner.queue);
                        }
                        return Some(t);
                    }
                    None if inner.senders == 0 => return None,
                    None => {
                        inner = self.shared.available.wait(inner).unwrap();
                    }
                }   
            }
        }
    }

}

// Flavours:
// - Synchronous channels: Channel where send() can block. Limited capacity
//     - Mutex + Condvar + VecDeque OR
//     - Atomic VecDeque (atomic queue) + thread::park + thread::Thread::notify 
//                                        (for signalling, instead of condvar)
// - Asynchronous channels: Channel where send() cannot block. Unbounded.
//      - Mutex + Condvar + VecDeque
//      - Mutex + Condvar + LinkedList
//      - Atomic linked list, linked list of T
//      - Atomic blocked linked list, linked list of atomic VecDeque<T> (in crossbeam crate)
// - Rendezvous channels: Synchronous with capacity = 0. Used for thread synchronization
// - Oneshot channels: Any capacity. In practice, only one call to send(). For example, 
//                     channel that use to tell threads to exit early on ctrl c

// async/await

mod four {
    use super::*;

    struct Inner<T> {
        queue: VecDeque<T>,
        senders: usize,
    }

    struct Shared<T> {
        shared: Mutex<Inner<T>>,
        available: Condvar,
    }
    pub struct Receiver<T> {
        shared: Arc<Shared<T>>,
        stolen_buffer: VecDeque<T>,
    }

    impl<T> Receiver<T> {
        pub fn recv(&mut self) -> Option<T> {
            // If we already have it, dont' need to take lock
            if let Some(t) = self.stolen_buffer.pop_front() {
                return Some(t);
            }

            let mut inner = self.shared.shared.lock().unwrap();
            loop { 
                match inner.queue.pop_front() {
                    Some(t) => {
                        if inner.queue.is_empty() {
                            // there are still items in the inner queue, so
                            // swap the contents of inner queue into our stolen buffer
                            std::mem::swap(&mut self.stolen_buffer, &mut inner.queue);
                        }
                        return Some(t);
                    }
                    None if inner.senders == 0 => return None,
                    None => {
                        inner = self.shared.available.wait(inner).unwrap();
                    }
                }   
            }
        }
    }

}


#[cfg(test)]
mod tests {
    use super::*;

}
