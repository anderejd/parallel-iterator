//! parallel-iterator
//! =================
//!
//! A minimal example
//! -----------------
//!
//! This code is copy-pasted from `examples/example_1.rs`.
//!
//! ```rust
//! extern crate parallel_iterator;
//!
//! use parallel_iterator::ParallelIterator;
//!
//! fn do_some_work(i: u32) -> u32 {
//!     i + 1 // let's pretend this is a heavy calculation
//! }
//!
//! fn main() {
//!     for i in ParallelIterator::new(|| (0u32..100), || do_some_work) {
//!     	println!("Got a result: {}!", i);
//!     }
//! }
//! ```
//!
//!
//! A _slightly_ more realistic example
//! -----------------------------------
//!
//! This code is copy-pasted from `examples/example_2.rs`.
//!
//! ```rust
//! extern crate parallel_iterator;
//!
//! use parallel_iterator::ParallelIterator;
//!
//! fn do_some_work(i: usize, out: &mut Vec<usize>) {
//!     for j in 0..i {
//!         out.push(j); // The caller can pre-allocate.
//!     }
//! }
//!
//! fn main() {
//!     const MAX: usize = 1000;
//!     let xform_ctor = || {
//!         let mut buffer = Vec::with_capacity(MAX);
//!         move |i| {
//!             buffer.clear(); // Clear but keep the internal allocation.
//!             do_some_work(i, &mut buffer);
//!             buffer.last().map(|u| *u) // This is just an example value.
//!         }
//!     };
//!     for i in ParallelIterator::new(|| (0..MAX), xform_ctor) {
//!         match i {
//!             Some(i) => println!("Got Some({})!", i),
//!             None => println!("Got None!"),
//!         }
//!     }
//! }
//! ```
//!
//! Please see the documentation on the ParallelIterator struct for more details.

#![forbid(warnings)]
#![forbid(unsafe_code)]

extern crate crossbeam_channel;
extern crate num_cpus;

use crossbeam_channel::bounded;
use std::marker::Send;
use std::marker::Sync;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

pub struct ParallelIterator<O> {
    channel: crossbeam_channel::IntoIter<O>,
    threads: Vec<JoinHandle<()>>,
}

impl<O> ParallelIterator<O> {
    /// * `PC` - Producer Constructor. Enables usage of !Send and !Sync objects
    /// in the producer function.
    ///
    /// * `XC` - Xform (closure) Constructor. Enables usage of !Send and !Sync 
    /// objects in the transform closure. This can be used for creating thread 
    /// local caches, like large allocations re-used by different tasks, 
    /// packaged as a closure.
    ///
    /// * `P` - Producer iterator. Consumed internally by the transform/worker
    /// threads.
    ///
    /// * `X` - Xform closure. Applied to each job item produced by the producer
    /// iterator, in parallel by multiple worker threads. This can be `FnMut`
    /// since it's owned by a dedicated worker thread and will never be called
    /// by some other thread. The closure can safely store and reuse mutable
    /// resources between job items, for example large memory allocations.
    ///
    /// * `I` - Input item. Or task, produced by the producer iterator,
    /// transformed by the Xform closures.
    ///
    /// * `O` - Output item. Returned by the Xform closure(s) and by the
    /// Iterator::next method.
    ///
    pub fn new<PC, XC, P, X, I>(producer_ctor: PC, xform_ctor: XC) -> Self
    where
        PC: 'static + Send + FnOnce() -> P,
        XC: 'static + Send + Sync + Fn() -> X,
        X: FnMut(I) -> O,
        I: 'static + Send,
        O: 'static + Send,
        P: IntoIterator<Item = I>,
    {
        let mut threads = vec![];
        let jobs_rx = {
            let (tx, rx) = bounded(1);
            let join_handle = thread::spawn(move || {
                for e in producer_ctor() {
                    // Using expect here since this is most likely a fatal error
                    // and the panic should propagate to parent thread.
                    tx.send(e).expect("Producer thread failed to send job.");
                }
            });
            threads.push(join_handle);
            rx
        };
        let results_rx = {
            let (tx, rx) = bounded(1);
            let xform_ctor = Arc::new(xform_ctor);
            for _ in 0..num_cpus::get() {
                let tx = tx.clone();
                let jobs_rx = jobs_rx.clone();
                let xform_ctor = xform_ctor.clone();
                let join_handle = thread::spawn(move || {
                    let mut xform = xform_ctor();
                    for e in jobs_rx {
                        // Using expect here since this is most likely a fatal
                        // error and the panic should propagate to the parent
                        // thread.
                        tx.send(xform(e))
                            .expect("Worker thread failed to send result.");
                    }
                });
                threads.push(join_handle);
            }
            rx
        };
        Self {
            channel: results_rx.into_iter(),
            threads,
        }
    }

    fn join_threads(&mut self) {
        while let Some(join_handle) = self.threads.pop() {
            // Using expect() here since trying to get the inner panic message
            // in a typesafe way is not possible?
            join_handle.join().expect("A child thread has paniced.");
        }
    }
}

impl<T> Iterator for ParallelIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let item = self.channel.next();
        if item.is_some() {
            return item;
        }
        self.join_threads();
        item // Should always be None here.
    }
}

#[cfg(test)]
mod tests {
    use super::ParallelIterator;

    /// Test helper
    fn do_some_work(i: u32) -> u32 {
        (0..1000).fold(i, |acc, x| acc.wrapping_add(x))
    }

    #[test]
    fn test_parallel_vs_sequential() {
        let prod_ctor = || (0u32..100);
        let xform_ctor = || do_some_work;
        let result_xform = |acc: u32, x| acc.wrapping_add(x);
        let prod = prod_ctor();
        let par_r =
            ParallelIterator::new(prod_ctor, xform_ctor).fold(0, &result_xform);
        let seq_r = prod.map(do_some_work).fold(0, &result_xform);
        assert_eq!(par_r, seq_r);
    }
}
