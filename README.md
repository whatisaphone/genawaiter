# genawaiter

[![crates.io](https://img.shields.io/crates/v/genawaiter.svg)](https://crates.io/crates/genawaiter)
[![Docs](https://docs.rs/genawaiter/badge.svg)](https://docs.rs/genawaiter)
[![CI](https://github.com/whatisaphone/genawaiter/workflows/CI/badge.svg)](https://github.com/whatisaphone/genawaiter/actions)

This crate implements stackless generators (aka coroutines) in stable Rust. Instead of using `yield`, which [won't be stabilized anytime soon][yield-unstable], you use `async`/`await`, which is stable today:

[yield-unstable]: https://doc.rust-lang.org/nightly/unstable-book/language-features/generators.html

```rust
async fn odd_numbers_less_than_ten(co: Co<i32>) {
    let mut n = 1;
    while n < 10 {
        co.yield_(n).await;
        n += 2;
    }
}

for n in Gen::new(odd_numbers_less_than_ten) {
    println!("{}", n);
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
