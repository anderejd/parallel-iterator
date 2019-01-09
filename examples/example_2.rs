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
