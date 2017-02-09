#![deny(warnings)]

extern crate chan;

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

/// producer_ctor and xform_ctor is needed to allow construction
/// in the correct producer / worker thread.
/// TODO: Figure out how to simplify this trait mess :)
pub fn scatter_gather<PC, XC, P, X, J, R>(producer_ctor: PC, xform_ctor: XC) -> GatherIter<R>
    where PC: 'static + Send + FnOnce() -> P,
          XC: 'static + Send + Sync + Fn() -> X,
          X: FnMut(J) -> R,
          J: 'static + Send,
          R: 'static + Send,
          P: IntoIterator<Item = J>
{
    let jobs_rx = {
        let (tx, rx) = chan::sync(0);
        thread::spawn(move || for e in producer_ctor().into_iter() {
            tx.send(e);
        });
        rx
    };
    let results_rx = {
        let (tx, rx) = chan::sync(0);
        let xform_ctor = Arc::new(xform_ctor); // TODO: Investigate why this is needed.
        // TODO: Find and use a num_cpu function.
        for _ in 0..8 {
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
    #[test]
    fn it_works() {}
}
