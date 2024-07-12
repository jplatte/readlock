//! Versions of `Shared` and `SharedReadLock` that are implemented in terms of
//! the [rclite] crate. Because [`rclite::Arc`] doesn't have weak references,
//! there is no `WeakReadLock` here.

use std::{fmt, ops};

use rclite::Arc;
use tokio::sync::RwLock;

use crate::{readguard_into_ref, SharedReadGuard, SharedWriteGuard};

/// A wrapper around a resource possibly shared with [`SharedReadLock`]s, but no
/// other `Shared`s.
pub struct Shared<T>(Arc<RwLock<T>>);

impl<T> Shared<T> {
    /// Create a new `Shared`.
    pub fn new(data: T) -> Self {
        Self(Arc::new(RwLock::new(data)))
    }

    /// Returns the inner value, if the `Shared` has no associated
    /// `SharedReadLock`s.
    ///
    /// Otherwise, an `Err` is returned with the same `Shared` that was passed
    /// in.
    ///
    /// This will succeed even if there are outstanding weak references.
    ///
    /// # Panics
    ///
    /// This function will panic if the lock around the inner value is poisoned.
    pub fn unwrap(this: Self) -> Result<T, Self> {
        match Arc::try_unwrap(this.0) {
            Ok(rwlock) => Ok(rwlock.into_inner()),
            Err(arc) => Err(Self(arc)),
        }
    }

    /// Get a reference to the inner value.
    ///
    /// Usually, you don't need to call this function since `Shared<T>`
    /// implements `Deref`. Use this if you want to pass the inner value to a
    /// generic function where the compiler can't infer that you want to have
    /// the `Shared` dereferenced otherwise.
    #[track_caller]
    pub fn get(this: &Self) -> &T {
        let read_guard =
            this.0.try_read().expect("nothing else can hold a write lock at this time");
        unsafe { readguard_into_ref(read_guard) }
    }

    /// Lock this `Shared` to be able to mutate it, causing the current task to
    /// yield until the lock has been acquired.
    pub async fn lock(this: &mut Self) -> SharedWriteGuard<'_, T> {
        SharedWriteGuard(this.0.write().await)
    }

    /// Get a [`SharedReadLock`] for accessing the same resource read-only from
    /// elsewhere.
    pub fn get_read_lock(this: &Self) -> SharedReadLock<T> {
        SharedReadLock(this.0.clone())
    }
}

impl<T> ops::Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Shared::get(self)
    }
}

impl<T: fmt::Debug> fmt::Debug for Shared<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A read-only reference to a resource possibly shared with up to one
/// [`Shared`] and many other [`SharedReadLock`]s.
#[derive(Clone)]
pub struct SharedReadLock<T>(Arc<RwLock<T>>);

impl<T> SharedReadLock<T> {
    /// Lock this `SharedReadLock`, causing the current task to
    /// yield until the lock has been acquired.
    pub async fn lock(&self) -> SharedReadGuard<'_, T> {
        SharedReadGuard(self.0.read().await)
    }

    /// Try to lock this `SharedReadLock`.
    pub fn try_lock(&self) -> Option<SharedReadGuard<'_, T>> {
        // FIXME: Custom TryLockError?
        self.0.try_read().ok().map(SharedReadGuard)
    }
}

impl<T: fmt::Debug> fmt::Debug for SharedReadLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
