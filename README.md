parallel-iterator
=================

[![Safety Dance](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

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

This code is copy-pasted from `examples/example_1.rs`.

```rust
extern crate parallel_iterator;

use parallel_iterator::ParallelIterator;

fn do_some_work(i: u32) -> u32 {
    i + 1 // let's pretend this is a heavy calculation
}

fn main() {
    for i in ParallelIterator::new(|| (0u32..100), || do_some_work) {
    	println!("Got a result: {}!", i);
    }
}
```


A _slightly_ more realistic example
-----------------------------------

This code is copy-pasted from `examples/example_2.rs`.

```rust
extern crate parallel_iterator;

use parallel_iterator::ParallelIterator;

fn do_some_work(i: usize, out: &mut Vec<usize>) {
    for j in 0..i {
        out.push(j); // The caller can pre-allocate.
    }
}

fn main() {
    const MAX: usize = 1000;
    let xform_ctor = || {
        let mut buffer = Vec::with_capacity(MAX);
        move |i| {
            buffer.clear(); // Clear but keep the internal allocation.
            do_some_work(i, &mut buffer);
            buffer.last().map(|u| *u) // This is just an example value.
        }
    };
    for i in ParallelIterator::new(|| (0..MAX), xform_ctor) {
        match i {
            Some(i) => println!("Got Some({})!", i),
            None => println!("Got None!"),
        }
    }
}
```

Please see the documentation on the ParallelIterator struct for more details.

Changelog
---------

### 0.1.5
 - Updated dependencies.

### 0.1.4
 - Added examples.
 - Improved documentation.
 - Updated dependencies.

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
