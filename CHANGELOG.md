# 0.1.2

- Add `#[clippy::has_significant_drop]` attribute to guard types so the
  [`clippy::significant_drop_in_scrutinee`] lint works with them

[`clippy::significant_drop_in_scrutinee`]: https://rust-lang.github.io/rust-clippy/master/index.html#significant_drop_in_scrutinee

# 0.1.1

- Implement `Default` for `Shared<T>`
