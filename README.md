parallel-iterator
=================

Parallelize any iterator!

Features:
 - Support for !Send and !Sync for producer iterators and transform closures.
   This allows safe and easy thread local data, using the captured closure
   environments.
 - Propagates child thread panics.
 - Internal thread handling, don't worry about it! (TODO: make this
   configurable)

A minimal example:
```rust
extern crate parallel_iterator;

use parallel_iterator::ParallelIterator;

fn do_some_work(i: u32) -> u32 {
    (0..1000).fold(i, |acc, x| acc.wrapping_add(x))
}

fn main() {
    let result_xform = |acc: u32, x| acc.wrapping_add(x);
    for i in ParallelIterator::new(|| (0u32..100), || do_some_work) {
    	println!("Got a result: {}!", i);
    }
}
```
