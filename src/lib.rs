/*!
This crate implements generators for Rust. Generators are a feature common across many
programming language. They let you yield a sequence of values from a function. A few
common use cases are:

- Easily building iterators.
- Avoiding allocating a list for a function which returns multiple values.

Rust has this feature too, but it is currently unstable (and thus nightly-only). But
with this crate, you can use them on stable Rust!

# Features

This crate has these features:

- `futures03` (disabled by default) – Implements `Stream` for all generator types.
  Adds a dependency on `futures-core`.
- `proc_macro` (enabled by default) – Adds support for macros, and adds various
  compile-time dependencies.

# Choose your guarantees

This crate supplies three concrete implementations of generators:

1. [`genawaiter::stack`](stack) – Allocation-free. You should prefer this when possible.

2. [`genawaiter::rc`](rc) – This allocates.

3. [`genawaiter::sync`](sync) – This allocates, and can be shared between threads.

   [unus]: https://github.com/whatisaphone/genawaiter/blob/4a2b185/src/waker.rs#L9
   [duo]: https://github.com/whatisaphone/genawaiter/blob/4a2b185/src/rc/engine.rs#L26

Here are the differences in table form:

|                                       | [`stack::Gen`] | [`rc::Gen`] | [`sync::Gen`] |
|---------------------------------------|----------------|-------------|---------------|
| Allocations per generator            | 0               | 2           | 2             |
| Generator can be moved after created | no              | yes         | yes           |
| Thread-safe                          | no              | no          | yes           |

# Creating a generator

Once you've chosen how and whether to allocate (see previous section), you can create a
generator using a macro from the `gen` family:

- [`stack::let_gen!`](stack/macro.let_gen.html)
- [`rc::gen!`](rc/macro.gen.html)
- [`sync::gen!`](sync/macro.gen.html)

```rust
# #[cfg(feature = "proc_macro")]
# fn feature_gate() {
# use genawaiter::{sync::gen, yield_};
#
let count_to_ten = gen!({
    for n in 0..10 {
        yield_!(n);
    }
});

# let result: Vec<_> = count_to_ten.into_iter().collect();
# assert_eq!(result, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
# }
```

To re-use logic between multiple generators, you can use a macro from the `producer`
family, and then pass the producer to `Gen::new`.

- [`stack_producer!`] and [`let_gen_using!`](stack/macro.let_gen_using.html)
- [`rc_producer!`] and [`Gen::new`](rc::Gen::new)
- [`sync_producer!`] and [`Gen::new`](sync::Gen::new)

```rust
# #[cfg(feature = "proc_macro")]
# fn feature_gate() {
# use genawaiter::{sync::Gen, sync_producer as producer, yield_};
#
let count_producer = producer!({
    for n in 0..10 {
        yield_!(n);
    }
});

let count_to_ten = Gen::new(count_producer);

# let result: Vec<_> = count_to_ten.into_iter().collect();
# assert_eq!(result, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
# }
```

If neither of these offers enough control for you, you can always skip the macros and
use the low-level API directly:

```rust
# use genawaiter::sync::{Co, Gen};
#
let count_to_ten = Gen::new(|mut co| async move {
    for n in 0..10 {
        co.yield_(n).await;
    }
});

# let result: Vec<_> = count_to_ten.into_iter().collect();
# assert_eq!(result, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
```

# A tale of three types

A generator can control the flow of up to three types of data:

- **Yield** – Each time a generator suspends execution, it can produce a value.
- **Resume** – Each time a generator is resumed, a value can be passed in.
- **Completion** – When a generator completes, it can produce one final value.

## Yield

Values can be yielded from the generator by calling `yield_`, and immediately awaiting
the future it returns. You can get these values out of the generator in either of two
ways:

- Call `resume()` or `resume_with()`. The values will be returned in a
  `GeneratorState::Yielded`.

  ```rust
  # #[cfg(feature = "proc_macro")]
  # fn feature_gate() {
  # use genawaiter::{sync::gen, yield_, GeneratorState};
  #
  let mut generator = gen!({
      yield_!(10);
  });
  let ten = generator.resume();
  assert_eq!(ten, GeneratorState::Yielded(10));
  # }
  ```

- Treat it as an iterator. For this to work, both the resume and completion types must
  be `()` .

  ```rust
  # #[cfg(feature = "proc_macro")]
  # fn feature_gate() {
  # use genawaiter::{sync::gen, yield_};
  #
  let generator = gen!({
      yield_!(10);
  });
  let xs: Vec<_> = generator.into_iter().collect();
  assert_eq!(xs, [10]);
  # }
  ```

## Resume

You can also send values back into the generator, by using `resume_with`. The generator
receives them from the future returned by `yield_`.

```rust
# #[cfg(feature = "proc_macro")]
# fn feature_gate() {
# use genawaiter::{sync::gen, yield_};
#
let mut printer = gen!({
    loop {
        let string = yield_!(());
        println!("{}", string);
    }
});
printer.resume_with("hello");
printer.resume_with("world");
# }
```

## Completion

A generator can produce one final value upon completion, by returning it from the
function. The consumer will receive this value as a `GeneratorState::Complete`.

```rust
# #[cfg(feature = "proc_macro")]
# fn feature_gate() {
# use genawaiter::{sync::gen, yield_, GeneratorState};
#
let mut generator = gen!({
    yield_!(10);
    "done"
});
assert_eq!(generator.resume(), GeneratorState::Yielded(10));
assert_eq!(generator.resume(), GeneratorState::Complete("done"));
# }
```

# Async generators

If you await other futures inside the generator, it becomes an _async generator_. It
does not makes sense to treat an async generator as an `Iterable`, since you cannot
`await` an `Iterable`. Instead, you can treat it as a `Stream`. This requires opting in
to the dependency on `futures` with the `futures03` feature.

```toml
[dependencies]
genawaiter = { version = "...", features = ["futures03"] }
```

```rust
# #[cfg(all(feature = "proc_macro", feature = "futures03"))]
# fn feature_gate() {
# use futures::executor::block_on_stream;
# use genawaiter::{sync::gen, yield_};
#
async fn async_one() -> i32 { 1 }
async fn async_two() -> i32 { 2 }

let gen = gen!({
    let one = async_one().await;
    yield_!(one);
    let two = async_two().await;
    yield_!(two);
});
let stream = block_on_stream(gen);
let items: Vec<_> = stream.collect();
assert_eq!(items, [1, 2]);
# }
```

Async generators also provide a `async_resume` method for lower-level control. (This
works even without the `futures03` feature.)

```rust
# #[cfg(feature = "proc_macro")]
# async fn feature_gate() {
# use genawaiter::{sync::gen, yield_, GeneratorState};
# use std::task::Poll;
#
# let mut gen = gen!({
#     yield_!(10);
# });
#
match gen.async_resume().await {
    GeneratorState::Yielded(_) => {}
    GeneratorState::Complete(_) => {}
}
# }
```

# Backported stdlib types

This crate supplies [`Generator`](trait.Generator.html) and
[`GeneratorState`](enum.GeneratorState.html). They are copy/pasted from the stdlib (with
stability attributes removed) so they can be used on stable Rust. If/when real
generators are stabilized, hopefully they would be drop-in replacements. Javascript
developers might recognize this as a polyfill.

There is also a [`Coroutine`](trait.Coroutine.html) trait, which does not come from the
stdlib. A `Coroutine` is a generalization of a `Generator`. A `Generator` constrains the
resume argument type to `()`, but in a `Coroutine` it can be anything.
*/

#![cfg_attr(feature = "nightly", feature(async_closure))]
#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(missing_docs, clippy::cargo, clippy::pedantic)]
#![cfg_attr(feature = "strict", deny(warnings))]

#[cfg(test)]
extern crate self as genawaiter;

pub use crate::ops::{Coroutine, Generator, GeneratorState};

#[cfg(feature = "proc_macro")]
use proc_macro_hack::proc_macro_hack;

/// Creates a producer for use with [`sync::Gen`].
///
/// A producer can later be turned into a generator using
/// [`Gen::new`](sync::Gen::new).
///
/// This macro takes one argument, which should be a block containing one or
/// more calls to [`yield_!`].
///
/// # Example
///
/// ```rust
/// use genawaiter::{sync::Gen, sync_producer as producer, yield_};
///
/// let my_producer = producer!({
///     yield_!(10);
/// });
///
/// let mut my_generator = Gen::new(my_producer);
/// # my_generator.resume();
/// ```
#[cfg(feature = "proc_macro")]
#[proc_macro_hack]
pub use genawaiter_proc_macro::sync_producer;

/// Creates a producer for use with [`rc::Gen`].
///
/// A producer can later be turned into a generator using
/// [`Gen::new`](rc::Gen::new).
///
/// This macro takes one argument, which should be a block containing one or
/// more calls to [`yield_!`].
///
/// # Example
///
/// ```rust
/// use genawaiter::{rc::Gen, rc_producer as producer, yield_};
///
/// let my_producer = producer!({
///     yield_!(10);
/// });
///
/// let mut my_generator = Gen::new(my_producer);
/// # my_generator.resume();
/// ```
#[cfg(feature = "proc_macro")]
#[proc_macro_hack]
pub use genawaiter_proc_macro::rc_producer;

#[doc(hidden)] // This is not quite usable currently, so hide it for now.
#[cfg(feature = "proc_macro")]
#[proc_macro_hack]
pub use genawaiter_proc_macro::stack_producer;

mod core;
mod ext;
#[macro_use]
mod macros;
mod ops;
pub mod rc;
pub mod stack;
pub mod sync;
#[cfg(test)]
mod testing;
mod waker;
