#![deny(warnings)]

extern crate chan;
extern crate num_cpus;

use std::marker::Send;
use std::marker::Sync;
use std::sync::Arc;
use std::thread;

/// The purpose of this newtype is to hide the channel.
pub struct GatherIter<T>(chan::Iter<T>);
impl<T> Iterator for GatherIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.0.next()
    }
}

/// producer_ctor and xform_ctor is needed to allow construction in the
/// correct producer / worker thread.
pub fn scatter_gather<PC, XC, P, X, J, R>(producer_ctor: PC, xform_ctor: XC) -> GatherIter<R>
where
    PC: 'static + Send + FnOnce() -> P,
    XC: 'static + Send + Sync + Fn() -> X,
    X: FnMut(J) -> R,
    J: 'static + Send,
    R: 'static + Send,
    P: IntoIterator<Item = J>,
{
    let jobs_rx = {
        let (tx, rx) = chan::sync(1);
        thread::spawn(move || {
            for e in producer_ctor().into_iter() {
                tx.send(e);
            }
        });
        rx
    };
    let results_rx = {
        let (tx, rx) = chan::sync(1);

        // TODO: Learn how to use lifetimes to get rid of this Arc.
        let xform_ctor = Arc::new(xform_ctor);

        for _ in 0..num_cpus::get() {
            let tx = tx.clone();
            let jobs_rx = jobs_rx.clone();
            let xform_ctor = xform_ctor.clone();
            thread::spawn(move || {
                let mut xform = xform_ctor();
                for e in jobs_rx {
                    tx.send(xform(e));
                }
            });
        }
        rx
    };
    GatherIter(results_rx.iter())
}

#[cfg(test)]
mod tests {
    use super::scatter_gather;

    /// Test helper
    fn heavy_test_work(i: u32) -> u32 {
        (0..1000000000).fold(i, |acc, x| acc.wrapping_add(x))
    }

    #[test]
    fn test_parallel_vs_sequential() {
        let prod_ctor = || (0u32..10000);
        let xform_ctor = || heavy_test_work;
        let result_xform = |acc: u32, x| acc.wrapping_add(x);
        let prod = prod_ctor();
        let par_r = scatter_gather(prod_ctor, xform_ctor).fold(0, &result_xform);
        let seq_r = prod.map(heavy_test_work).fold(0, &result_xform);
        assert_eq!(par_r, seq_r);
    }
}
