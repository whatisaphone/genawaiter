/*!
This module implements a generator which stores its state on the heap.

You can create a basic generator with [`gen!`] and [`yield_!`].

[`gen!`]: macro.gen.html

```rust
# #[cfg(feature = "proc_macro")]
# fn feature_gate() {
# use genawaiter::{rc::gen, yield_};
#
let mut my_generator = gen!({
    yield_!(10);
});
# my_generator.resume();
# }
```

If you need to reuse logic between multiple generators, you can define the logic with
[`rc_producer!`] and [`yield_!`], and instantiate generators with [`Gen::new`].

```rust
# #[cfg(feature = "proc_macro")]
# fn feature_gate() {
# use genawaiter::{rc::Gen, rc_producer as producer, yield_};
#
let my_producer = producer!({
    yield_!(10);
});
let mut my_generator = Gen::new(my_producer);
# my_generator.resume();
# }
```

If you don't like macros, you can use the low-level API directly.

```rust
# use genawaiter::rc::{Co, Gen};
#
async fn my_producer(co: Co<u8>) {
    co.yield_(10).await;
}
let mut my_generator = Gen::new(my_producer);
# my_generator.resume();
```

# Examples

## Using `Iterator`

Generators implement `Iterator`, so you can use them in a for loop:

```rust
# #[cfg(feature = "proc_macro")]
# fn feature_gate() {
use genawaiter::{rc::gen, yield_};

let odds_under_ten = gen!({
    let mut n = 1;
    while n < 10 {
        yield_!(n);
        n += 2;
    }
});

# let mut test = Vec::new();
for num in odds_under_ten {
    println!("{}", num);
    # test.push(num);
}
# assert_eq!(test, [1, 3, 5, 7, 9]);
# }
```

## Collecting into a `Vec`

```rust
# #[cfg(feature = "proc_macro")]
# fn feature_gate() {
# use genawaiter::{rc::gen, yield_};
#
# let odds_under_ten = gen!({
#     for n in (1..).step_by(2).take_while(|&n| n < 10) { yield_!(n); }
# });
#
let xs: Vec<_> = odds_under_ten.into_iter().collect();
assert_eq!(xs, [1, 3, 5, 7, 9]);
# }
```

## A generator is a closure

Like any closure, you can capture values from outer scopes.

```rust
# #[cfg(feature = "proc_macro")]
# fn feature_gate() {
# use genawaiter::{rc::gen, yield_, GeneratorState};
#
let two = 2;
let mut multiply = gen!({
    yield_!(10 * two);
});
assert_eq!(multiply.resume(), GeneratorState::Yielded(20));
# }
```

## Using `resume()`

```rust
# #[cfg(feature = "proc_macro")]
# fn feature_gate() {
# use genawaiter::{rc::gen, yield_, GeneratorState};
#
# let mut odds_under_ten = gen!({
#     for n in (1..).step_by(2).take_while(|&n| n < 10) { yield_!(n); }
# });
#
assert_eq!(odds_under_ten.resume(), GeneratorState::Yielded(1));
assert_eq!(odds_under_ten.resume(), GeneratorState::Yielded(3));
assert_eq!(odds_under_ten.resume(), GeneratorState::Yielded(5));
assert_eq!(odds_under_ten.resume(), GeneratorState::Yielded(7));
assert_eq!(odds_under_ten.resume(), GeneratorState::Yielded(9));
assert_eq!(odds_under_ten.resume(), GeneratorState::Complete(()));
# }
```

## Passing resume arguments

You can pass values into the generator.

Note that the first resume argument will be lost. This is because at the time the first
value is sent, there is no future being awaited inside the generator, so there is no
place the value could go where the generator could observe it.

```rust
# #[cfg(feature = "proc_macro")]
# fn feature_gate() {
# use genawaiter::{rc::gen, yield_};
#
let mut check_numbers = gen!({
    let num = yield_!(());
    assert_eq!(num, 1);

    let num = yield_!(());
    assert_eq!(num, 2);
});

check_numbers.resume_with(0);
check_numbers.resume_with(1);
check_numbers.resume_with(2);
# }
```

## Returning a completion value

You can return a completion value with a different type than the values that are
yielded.

```rust
# #[cfg(feature = "proc_macro")]
# fn feature_gate() {
# use genawaiter::{rc::gen, yield_, GeneratorState};
#
let mut numbers_then_string = gen!({
    yield_!(10);
    yield_!(20);
    "done!"
});

assert_eq!(numbers_then_string.resume(), GeneratorState::Yielded(10));
assert_eq!(numbers_then_string.resume(), GeneratorState::Yielded(20));
assert_eq!(numbers_then_string.resume(), GeneratorState::Complete("done!"));
# }
```

## Defining a reusable producer function

```rust
# #[cfg(feature = "proc_macro")]
# fn feature_gate() {
# use genawaiter::{rc::{producer_fn, Gen}, yield_, GeneratorState};
#
#[producer_fn(u8)]
async fn produce() {
    yield_!(10);
}

let mut gen = Gen::new(produce);
assert_eq!(gen.resume(), GeneratorState::Yielded(10));
# }
```

## Defining a reusable producer closure

```rust
# #[cfg(feature = "proc_macro")]
# fn feature_gate() {
# use genawaiter::{rc::Gen, yield_, GeneratorState};
use genawaiter::rc_producer as producer;

let produce = producer!({
    yield_!(10);
});

let mut gen = Gen::new(produce);
assert_eq!(gen.resume(), GeneratorState::Yielded(10));
# }
```

## Using the low-level API

You can define an `async fn` directly, instead of relying on the `gen!` or `producer!`
macros.

```rust
use genawaiter::rc::{Co, Gen};

async fn producer(co: Co<i32>) {
    let mut n = 1;
    while n < 10 {
        co.yield_(n).await;
        n += 2;
    }
}

let odds_under_ten = Gen::new(producer);
let result: Vec<_> = odds_under_ten.into_iter().collect();
assert_eq!(result, [1, 3, 5, 7, 9]);
```

## Using the low-level API with an async closure (nightly Rust only)

```ignore
# use genawaiter::{rc::Gen, GeneratorState};
#
let gen = Gen::new(async move |co| {
    co.yield_(10).await;
    co.yield_(20).await;
});
assert_eq!(gen.resume(), GeneratorState::Yielded(10));
assert_eq!(gen.resume(), GeneratorState::Yielded(20));
assert_eq!(gen.resume(), GeneratorState::Complete(()));
```

## Using the low-level API with an async <del>closure</del> faux·sure (for stable Rust)

```
# use genawaiter::{rc::Gen, GeneratorState};
#
let mut gen = Gen::new(|co| async move {
    co.yield_(10).await;
    co.yield_(20).await;
});
assert_eq!(gen.resume(), GeneratorState::Yielded(10));
assert_eq!(gen.resume(), GeneratorState::Yielded(20));
assert_eq!(gen.resume(), GeneratorState::Complete(()));
```

## Using the low-level API with function arguments

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
*/

pub use crate::rc::{engine::Co, generator::Gen};

/// Creates a generator.
///
/// This macro takes one argument, which is the body of the generator. It should
/// contain one or more calls to the [`yield_!`] macro.
///
/// # Examples
///
/// [_See the module-level docs for examples._](.)
#[cfg(feature = "proc_macro")]
pub use genawaiter_macro::rc_gen as gen;

/// Turns a function into a producer, which can then be used to create a
/// generator.
///
/// The body of the function should contain one or more [`yield_!`] expressions.
///
/// # Examples
///
/// [_See the module-level docs for examples._](.)
#[cfg(feature = "proc_macro")]
pub use genawaiter_proc_macro::rc_producer_fn as producer_fn;

mod engine;
mod generator;
mod iterator;
#[cfg(feature = "futures03")]
mod stream;

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
    use std::{
        cell::{Cell, RefCell},
        future::Future,
    };

    async fn simple_producer(co: Co<i32>) -> &'static str {
        co.yield_(10).await;
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
    fn resume_args() {
        async fn gen(resumes: &RefCell<Vec<&str>>, co: Co<i32, &'static str>) {
            let resume_arg = co.yield_(10).await;
            resumes.borrow_mut().push(resume_arg);
            let resume_arg = co.yield_(20).await;
            resumes.borrow_mut().push(resume_arg);
        }

        let resumes = RefCell::new(Vec::new());
        let mut gen = Gen::new(|co| gen(&resumes, co));
        assert_eq!(*resumes.borrow(), &[] as &[&str]);

        assert_eq!(gen.resume_with("ignored"), GeneratorState::Yielded(10));
        assert_eq!(*resumes.borrow(), &[] as &[&str]);

        assert_eq!(gen.resume_with("abc"), GeneratorState::Yielded(20));
        assert_eq!(*resumes.borrow(), &["abc"]);

        assert_eq!(gen.resume_with("def"), GeneratorState::Complete(()));
        assert_eq!(*resumes.borrow(), &["abc", "def"]);
    }

    #[test]
    #[should_panic(expected = "non-async method")]
    fn forbidden_await_helpful_message() {
        async fn wrong(_: Co<i32>) {
            DummyFuture.await;
        }

        let mut gen = Gen::new(wrong);
        gen.resume();
    }

    #[test]
    #[should_panic(expected = "Co::yield_")]
    fn multiple_yield_helpful_message() {
        async fn wrong(co: Co<i32>) {
            let _ = co.yield_(10);
            let _ = co.yield_(20);
        }

        let mut gen = Gen::new(wrong);
        gen.resume();
    }

    #[test]
    #[should_panic = "should have been dropped by now"]
    fn escaped_co_helpful_message() {
        async fn shenanigans(co: Co<i32>) -> Co<i32> {
            co
        }

        let mut gen = Gen::new(shenanigans);
        let escaped_co = match gen.resume() {
            GeneratorState::Yielded(_) => panic!(),
            GeneratorState::Complete(co) => co,
        };
        let _ = escaped_co.yield_(10);
    }

    /// This tests in a roundabout way that the `Gen` object can be moved. This
    /// should happen without moving the allocations inside so we don't
    /// segfault.
    #[test]
    fn gen_is_movable() {
        #[inline(never)]
        async fn produce(addrs: &mut Vec<*const i32>, co: Co<i32>) -> &'static str {
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
        ) -> Gen<i32, (), impl Future<Output = &'static str> + '_> {
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
