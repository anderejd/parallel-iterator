extern crate parallel_iterator;

use parallel_iterator::ParallelIterator;

fn do_some_work(i: u32) -> u32 {
    i + 1 // let's pretend this is a heavy calculation
}

fn main() {
    for i in ParallelIterator::new(|| (0u32..100), || do_some_work) {
        println!("Got a number: {}!", i);
    }
}
