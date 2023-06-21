//! Versions of `Shared` and `SharedReadLock` that are implemented in terms of
//! the [rclite] crate. Because [`rclite::Arc`] doesn't have weak references,
//! there is no `WeakReadLock` here.

use std::{
    fmt, ops,
    sync::{LockResult, PoisonError, RwLock},
};

use rclite::Arc;

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
            Ok(rwlock) => Ok(rwlock.into_inner().unwrap()),
            Err(arc) => Err(Self(arc)),
        }
    }

    /// Get a reference to the inner value.
    ///
    /// Usually, you don't need to call this function since `Shared<T>`
    /// implements `Deref`. Use this if you want to pass the inner value to a
    /// generic function where the compiler can't infer that you want to have
    /// the `Shared` dereferenced otherwise.
    ///
    /// # Panics
    ///
    /// This function will panic if the lock around the inner value is poisoned.
    #[track_caller]
    pub fn get(this: &Self) -> &T {
        Self::try_get(this).unwrap()
    }

    /// Try to get a reference to the inner value, returning an error if the
    /// lock around it is poisoned.
    pub fn try_get(this: &Self) -> LockResult<&T> {
        match this.0.read() {
            Ok(read_guard) => Ok(unsafe { readguard_into_ref(read_guard) }),
            Err(poison_err) => {
                let read_guard = poison_err.into_inner();
                let r = unsafe { readguard_into_ref(read_guard) };
                Err(PoisonError::new(r))
            }
        }
    }

    /// Lock this `Shared` to be able to mutate it, blocking the current thread
    /// until the operation succeeds.
    pub fn lock(this: &mut Self) -> SharedWriteGuard<'_, T> {
        SharedWriteGuard(this.0.write().unwrap())
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
    /// Lock this `SharedReadLock`, blocking the current thread until the
    /// operation succeeds.
    pub fn lock(&self) -> SharedReadGuard<'_, T> {
        SharedReadGuard(self.0.read().unwrap())
    }
}

impl<T: fmt::Debug> fmt::Debug for SharedReadLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
