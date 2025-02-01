#![allow(missing_docs)]

use readlock_tokio::Shared;
use tokio::{
    task,
    time::{sleep, Duration},
};

mod lite;

#[tokio::test]
async fn parallel_read_write() {
    let mut shared = Shared::new(1);
    let readlock = Shared::get_read_lock(&shared);

    let join_handle = task::spawn(async move { while *readlock.lock().await < 1024 {} });
    sleep(Duration::from_millis(5)).await;
    for _ in 0..10 {
        let value: i32 = *shared;
        *Shared::lock(&mut shared).await += value;
    }

    sleep(Duration::from_millis(5)).await;
    assert!(join_handle.is_finished());
}
