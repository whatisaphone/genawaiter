# genawaiter

[![crate-badge]][crate-link] [![docs-badge]][docs-link] [![ci-badge]][ci-link]

[crate-badge]: https://img.shields.io/crates/v/genawaiter.svg
[crate-link]: https://crates.io/crates/genawaiter
[docs-badge]: https://docs.rs/genawaiter/badge.svg
[docs-link]: https://docs.rs/genawaiter
[ci-badge]: https://github.com/whatisaphone/genawaiter/workflows/CI/badge.svg
[ci-link]: https://github.com/whatisaphone/genawaiter/actions

This crate implements stackless generators (aka coroutines) in stable Rust. Instead of using `yield`, which [won't be stabilized anytime soon][yield-unstable], you use `async`/`await`, which is stable today.

[yield-unstable]: https://doc.rust-lang.org/nightly/unstable-book/language-features/generators.html

Features:

- supports resume arguments and completion values
- supports async generators (e.g., `Stream`s)
- allocation-free
- no runtime dependencies
  - no compile-time dependencies either, with `default-features = false`
- built on top of standard language constructs, which means there are no platform-specific shenanigans

Example:

```rust
let odd_numbers_less_than_ten = gen!({
    let mut n = 1;
    while n < 10 {
        yield_!(n); // Suspend a function at any point with a value.
        n += 2;
    }
});

// Generators can be used as ordinary iterators.
for num in odd_numbers_less_than_ten {
    println!("{}", num);
}
```

Result:

```text
1
3
5
7
9
```

And here is the same generator, this time without macros. This is how you do things with `default-features = false` (which eliminates the proc macro dependencies).

```rust
let odd_numbers_less_than_ten = Gen::new(|co| async move {
    let mut n = 1;
    while n < 10 {
        co.yield_(n).await;
        n += 2;
    }
});
```

[See the docs for more.](https://docs.rs/genawaiter)
