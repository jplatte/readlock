# readlock

(Shared) Read-Only Lock:
A thing that can be useful when you don't really want shared mutability,
you just want to mutate a value from one place and read it from many others.

This library provides three types:

- `Shared<T>`: similar to `Arc<RwLock<T>>`, but you can only create
  `SharedReadLock<T>`s and `WeakReadLock<T>`s from it that share access to the
  same inner value, not further `Shared<T>`s. Also, acquiring a write lock
  requires unique ownership / borrowing (`&mut self`). However: Reading requires
  *no* locking because mutably borrowing the `Shared` means that no other thread
  can be mutating the value at the same time (all other reference to the value
  are read-only).
- `SharedReadLock<T>`: like a `Arc<RwLock<T>>` that is only ever used for
  reading. Can be downgraded to `WeakReadLock`.
- `WeakReadLock<T>`: like a `Weak<RwLock<T>>`. That is, it references the same
  memory, but if the original `Shared` and any derived `SharedReadLock`s to that
  value are dropped, it will be deallocated regardless of any `WeakReadLock`s.
  Must be upgraded into `SharedReadLock` to access the inner value.
