#[allow(dead_code, unused_variables)]

use std::future::Future;

fn main() {
    println!("Hello, world!");
    let x = foo();

    /*
        // both are async functions returning futures
        let mut network = read_from_network();
        let mut terminal = read_from_terminal();
        let mut foo = foo2();

        loop {
            select! {
                stream <- (&mut network).await => {
                    // do something on stream
                }
                line <- (&mut terminal).await => {
                    // do something with line
                }
                foo <- (&mut foo2).await => {

                }
            }
        }
     */

    /*
        let mut f1 = tokio::fs::File::open("reader");
        let mut f2 = tokio::fs::File::create("writer");
        
        // Below copies bytes from reader to writer
        let copy = tokio::io::copy(&mut f1, &mut f2);

        select !{ 
            stream <- (&mut network).await => {
                    // do something on stream
            }
            line <- (&mut terminal).await => {
                    // do something with line
            }
            _ <- copy.await => {
                // NOTE: this operations can yield halfway
                // for example if the disk is unable to write anymore
                // and therefore the operation cannot complete
                // This is an error condition that needs to be handled!
            }
        }
        // after the select
        // possible for some bytes to have been copied from reader to writer but not all 

     */

    /*
        fuse

        select will still go through all the "branches", although a future has 
        completed, because it doesn't remember.
        So this particular future needs to be safe to be polled again although
        it has already yielded the future => fuse!

        loop {
            select !{ 
                stream <- (&mut network).await => {
                    // do something on stream
                }
                line <- (&mut terminal).await => {
                    // do something with line
                }
            }
        }
     */

    // Overhead of async io read vs io read?
    // Cost of system calls for async are amortized
    // usually they end up faster because less threads are spawned

    /*
        let files: Vec<_> = (0..3).map(|i| tokio::fs::read_to_string(format!("file{}", i)))
                                    .collect();
        
        // files can be read concurrently versus sequentially awaiting
        // NOTE that in practice probably can only use for a few items, because
        // the API forces you to write them all out, which is not feasible
        // if you have 100 of them
        let (f1, f2, f3) = join!(files[0], files[1], files[2]);

        however, there is a try_join_all and you can iterators into it, 
        even if it finishes reading files[2] first it will make sure that 
        the items are in the same order ie f1 == files[0]. 
        This reordering is not free and can use
        FuturesUnordered to opt out of it
    
     */

    /*
        let runtime = tokio::runtime::Runtime::new();
        runtime.block_on(async {
            let mut accept = tokio::new::TcpListener::bind("0.0.0.0.8080");
            let mut connections = futures::future::FuturesUnordered::new();
            loop {
                select! {
                    stream <- (&mut accept).await => {
                        connections.push(handle_connection(stream));
                    }
                    // The branch is here because something needs to await on all the futures stored in connections
                    // NOTE: this is just one one top level future served by one thread! Othter threads can't help out
                    // Solution below, using tokio spawn
                    _ <- (&mut connections).await => {}
                }
            }
        });
        
        let runtime = tokio::runtime::Runtime::new();
        runtime.block_on(async {
            let mut accept = tokio::new::TcpListener::bind("0.0.0.0.8080");
            while let Ok(stream) = accept.await {
                // spawn is a hook into the executor, used to pass future to runtime
                // so runtime has two futures, the original one in block_on()
                // and the one for returned from handle_connection and now 
                // multiple threads can work on them
                // Not to be confused with thread::spawn
                // The future passed to spawn needs to be Send so that it can be passed
                // to another thread to be worked on
                tokio::spawn(handle_connection(stream))
            }
        });

        // tokio uses thread locals to make interface nicer, otherwise a handle
        // to runtime will be have to be passed throughout the application
        // ie can't just call tokio::spawn, but runtime.spawn(..)
        // the downside is can't work in environments where there are no thread locals
        // like embedded
     */
}

async fn foo() -> usize {
    let mut x  = [0; 1024];
    let n: usize = tokio::fs::read_into("file.data", &mut x[..]).await;
    println!("{:?}", x[..n]);
    0

    /*
    // can be thought of as different chunks
    // chunk 1
    {
        let mut x  = [0; 1024];
        let fut = tokio::fs::read_into("file.data", &mut x[..])
    }

    fut.await 
    yield; // actually: return

    // chunk 2
    {
        let n = fut.output();
        println!("{:?}", x[..n]);
    }

    where is x stored?

    // compiler generates state machines that has the local context
    // think of it as a union with the largest chunk state
    enum StateMachine {
        Chunk1 { x: [u8; 1024], fut: tokio::fs::ReadIntoFuture<'x> },
        Chunk2 { // don't really have any state }
    }

    fn foo() -> impl Future<Output = ()> /* return value is StateMachine! */

    */

    /*
        NOTE: revisit again if context becomes clearer on async traits

        async_trait changes the return type to Pin<Box<dyn Future>
        Heap allocating all futures! so get dynamic dispatch and
        no monomorphization 

        Cons: 
        1. Alot of memory allocation pressure
        2. Alot of indirection for all futures
            (if the async trait is read, 
                need to do memory allocation and pointer indrection for every read)

        struct Request;
        struct Response;
        
        trait Service {
            fn read(&mut self, _: Request) -> Pin<Box<dyn Future<Output = Response>>;
        }
        struct X:
        impl Service for X {
            fn call(&mut self, _: Request) -> Pin<Box<dyn Future<Output = Response>> {
                Box::pin(async move { Response })
            }
        }
     */

    /*
        use std::future::Future;
        use std::sync::{Arc, Mutex};
        use tokio::sync::Mutex;

        async fn main() {
            let x = Arc::new(Mutex::new(0));
            let x1 = Arc::clone(&x);
            tokio::spawn(async move {
                loop {
                    let x = x1.lock();
                    tokio::fs::read_to_string("file").await;
                    *x1 += 1
                    1. This runs first, it gets lock
                    2. Runs to read to string file but can't make progress
                       so it yields
                    3. Tries to run the next future
                }
            });

            let x2 = Arc::clone(&x);
            tokio::spawn(async move {
                loop {
                    4. Tries to grab the lock, but the lock is held by the first future
                    5. Since this is a std lib mutex, it blocks the executor's one thread 
                       because it doesnt know anything about futures
                    6. Lock is never released so deadlock!
                    7. Need to use async aware mutex like tokio::sync::Mutex
                    
                    *x2.lock() += 1;
                }
            })
        }

        Async Mutex are much slower!
        So can use std lib Mutex, as long as your critical section is short
        and doesn't contain any yield (await) points

    
     */

}




async fn read_to_string(_: &str) {}

// Future is a value that is not yet ready
fn foo2() -> impl Future<Output = usize > {
    async {
        println!("foo1");
        read_to_string("file1").await;
        /* 
         let x = read_to_string("file").await;
         
         Can think of it like this as a mental model
         though in practice this is not what happens

         let fut = read_to_string("file");
         while !fut.is_ready() {
            std::thread::yield_now();
            fut.try_complete();
         }
         */

        /*
         let fut = read_to_string("file");
         let x = loop {
            let Some(result) = fut.try_check_completed() {
                break result;
            } else {
                fut.try_make_progress();
                yield;
            }
         }
        */
        0
    }
}