# 0.1.9

- Add docs.rs configuration for showing the `lite` feature

# 0.1.8

- Add `SharedReadLock::try_lock`

# 0.1.7

- Add `SharedWriteGuard::from_inner`

# 0.1.6

- Add `lite` feature flag and module

# 0.1.5

- Add `Shared::{read_count, weak_count}`

# 0.1.4

- Relax bounds for `Clone` implementations of `SharedReadLock` and `WeakReadLock`

# 0.1.3

- Add conversion functions between `Shared<T>`, `SharedReadLock<T>` and
  `Arc<RwLock<T>>` (the inner representation of both)
- Add `SharedReadGuard::from_inner`

# 0.1.2

- Add `#[clippy::has_significant_drop]` attribute to guard types so the
  [`clippy::significant_drop_in_scrutinee`] lint works with them

[`clippy::significant_drop_in_scrutinee`]: https://rust-lang.github.io/rust-clippy/master/index.html#significant_drop_in_scrutinee

# 0.1.1

- Implement `Default` for `Shared<T>`
