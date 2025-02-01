#![allow(missing_docs)]

use std::{self, thread, time::Duration};

use readlock::Shared;

mod lite;

#[test]
fn parallel_read_write() {
    let mut shared = Shared::new(1);
    let readlock = Shared::get_read_lock(&shared);

    let join_handle = thread::spawn(move || while *readlock.lock() < 1024 {});
    thread::sleep(Duration::from_millis(5));
    for _ in 0..10 {
        let value: i32 = *shared;
        *Shared::lock(&mut shared) += value;
    }

    if cfg!(miri) {
        join_handle.join().unwrap();
    } else {
        thread::sleep(Duration::from_millis(5));
        assert!(join_handle.is_finished());
    }
}
