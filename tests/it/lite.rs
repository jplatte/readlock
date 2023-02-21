#![cfg(feature = "lite")]

use std::{self, thread, time::Duration};

use readlock::lite::Shared;

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

    thread::sleep(Duration::from_millis(5));
    assert!(join_handle.is_finished());
}
