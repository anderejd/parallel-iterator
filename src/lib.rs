#![forbid(warnings)]
#![forbid(unsafe_code)]

extern crate crossbeam_channel;
extern crate num_cpus;

use std::marker::Send;
use std::marker::Sync;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use crossbeam_channel::bounded;

/// <O> The type returned by the Iterator::next method.
pub struct ParallelIterator<O> {
    channel: crossbeam_channel::IntoIter<O>,
    threads: Vec<JoinHandle<()>>,
}

impl< O> ParallelIterator< O> {
    /// <PC> Producer Constructor. Enables usage of !Send and !Sync objects in the
    /// producer function.
    ///
    /// <XC> Xform Constructor. Enables usage of !Send and !Sync objects in the
    /// producer function. This can be useful for thread local caches and re-using
    /// large allocations between different tasks, packaged as a closure.
    ///
    /// <P> Producer iterator. Consumed internally by the transform/worker threads.
    ///
    /// <X> Xform closure. Applied to each job item produced by the producer
    /// iterator, in parallel by multiple worker threads.
    ///
    /// <I> Input item. Or task, produced by the producer iterator, transformed
    /// by the Xform closures.
    ///
    /// <O> Output item. Returned by the Xform closure(s) and by the
    /// Iterator::next method.
    ///
    pub fn new<PC, XC, P, X, I>(
        producer_ctor: PC,
        xform_ctor: XC,
    ) -> Self
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
                        // error and the panic should propagate to parent
                        // thread.
                        tx.send(xform(e)).expect("Worker thread failed to send result.");
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

    fn join_threads(&mut self)
    {
        while let Some(join_handle) = self.threads.pop() {
            // Using expect() here since trying to get the inner panic message
            // in a typesafe way is not possible?
            join_handle
                .join()
                .expect("A child thread has paniced.");
        }
    }
}

impl< T> Iterator for ParallelIterator< T> {
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
        let par_r = ParallelIterator::new(prod_ctor, xform_ctor).fold(0, &result_xform);
        let seq_r = prod.map(do_some_work).fold(0, &result_xform);
        assert_eq!(par_r, seq_r);
    }
}
