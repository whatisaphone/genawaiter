/*!
This module implements a generator which stores its state on the heap.

You can create a generator with [`Gen::new`][`rc::Gen::new`]. Pass it a function that
bootstraps the generator.

Values can be yielded from the generator by calling [`Co::yield_`][`rc::Co::yield_`],
and immediately awaiting the future it returns:

```rust
# use genawaiter::rc::Co;
#
# async fn f(co: Co<&str>) {
co.yield_("value").await;
# }
```

You can get values out of the generator in either of two ways:

- Treat it as an iterator. In this case, the future's output must be `()`.
- Call `resume()` until it completes. In this case, the future's output can be anything,
  and it will be returned in the final `GeneratorState::Complete`.

These generators are memory-safe no matter what you do, but some operations are left
unspecified. You can avoid unspecified behavior by not doing silly things. Here is a
non-exhaustive list of silly things:

- Whenever calling `yield_()`, always immediately await its result.
- Do not `await` any futures other than ones returned by `Co::yield_`.
- Do not let the `Co` object escape the scope of the generator. Once the starting future
  returns `Poll::Ready`, the `Co` object should already have been dropped.

# Examples

(See the crate-level docs for the definition of `odd_numbers_less_than_ten`.)

## Using `Iterator`

Generators implement `Iterator`, so you can use them in a for loop:

```rust
# use genawaiter::{rc::{Co, Gen}, GeneratorState};
#
# async fn odd_numbers_less_than_ten(co: Co<i32>) {
#     for n in (1..).step_by(2).take_while(|&n| n < 10) { co.yield_(n).await; }
# }
#
for n in Gen::new(odd_numbers_less_than_ten) {
    println!("{}", n);
}
```

## Collecting into a `Vec`

```rust
# use genawaiter::{rc::{Co, Gen}, GeneratorState};
#
# async fn odd_numbers_less_than_ten(co: Co<i32>) {
#     for n in (1..).step_by(2).take_while(|&n| n < 10) { co.yield_(n).await; }
# }
#
let gen = Gen::new(odd_numbers_less_than_ten);
let xs: Vec<_> = gen.into_iter().collect();
assert_eq!(xs, [1, 3, 5, 7, 9]);
```

## Using `resume()`

```rust
# use genawaiter::{rc::{Co, Gen}, GeneratorState};
#
# async fn odd_numbers_less_than_ten(co: Co<i32>) {
#     for n in (1..).step_by(2).take_while(|&n| n < 10) { co.yield_(n).await; }
# }
#
let mut gen = Gen::new(odd_numbers_less_than_ten);
assert_eq!(gen.resume(), GeneratorState::Yielded(1));
assert_eq!(gen.resume(), GeneratorState::Yielded(3));
assert_eq!(gen.resume(), GeneratorState::Yielded(5));
assert_eq!(gen.resume(), GeneratorState::Yielded(7));
assert_eq!(gen.resume(), GeneratorState::Yielded(9));
assert_eq!(gen.resume(), GeneratorState::Complete(()));
```

## Using an async closure (nightly only)

```compile_fail
# use genawaiter::{rc::{Co, Gen}, GeneratorState};
#
let mut gen = Gen::new(async move |co| {
    co.yield_(10).await;
    co.yield_(20).await;
});
assert_eq!(gen.resume(), GeneratorState::Yielded(10));
assert_eq!(gen.resume(), GeneratorState::Yielded(20));
assert_eq!(gen.resume(), GeneratorState::Complete(()));
```

## Passing arguments

This is just ordinary Rust, nothing special.

```rust
# use genawaiter::{rc::{Co, Gen}, GeneratorState};
#
async fn multiples_of(num: i32, co: Co<i32>) {
    let mut cur = num;
    loop {
        co.yield_(cur).await;
        cur += num;
    }
}

let mut gen = Gen::new(|co| multiples_of(10, co));
assert_eq!(gen.resume(), GeneratorState::Yielded(10));
assert_eq!(gen.resume(), GeneratorState::Yielded(20));
assert_eq!(gen.resume(), GeneratorState::Yielded(30));
```

## Returning a final value

You can return a final value with a different type than the values that are yielded.

```rust
# use genawaiter::{rc::{Co, Gen}, GeneratorState};
#
async fn numbers_then_string(co: Co<i32>) -> &'static str {
    co.yield_(10).await;
    co.yield_(20).await;
    "done!"
}

let mut gen = Gen::new(numbers_then_string);
assert_eq!(gen.resume(), GeneratorState::Yielded(10));
assert_eq!(gen.resume(), GeneratorState::Yielded(20));
assert_eq!(gen.resume(), GeneratorState::Complete("done!"));
```
*/

pub use engine::Co;
pub use generator::Gen;

mod engine;
mod generator;
mod iterator;

#[cfg(feature = "nightly")]
#[cfg(test)]
mod nightly_tests;

#[cfg(test)]
mod tests {
    use crate::{
        rc::{Co, Gen},
        testing::DummyFuture,
        GeneratorState,
    };
    use std::future::Future;

    async fn simple_producer(c: Co<i32>) -> &'static str {
        c.yield_(10).await;
        "done"
    }

    #[test]
    fn function() {
        let mut gen = Gen::new(simple_producer);
        assert_eq!(gen.resume(), GeneratorState::Yielded(10));
        assert_eq!(gen.resume(), GeneratorState::Complete("done"));
    }

    #[test]
    fn simple_closure() {
        async fn gen(i: i32, co: Co<i32>) -> &'static str {
            co.yield_(i * 2).await;
            "done"
        }

        let mut gen = Gen::new(|co| gen(5, co));
        assert_eq!(gen.resume(), GeneratorState::Yielded(10));
        assert_eq!(gen.resume(), GeneratorState::Complete("done"));
    }

    #[test]
    #[should_panic(expected = "Co::yield_")]
    fn forbidden_await_helpful_message() {
        async fn wrong(_: Co<i32>) {
            DummyFuture.await;
        }

        let mut gen = Gen::new(wrong);
        gen.resume();
    }

    /// This tests in a roundabout way that the `Gen` object can be moved. This
    /// should happen without moving the allocations inside so we don't
    /// segfault.
    #[test]
    fn gen_is_movable() {
        #[inline(never)]
        async fn produce(addrs: &mut Vec<*const i32>, co: Co<i32>) -> &'static str {
            use std::cell::Cell;

            let sentinel: Cell<i32> = Cell::new(0x8001);
            // If the future state moved, this reference would become invalid, and
            // hilarity would ensue.
            let sentinel_ref: &Cell<i32> = &sentinel;

            // Test a few times that `sentinel` and `sentinel_ref` point to the same
            // data.

            assert_eq!(sentinel.get(), 0x8001);
            sentinel_ref.set(0x8002);
            assert_eq!(sentinel.get(), 0x8002);
            addrs.push(sentinel.as_ptr());

            co.yield_(10).await;

            assert_eq!(sentinel.get(), 0x8002);
            sentinel_ref.set(0x8003);
            assert_eq!(sentinel.get(), 0x8003);
            addrs.push(sentinel.as_ptr());

            co.yield_(20).await;

            assert_eq!(sentinel.get(), 0x8003);
            sentinel_ref.set(0x8004);
            assert_eq!(sentinel.get(), 0x8004);
            addrs.push(sentinel.as_ptr());

            "done"
        }

        /// Create a generator, resume it once (so `sentinel_ref` gets
        /// initialized), and then move it out of the function.
        fn create_generator(
            addrs: &mut Vec<*const i32>,
        ) -> Gen<i32, impl Future<Output = &'static str> + '_> {
            let mut gen = Gen::new(move |co| produce(addrs, co));
            assert_eq!(gen.resume(), GeneratorState::Yielded(10));
            gen
        }

        let mut addrs = Vec::new();
        let mut gen = create_generator(&mut addrs);

        assert_eq!(gen.resume(), GeneratorState::Yielded(20));
        assert_eq!(gen.resume(), GeneratorState::Complete("done"));
        drop(gen);

        assert!(addrs.iter().all(|&p| p == addrs[0]));
    }
}
