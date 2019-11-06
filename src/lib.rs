/*!
This crate implements generators for Rust. Generators are a feature common across many
programming language. They let you yield a sequence of values from a function. A few
common use cases are:

- Easily building iterators.
- Avoiding allocating a list for a function which returns multiple values.

Rust has this feature too, but it is currently unstable (and thus nightly-only). But
with this crate, you can use them on stable Rust!

# A tale of three types

A generator can control the flow of up to three types of data:

- **Yield** – Each time a generator suspends execution, it can produce a value.
- **Resume** – Each time a generator is resumed, a value can be passed in.
- **Completion** – When a generator completes, it can produce one final value.

The three types are specified in the type signature of the generator. Only the first
is required; the last two are optional:

```rust
# use genawaiter::rc::Co;
#
type Yield = // ...
#     ();
type Resume = // ...
#     ();
type Completion = // ...
#     ();

async fn generator(co: Co<Yield, Resume>) -> Completion { /* ... */ }
```

## Yielded values

Values can be yielded from the generator by calling `yield_`, and immediately awaiting
the future it returns. You can get these values out of the generator in either of two
ways:

- Call `resume()` or `resume_with()`. The values will be returned in a
  `GeneratorState::Yielded`.

  ```rust
  # use genawaiter::{GeneratorState, rc::{Co, Gen}};
  #
  async fn give_me_a_ten(co: Co<i32>) {
      co.yield_(10).await;
  }

  let mut generator = Gen::new(give_me_a_ten);
  let ten = generator.resume();
  assert_eq!(ten, GeneratorState::Yielded(10));
  ```

- Treat it as an iterator. For this to work, both the resume and completion types must
  be `()` .

  ```rust
  # use genawaiter::rc::{Co, Gen};
  #
  async fn give_me_a_ten(co: Co<i32>) {
      co.yield_(10).await;
  }

  let generator = Gen::new(give_me_a_ten);
  let xs: Vec<_> = generator.into_iter().collect();
  assert_eq!(xs, [10]);
  ```

## Resume arguments

You can also send values back into the generator, by using `resume_with`. The generator
receives them as the output of the future returned by `yield_`.

```rust
# use genawaiter::{GeneratorState, rc::{Co, Gen}};
#
async fn printer(co: Co<(), &'static str>) {
    loop {
        let text = co.yield_(()).await;
        println!("{}", text);
    }
}

let mut generator = Gen::new(printer);
generator.resume_with("hello");
generator.resume_with("world");
```

## Completion value

A generator can produce one final value upon completion, by returning it from the
function. The consumer will receive this value as a `GeneratorState::Complete`.

```rust
# use genawaiter::{GeneratorState, rc::{Co, Gen}};
#
async fn foobar(co: Co<i32>) -> &'static str {
    co.yield_(10).await;
    "done"
}

let mut generator = Gen::new(foobar);
assert_eq!(generator.resume(), GeneratorState::Yielded(10));
assert_eq!(generator.resume(), GeneratorState::Complete("done"));
```

# Backported stdlib types

This crate supplies [`Generator`](trait.Generator.html) and
[`GeneratorState`](enum.GeneratorState.html). They are copy/pasted from the stdlib (with
stability attributes removed) so they can be used on stable Rust. If/when real
generators are stabilized, hopefully they would be drop-in replacements. Javscript
developers might recognize this as a polyfill.

There is also a [`Coroutine`](trait.Coroutine.html) trait, which does not come from the
stdlib. A `Coroutine` is a generalization of a `Generator`. A `Generator` constrains the
resume argument type to `()`, but in a `Coroutine` it can be anything.

# Choose your guarantees

This crate supplies two concrete implementations of the
[`Coroutine`](trait.Coroutine.html) trait:

1. [`genawaiter::rc`](rc) – This uses 100% safe code, but requires allocation.
2. [`genawaiter::stack`](stack) – This works without allocating memory, but has a number
   of downsides:

   - It uses a macro.
   - It uses unsafe code under the hood.
   - It is possible to violate memory safety (but only if you do silly things with the
     `co` object).
*/

#![cfg_attr(feature = "nightly", feature(async_await, async_closure))]
#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(missing_docs, clippy::cargo, clippy::pedantic)]
#![cfg_attr(feature = "strict", deny(warnings))]

pub use ops::{Coroutine, Generator, GeneratorState};

mod ops;
pub mod rc;
pub mod stack;
#[cfg(test)]
mod testing;
mod waker;
