/*!
This module implements a generator which doesn't allocate.

You can create a basic generator with [`let_gen!`] and [`yield_!`].

[`let_gen!`]: macro.let_gen.html

```rust
# #[cfg(feature = "proc_macro")]
# fn feature_gate() {
# use genawaiter::{stack::let_gen, yield_};
#
let_gen!(my_generator, {
    yield_!(10);
});
# my_generator.resume();
# }
```

If you don't like macros, you can use the low-level API directly, though note that this
requires you to trade away safety.

```rust
# use genawaiter::stack::{Co, Gen, Shelf};
#
async fn my_producer(co: Co<'_, u8>) {
    co.yield_(10).await;
}
let mut shelf = Shelf::new();
let mut my_generator = unsafe { Gen::new(&mut shelf, my_producer) };
# my_generator.resume();
```

# Examples

## Using `Iterator`

Generators implement `Iterator`, so you can use them in a for loop:

```rust
# #[cfg(feature = "proc_macro")]
# fn feature_gate() {
use genawaiter::{stack::let_gen, yield_};

let_gen!(odds_under_ten, {
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
# use genawaiter::{stack::let_gen, yield_};
#
# let_gen!(odds_under_ten, {
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
# use genawaiter::{stack::let_gen, yield_, GeneratorState};
#
let two = 2;
let_gen!(multiply, {
    yield_!(10 * two);
});
assert_eq!(multiply.resume(), GeneratorState::Yielded(20));
# }
```

## Using `resume()`

```rust
# #[cfg(feature = "proc_macro")]
# fn feature_gate() {
# use genawaiter::{stack::let_gen, yield_, GeneratorState};
#
# let_gen!(odds_under_ten, {
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
# use genawaiter::{stack::let_gen, yield_};
#
let_gen!(check_numbers, {
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
# use genawaiter::{stack::let_gen, yield_, GeneratorState};
#
let_gen!(numbers_then_string, {
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
# use genawaiter::{stack::{let_gen_using, producer_fn}, yield_, GeneratorState};
#
#[producer_fn(u8)]
async fn produce() {
    yield_!(10);
}

let_gen_using!(gen, produce);
assert_eq!(gen.resume(), GeneratorState::Yielded(10));
# }
```

## Using the low-level API

You can define an `async fn` directly, instead of relying on the `gen!` or `producer!`
macros.

```rust
use genawaiter::stack::{let_gen_using, Co};

async fn producer(co: Co<'_, i32>) {
    let mut n = 1;
    while n < 10 {
        co.yield_(n).await;
        n += 2;
    }
}

let_gen_using!(odds_under_ten, producer);
let result: Vec<_> = odds_under_ten.into_iter().collect();
assert_eq!(result, [1, 3, 5, 7, 9]);
```

## Using the low-level API with an async closure (nightly Rust only)

```ignore
# use genawaiter::{stack::let_gen_using, GeneratorState};
#
let_gen_using!(gen, async move |co| {
    co.yield_(10).await;
    co.yield_(20).await;
});
assert_eq!(gen.resume(), GeneratorState::Yielded(10));
assert_eq!(gen.resume(), GeneratorState::Yielded(20));
assert_eq!(gen.resume(), GeneratorState::Complete(()));
```

## Using the low-level API with an async <del>closure</del> faux·sure (for stable Rust)

```
# use genawaiter::{stack::let_gen_using, GeneratorState};
#
let_gen_using!(gen, |co| async move {
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
# use genawaiter::{stack::{let_gen_using, Co}, GeneratorState};
#
async fn multiples_of(num: i32, co: Co<'_, i32>) {
    let mut cur = num;
    loop {
        co.yield_(cur).await;
        cur += num;
    }
}

let_gen_using!(gen, |co| multiples_of(10, co));
assert_eq!(gen.resume(), GeneratorState::Yielded(10));
assert_eq!(gen.resume(), GeneratorState::Yielded(20));
assert_eq!(gen.resume(), GeneratorState::Yielded(30));
```
*/

pub use crate::stack::{
    engine::Co,
    generator::{Gen, Shelf},
};

/// Creates a generator.
///
/// The first argument is the name of the resulting variable.
///
/// ```ignore
/// let_gen!(my_generator, { /* ... */ });
/// // Think of this as the spiritual equivalent of:
/// let mut my_generator = Gen::new(/* ... */);
/// ```
///
/// The second argument is the body of the generator. It should contain one or
/// more calls to the [`yield_!`] macro.
///
/// This macro is a shortcut for creating both a generator and its backing state
/// (called a [`Shelf`](struct.Shelf.html)). If you (or your IDE) dislike
/// macros, you can also do the bookkeeping by hand by using
/// [`Gen::new`](struct.Gen.html#method.new), though note that this requires you
/// to trade away safety.
///
/// # Examples
///
/// [_See the module-level docs for examples._](.)
#[cfg(feature = "proc_macro")]
pub use genawaiter_macro::stack_let_gen as let_gen;

/// Creates a generator using a producer defined elsewhere.
///
/// The first argument is the name of the resulting variable.
///
/// ```ignore
/// let_gen!(my_generator, { /* ... */ });
/// // Think of this as the spiritual equivalent of:
/// let mut my_generator = Gen::new(/* ... */);
/// ```
///
/// The second line is the producer that will be used. It can be one of these
/// two things:
///
/// 1.  The result of [`stack_producer!`] or [`stack_producer_fn!`]
///
///     [`stack_producer_fn!`]: attr.producer_fn.html
///
/// 2.  A function with this type:
///
///     ```ignore
///     async fn producer(co: Co<'_, Yield, Resume>) -> Completion { /* ... */ }
///     // which is equivalent to:
///     fn producer(co: Co<'_, Yield, Resume>) -> impl Future<Output = Completion> { /* ... */ }
///     ```
///
/// This macro is a shortcut for creating both a generator and its backing state
/// (called a [`Shelf`](struct.Shelf.html)). If you (or your IDE) dislike
/// macros, you can also do the bookkeeping by hand by using
/// [`Gen::new`](struct.Gen.html#method.new), though note that this requires you
/// to trade away safety.
///
/// # Examples
///
/// [_See the module-level docs for examples._](.)
pub use genawaiter_macro::stack_let_gen_using as let_gen_using;

/// Turns a function into a producer, which can then be used to create a
/// generator.
///
/// The body of the function should contain one or more [`yield_!`] expressions.
///
/// # Examples
///
/// [_See the module-level docs for examples._](.)
#[cfg(feature = "proc_macro")]
pub use genawaiter_proc_macro::stack_producer_fn as producer_fn;

#[macro_use]
mod macros;
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
        stack::{let_gen_using, Co},
        testing::DummyFuture,
        GeneratorState,
    };
    use std::{
        cell::RefCell,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
    };

    async fn simple_producer(co: Co<'_, i32>) -> &'static str {
        co.yield_(10).await;
        "done"
    }

    #[test]
    fn function() {
        let_gen_using!(gen, simple_producer);
        assert_eq!(gen.resume(), GeneratorState::Yielded(10));
        assert_eq!(gen.resume(), GeneratorState::Complete("done"));
    }

    #[test]
    fn simple_closure() {
        async fn gen(i: i32, co: Co<'_, i32>) -> &'static str {
            co.yield_(i * 2).await;
            "done"
        }

        let_gen_using!(gen, |co| gen(5, co));
        assert_eq!(gen.resume(), GeneratorState::Yielded(10));
        assert_eq!(gen.resume(), GeneratorState::Complete("done"));
    }

    #[test]
    fn resume_args() {
        async fn gen(resumes: &RefCell<Vec<&str>>, co: Co<'_, i32, &'static str>) {
            let resume_arg = co.yield_(10).await;
            resumes.borrow_mut().push(resume_arg);
            let resume_arg = co.yield_(20).await;
            resumes.borrow_mut().push(resume_arg);
        }

        let resumes = RefCell::new(Vec::new());
        let_gen_using!(gen, |co| gen(&resumes, co));
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
        async fn wrong(_: Co<'_, i32>) {
            DummyFuture.await;
        }

        let_gen_using!(gen, wrong);
        gen.resume();
    }

    #[test]
    #[should_panic(expected = "Co::yield_")]
    fn multiple_yield_helpful_message() {
        async fn wrong(co: Co<'_, i32>) {
            let _ = co.yield_(10);
            let _ = co.yield_(20);
        }

        let_gen_using!(gen, wrong);
        gen.resume();
    }

    #[test]
    #[should_panic = "should have been dropped by now"]
    fn escaped_co_helpful_message() {
        async fn shenanigans(co: Co<'_, i32>) -> Co<'_, i32> {
            co
        }

        let_gen_using!(gen, shenanigans);
        let escaped_co = match gen.resume() {
            GeneratorState::Yielded(_) => panic!(),
            GeneratorState::Complete(co) => co,
        };
        let _ = escaped_co.yield_(10);
    }

    /// Test the unsafe `Gen::drop` implementation.
    #[test]
    fn test_gen_drop() {
        struct SetFlagOnDrop(Arc<AtomicBool>);

        impl Drop for SetFlagOnDrop {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let flag = Arc::new(AtomicBool::new(false));
        {
            let capture_the_flag = flag.clone();
            let_gen_using!(gen, |co| {
                async move {
                    let _set_on_drop = SetFlagOnDrop(capture_the_flag);
                    co.yield_(10).await;
                    // We will never make it this far.
                    unreachable!();
                }
            });
            assert_eq!(gen.resume(), GeneratorState::Yielded(10));
            // `gen` is only a reference to the generator, and dropping a reference has
            // no effect. The underlying generator is hidden behind macro hygiene and so
            // cannot be dropped early.
            #[allow(clippy::drop_ref)]
            drop(gen);
            assert_eq!(flag.load(Ordering::SeqCst), false);
        }
        // After the block above ends, the generator goes out of scope and is dropped,
        // which drops the incomplete future, which drops `_set_on_drop`, which sets the
        // flag.
        assert_eq!(flag.load(Ordering::SeqCst), true);
    }
}

#[allow(dead_code)]
mod doctests {
    /**
    Make sure `co` cannot escape to the `'static` lifetime.

    ```compile_fail
    use genawaiter::stack::{let_gen_using, Co};

    async fn producer(co: Co<'static, i32>) {}

    let_gen_using!(gen, producer);
    ```
    */
    fn co_is_not_static() {}

    /**
    This test is exactly the same as above, but doesn't trigger the failure.

    ```
    use genawaiter::stack::{let_gen_using, Co};

    async fn producer(co: Co<'_, i32>) {}

    let_gen_using!(gen, producer);
    ```
    */
    fn co_is_not_static_baseline() {}
}

#[allow(dead_code)]
#[cfg(feature = "proc_macro")]
mod doc_compile_fail {
    /**
    Make sure `co` cannot be used as argument by user.

    ```compile_fail
    use genawaiter::{stack::{producer_fn, Co}, yield_};

    #[producer_fn(u8)]
    async fn odds(co: Co<'_, u8>) {
        yield_!(10);
    }
    ```
    */
    fn with_args_compile_fail() {}

    /**
    This test is exactly the same as above, except it passes.

    ```rust
    use genawaiter::{stack::{producer_fn, Co}, yield_};

    #[producer_fn(u8)]
    async fn odds() {
        yield_!(10);
    }
    ```
    */
    fn with_args_compile_fail_baseline() {}
}
