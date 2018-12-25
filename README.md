parallel-iterator
=================

Parallelize any iterator!

Features
--------

 - Support for !Send and !Sync for producer iterators and transform closures.
   This allows mutable, safe and easy thread local data, using the captured
   closure environments.
 - Propagates child thread panics.
 - Internal thread handling, don't worry about it! (TODO: make this
   configurable)

A minimal example
-----------------

```rust
extern crate parallel_iterator;

use parallel_iterator::ParallelIterator;

fn do_some_work(i: u32) -> u32 {
    (0..1000).fold(i, |acc, x| acc.wrapping_add(x))
}

fn main() {
    for i in ParallelIterator::new(|| (0u32..100), || do_some_work) {
    	println!("Got a result: {}!", i);
    }
}
```

Changelog
---------

### 0.1.3
 - Switched to crossbeam-channel instead of chan.
 - Updated dependencies.

### 0.1.2
 - Updated dependencies.

### 0.1.1
 - Removed dead code in the minimal example.

### 0.1.0
 - Initial publish.

License
-------

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
