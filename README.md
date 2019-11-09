# genawaiter [![crate-badge]][crate-link] [![docs-badge]][docs-link] [![ci-badge]][ci-link]

[crate-badge]: https://img.shields.io/crates/v/genawaiter.svg
[crate-link]: https://crates.io/crates/genawaiter
[docs-badge]: https://docs.rs/genawaiter/badge.svg
[docs-link]: https://docs.rs/genawaiter
[ci-badge]: https://github.com/whatisaphone/genawaiter/workflows/CI/badge.svg
[ci-link]: https://github.com/whatisaphone/genawaiter/actions

This crate implements stackless generators (aka coroutines) in stable Rust. Instead of using `yield`, which [won't be stabilized anytime soon][yield-unstable], you use `async`/`await`, which is stable today:

[yield-unstable]: https://doc.rust-lang.org/nightly/unstable-book/language-features/generators.html

```rust
let generator = Gen::new(|co| async move {
    let mut n = 1;
    while n < 10 {
        // Suspend a function at any point with a value.
        co.yield_(n).await;
        n += 2;
    }
});

// Generators can be used as ordinary iterators.
for num in generator {
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

[See the docs for more.](https://docs.rs/genawaiter)

## Development

### Install prerequisites

- [Rust]
- [pre-commit]

[Rust]: https://www.rust-lang.org/
[pre-commit]: https://pre-commit.com/

### Install the pre-commit hook

```sh
pre-commit install
```

This installs a Git hook that runs a quick sanity check before every commit.

### Run the app

```sh
cargo run
```

### Run the tests

```sh
cargo test
```
