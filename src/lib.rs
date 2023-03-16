#![doc = include_str!("../README.md")]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]

use std::{
    fmt, ops,
    sync::{Arc, LockResult, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak},
};

/// A wrapper around a resource possibly shared with [`SharedReadLock`]s and
/// [`WeakReadLock`]s, but no other `Shared`s.
pub struct Shared<T: ?Sized>(Arc<RwLock<T>>);

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
}

impl<T: ?Sized> Shared<T> {
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

    /// Attempt to create a `Shared` from its internal representation,
    /// `Arc<RwLock<T>>`.
    ///
    /// This returns `Ok(_)` only if there are no further references (including
    /// weak references) to the inner `RwLock` since otherwise, `Shared`s
    /// invariant of being the only instance that can mutate the inner value
    /// would be broken.
    pub fn try_from_inner(rwlock: Arc<RwLock<T>>) -> Result<Self, Arc<RwLock<T>>> {
        if Arc::strong_count(&rwlock) == 1 && Arc::weak_count(&rwlock) == 0 {
            Ok(Self(rwlock))
        } else {
            Err(rwlock)
        }
    }

    /// Turns this `Shared` into its internal representation, `Arc<RwLock<T>>`.
    pub fn into_inner(this: Self) -> Arc<RwLock<T>> {
        this.0
    }
}

/// SAFETY: Only allowed for a read guard obtained from the inner value of a
/// `Shared`. Transmuting lifetime here, this is okay because the resulting
/// reference's borrows this, which is the only `Shared` instance that could
/// mutate the inner value (you can not have two `Shared`s that reference the
/// same inner value) and the other references that can exist to the inner value
/// are only allowed to read as well.
unsafe fn readguard_into_ref<'a, T: ?Sized + 'a>(guard: RwLockReadGuard<'a, T>) -> &'a T {
    let reference: &T = &guard;
    &*(reference as *const T)
}

impl<T: ?Sized> ops::Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Shared::get(self)
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for Shared<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Default> Default for Shared<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// A read-only reference to a resource possibly shared with up to one
/// [`Shared`] and many [`WeakReadLock`]s.
pub struct SharedReadLock<T: ?Sized>(Arc<RwLock<T>>);

impl<T: ?Sized> SharedReadLock<T> {
    /// Lock this `SharedReadLock`, blocking the current thread until the
    /// operation succeeds.
    pub fn lock(&self) -> SharedReadGuard<'_, T> {
        SharedReadGuard(self.0.read().unwrap())
    }

    /// Create a new [`WeakReadLock`] pointer to this allocation.
    pub fn downgrade(&self) -> WeakReadLock<T> {
        WeakReadLock(Arc::downgrade(&self.0))
    }

    /// Upgrade a `SharedReadLock` to `Shared`.
    ///
    /// This only return `Ok(_)` if there are no other references (including a
    /// `Shared`, or weak references) to the inner value, since otherwise it
    /// would be possible to have multiple `Shared`s for the same inner value
    /// alive at the same time, which would violate `Shared`s invariant of
    /// being the only reference that is able to mutate the inner value.
    pub fn try_upgrade(self) -> Result<Shared<T>, Self> {
        if Arc::strong_count(&self.0) == 1 && Arc::weak_count(&self.0) == 0 {
            Ok(Shared(self.0))
        } else {
            Err(self)
        }
    }

    /// Create a `SharedReadLock` from its internal representation,
    /// `Arc<RwLock<T>>`.
    ///
    /// You can use this to create a `SharedReadLock` from a shared `RwLock`
    /// without ever using `Shared`, if you want to expose an API where there is
    /// a value that can be written only from inside one module or crate, but
    /// outside users should be allowed to obtain a reusable lock for reading
    /// the inner value.
    pub fn from_inner(rwlock: Arc<RwLock<T>>) -> Self {
        Self(rwlock)
    }

    /// Attempt to turn this `SharedReadLock` into its internal representation,
    /// `Arc<RwLock<T>>`.
    ///
    /// This returns `Ok(_)` only if there are no further references (including
    /// a `Shared`, or weak references) to the inner value, since otherwise
    /// it would be possible to have a `Shared` and an `Arc<RwLock<T>>` for
    /// the same inner value alive at the same time, which would violate
    /// `Shared`s invariant of being the only reference that is able to
    /// mutate the inner value.
    pub fn try_into_inner(self) -> Result<Arc<RwLock<T>>, Self> {
        if Arc::strong_count(&self.0) == 1 && Arc::weak_count(&self.0) == 0 {
            Ok(self.0)
        } else {
            Err(self)
        }
    }
}

impl<T: ?Sized> Clone for SharedReadLock<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for SharedReadLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A weak read-only reference to a resource possibly shared with up to one
/// [`Shared`] and many [`SharedReadLock`]s.
pub struct WeakReadLock<T: ?Sized>(Weak<RwLock<T>>);

impl<T: ?Sized> WeakReadLock<T> {
    /// Attempt to upgrade the `WeakReadLock` into a `SharedReadLock`, delaing
    /// dropping of the inner value if successful.
    ///
    /// Returns `None` if the inner value has already been dropped.
    pub fn upgrade(&self) -> Option<SharedReadLock<T>> {
        Weak::upgrade(&self.0).map(SharedReadLock)
    }
}

impl<T: ?Sized> Clone for WeakReadLock<T> {
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for WeakReadLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// RAII structure used to release the shared read access of a lock when
/// dropped.
#[clippy::has_significant_drop]
pub struct SharedReadGuard<'a, T: ?Sized>(RwLockReadGuard<'a, T>);

impl<'a, T: ?Sized + 'a> SharedReadGuard<'a, T> {
    /// Create a `SharedReadGuard` from its internal representation,
    /// `RwLockReadGuard<'a, T>`.
    pub fn from_inner(guard: RwLockReadGuard<'a, T>) -> Self {
        Self(guard)
    }
}

impl<'a, T: ?Sized + 'a> ops::Deref for SharedReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: fmt::Debug + ?Sized + 'a> fmt::Debug for SharedReadGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// RAII structure used to release the exclusive write access of a lock when
/// dropped.
#[clippy::has_significant_drop]
pub struct SharedWriteGuard<'a, T: ?Sized>(RwLockWriteGuard<'a, T>);

impl<'a, T: ?Sized + 'a> ops::Deref for SharedWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: ?Sized + 'a> ops::DerefMut for SharedWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: fmt::Debug + ?Sized + 'a> fmt::Debug for SharedWriteGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
