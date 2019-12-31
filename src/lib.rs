/*!
This crate implements generators for Rust. Generators are a feature common across many
programming language. They let you yield a sequence of values from a function. A few
common use cases are:

- Easily building iterators.
- Avoiding allocating a list for a function which returns multiple values.

Rust has this feature too, but it is currently unstable (and thus nightly-only). But
with this crate, you can use them on stable Rust!

# Choose your guarantees

This crate supplies three concrete implementations of generators:

1. [`genawaiter::stack`](stack) – Safe and allocation-free. You should prefer this in
   most cases.

2. [`genawaiter::sync`](sync) – This can be shared between threads and stored in a
   `static` variable. To make this possible, it stores its state on the heap.

3. [`genawaiter::rc`](rc) – This is single-threaded and also allocates. Using this is
   discouraged, and you should feel discouraged. Its only advantages over `stack` are
   (1) it doesn't use macros, and (2) it only has [two][unus] [lines][duo] of
   unsafe code, which are trivially auditable.

   [unus]: https://github.com/whatisaphone/genawaiter/blob/4a2b185/src/waker.rs#L9
   [duo]: https://github.com/whatisaphone/genawaiter/blob/4a2b185/src/rc/engine.rs#L26

Read on for more general info about how generators work, and how data flows in and out
of a generator.

# A tale of three types

A generator can control the flow of up to three types of data:

- **Yield** – Each time a generator suspends execution, it can produce a value.
- **Resume** – Each time a generator is resumed, a value can be passed in.
- **Completion** – When a generator completes, it can produce one final value.

The three types are specified in the type signature of the generator. Only the first
is required; the last two are optional:

```rust
# use genawaiter::rc::{Co, Gen};
#
type Yield = // ...
#     ();
type Resume = // ...
#     ();
type Completion = // ...
#     ();

async fn generator(co: Co<Yield, Resume>) -> Completion
# {}
# Gen::new(generator);
```

Rewritten as a non-`async` function, the above function has the same type as:

```rust
# use genawaiter::rc::{Co, Gen};
# use std::{future::Future, pin::Pin, task::{Context, Poll}};
#
# type Yield = ();
# type Resume = ();
# type Completion = ();
#
fn generator(co: Co<Yield, Resume>) -> impl Future<Output = Completion>
# {
#     struct DummyFuture;
#     impl Future for DummyFuture {
#         type Output = ();
#         fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
#             Poll::Pending
#         }
#     }
#     DummyFuture
# }
# Gen::new(generator);
```

## Yielded values

Values can be yielded from the generator by calling `yield_`, and immediately awaiting
the future it returns. You can get these values out of the generator in either of two
ways:

- Call `resume()` or `resume_with()`. The values will be returned in a
  `GeneratorState::Yielded`.

  ```rust
  # use genawaiter::{GeneratorState, rc::Gen};
  #
  let mut generator = Gen::new(|co| async move {
      co.yield_(10).await;
  });
  let ten = generator.resume();
  assert_eq!(ten, GeneratorState::Yielded(10));
  ```

- Treat it as an iterator. For this to work, both the resume and completion types must
  be `()` .

  ```rust
  # use genawaiter::rc::Gen;
  #
  let generator = Gen::new(|co| async move {
      co.yield_(10).await;
  });
  let xs: Vec<_> = generator.into_iter().collect();
  assert_eq!(xs, [10]);
  ```

## Resume arguments

You can also send values back into the generator, by using `resume_with`. The generator
receives them from the future returned by `yield_`.

```rust
# use genawaiter::{GeneratorState, rc::Gen};
#
let mut printer = Gen::new(|co| async move {
    loop {
        let string = co.yield_(()).await;
        println!("{}", string);
    }
});
printer.resume_with("hello");
printer.resume_with("world");
```

## Completion value

A generator can produce one final value upon completion, by returning it from the
function. The consumer will receive this value as a `GeneratorState::Complete`.

```rust
# use genawaiter::{GeneratorState, rc::Gen};
#
let mut generator = Gen::new(|co| async move {
    co.yield_(10).await;
    "done"
});
assert_eq!(generator.resume(), GeneratorState::Yielded(10));
assert_eq!(generator.resume(), GeneratorState::Complete("done"));
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
# use futures::executor::block_on_stream;
# use genawaiter::{GeneratorState, rc::Gen};
#
# #[cfg(feature = "futures03")] {
async fn async_one() -> i32 { 1 }
async fn async_two() -> i32 { 2 }

let gen = Gen::new(|co| async move {
    let one = async_one().await;
    co.yield_(one).await;
    let two = async_two().await;
    co.yield_(two).await;
});
let stream = block_on_stream(gen);
let items: Vec<_> = stream.collect();
assert_eq!(items, [1, 2]);
# }
```

Async generators also provide a `async_resume` method for lower-level control. (This
works even without the `futures03` feature.)

```rust
# use genawaiter::{GeneratorState, rc::Gen};
# use std::task::Poll;
#
# async fn x() {
# let mut gen = Gen::new(|co| async move {
#     co.yield_(10).await;
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

#![cfg_attr(feature = "nightly", feature(async_await, async_closure))]
#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(missing_docs, clippy::cargo, clippy::pedantic)]
#![cfg_attr(feature = "strict", deny(warnings))]

pub use ops::{Coroutine, Generator, GeneratorState};

mod core;
mod ext;
mod ops;
pub mod rc;
pub mod stack;
pub mod sync;
#[cfg(test)]
mod testing;
mod waker;

#[cfg(feature = "proc_macro")]
use proc_macro_hack::proc_macro_hack;

///
#[cfg(feature = "proc_macro")]
#[proc_macro_hack]
pub use genawaiter_proc_macro::sync_producer;

///
#[cfg(feature = "proc_macro")]
#[proc_macro_hack]
pub use genawaiter_proc_macro::rc_producer;

///
#[cfg(feature = "proc_macro")]
#[proc_macro_hack]
pub use genawaiter_proc_macro::stack_producer;

/// This macro is used to replace the keyword yield to
/// avoid using nightly features when using any of the three
/// `proc_macro_attributes` for easy generator definition.
///
/// # Example
/// ```
/// use genawaiter::{stack::producer_fn, yield_};
///
/// #[producer_fn(u8)]
/// async fn odds() {
///     for n in (1..).step_by(2).take_while(|n| *n < 10) {
///         yield_!(n)
///     }
/// }
/// ```
#[cfg(feature = "proc_macro")]
#[macro_export]
macro_rules! yield_ {
    ($val:tt) => {
        compile_error!("forgot to use attribute")
    };
}
