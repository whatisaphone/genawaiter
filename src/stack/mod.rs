/*!
This module implements a generator which doesn't allocate.

You can create a generator with the [`generator_mut!`] macro:

```rust
# use genawaiter::{generator_mut, stack::Co};
#
async fn producer(co: Co<'_, i32>) { /* ... */ }

generator_mut!(my_generator, producer);
```

This will create a variable named `my_generator` in the current scope, with type `&mut
Gen<...>`.

The macro is a shortcut for creating both a generator and its backing state (called a
[`Shelf`](struct.Shelf.html)). If you (or your IDE) dislike macros, you can also do the
bookkeeping by hand by using [`Gen::new`](struct.Gen.html#method.new), though note that
this requires you to trade away safety.

# Examples

## Using `Iterator`

Generators implement `Iterator`, so you can use them in a for loop:

```rust
use genawaiter::{generator_mut, stack::Co};

async fn odd_numbers_less_than_ten(co: Co<'_, i32>) {
    let mut n = 1;
    while n < 10 {
        co.yield_(n).await;
        n += 2;
    }
}

generator_mut!(gen, odd_numbers_less_than_ten);
# let mut test = Vec::new();
for num in gen {
    println!("{}", num);
    # test.push(num);
}
# assert_eq!(test, [1, 3, 5, 7, 9]);
```

## Collecting into a `Vec`

```rust
# use genawaiter::{generator_mut, stack::Co, GeneratorState};
#
# async fn odd_numbers_less_than_ten(co: Co<'_, i32>) {
#     for n in (1..).step_by(2).take_while(|&n| n < 10) { co.yield_(n).await; }
# }
#
generator_mut!(gen, odd_numbers_less_than_ten);
let xs: Vec<_> = gen.into_iter().collect();
assert_eq!(xs, [1, 3, 5, 7, 9]);
```

## Using `resume()`

```rust
# use genawaiter::{generator_mut, stack::Co, GeneratorState};
#
# async fn odd_numbers_less_than_ten(co: Co<'_, i32>) {
#     for n in (1..).step_by(2).take_while(|&n| n < 10) { co.yield_(n).await; }
# }
#
generator_mut!(gen, odd_numbers_less_than_ten);
assert_eq!(gen.resume(), GeneratorState::Yielded(1));
assert_eq!(gen.resume(), GeneratorState::Yielded(3));
assert_eq!(gen.resume(), GeneratorState::Yielded(5));
assert_eq!(gen.resume(), GeneratorState::Yielded(7));
assert_eq!(gen.resume(), GeneratorState::Yielded(9));
assert_eq!(gen.resume(), GeneratorState::Complete(()));
```

## Using an async closure (nightly Rust only)

```ignore
# use genawaiter::{generator_mut, stack::Co, GeneratorState};
#
generator_mut!(gen, async move |co| {
    co.yield_(10).await;
    co.yield_(20).await;
});
assert_eq!(gen.resume(), GeneratorState::Yielded(10));
assert_eq!(gen.resume(), GeneratorState::Yielded(20));
assert_eq!(gen.resume(), GeneratorState::Complete(()));
```

## Using an async <del>closure</del> faux·sure (works on stable Rust)

```
# use genawaiter::{generator_mut, stack::Co, GeneratorState};
#
generator_mut!(gen, |co| async move {
    co.yield_(10).await;
    co.yield_(20).await;
});
assert_eq!(gen.resume(), GeneratorState::Yielded(10));
assert_eq!(gen.resume(), GeneratorState::Yielded(20));
assert_eq!(gen.resume(), GeneratorState::Complete(()));
```

## Passing ordinary arguments

This is just ordinary Rust, nothing special.

```rust
# use genawaiter::{generator_mut, stack::Co, GeneratorState};
#
async fn multiples_of(num: i32, co: Co<'_, i32>) {
    let mut cur = num;
    loop {
        co.yield_(cur).await;
        cur += num;
    }
}

generator_mut!(gen, |co| multiples_of(10, co));
assert_eq!(gen.resume(), GeneratorState::Yielded(10));
assert_eq!(gen.resume(), GeneratorState::Yielded(20));
assert_eq!(gen.resume(), GeneratorState::Yielded(30));
```

## Passing resume arguments

You can pass values into the generator.

Note that the first resume argument will be lost. This is because at the time the first
value is sent, there is no future being awaited inside the generator, so there is no
place the value could go where the generator could observe it.

```rust
# use genawaiter::{generator_mut, stack::Co, GeneratorState};
#
async fn check_numbers(co: Co<'_, (), i32>) {
    let num = co.yield_(()).await;
    assert_eq!(num, 1);

    let num = co.yield_(()).await;
    assert_eq!(num, 2);
}

generator_mut!(gen, check_numbers);
gen.resume_with(0);
gen.resume_with(1);
gen.resume_with(2);
```

## Returning a completion value

You can return a completion value with a different type than the values that are
yielded.

```rust
# use genawaiter::{generator_mut, stack::Co, GeneratorState};
#
async fn numbers_then_string(co: Co<'_, i32>) -> &'static str {
    co.yield_(10).await;
    co.yield_(20).await;
    "done!"
}

generator_mut!(gen, numbers_then_string);
assert_eq!(gen.resume(), GeneratorState::Yielded(10));
assert_eq!(gen.resume(), GeneratorState::Yielded(20));
assert_eq!(gen.resume(), GeneratorState::Complete("done!"));
```
*/

pub use engine::Co;
pub use generator::{Gen, Shelf};

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
    use crate::{stack::Co, testing::DummyFuture, GeneratorState};
    use std::{
        cell::RefCell,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
    };

    async fn simple_producer(c: Co<'_, i32>) -> &'static str {
        c.yield_(10).await;
        "done"
    }

    #[test]
    fn function() {
        generator_mut!(gen, simple_producer);
        assert_eq!(gen.resume(), GeneratorState::Yielded(10));
        assert_eq!(gen.resume(), GeneratorState::Complete("done"));
    }

    #[test]
    fn simple_closure() {
        async fn gen(i: i32, co: Co<'_, i32>) -> &'static str {
            co.yield_(i * 2).await;
            "done"
        }

        generator_mut!(gen, |co| gen(5, co));
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
        generator_mut!(gen, |co| gen(&resumes, co));
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

        generator_mut!(gen, wrong);
        gen.resume();
    }

    #[test]
    #[should_panic(expected = "Co::yield_")]
    fn multiple_yield_helpful_message() {
        async fn wrong(co: Co<'_, i32>) {
            let _ = co.yield_(10);
            let _ = co.yield_(20);
        }

        generator_mut!(gen, wrong);
        gen.resume();
    }

    #[test]
    #[should_panic = "should have been dropped by now"]
    fn escaped_co_helpful_message() {
        async fn shenanigans(co: Co<'_, i32>) -> Co<'_, i32> {
            co
        }

        generator_mut!(gen, shenanigans);
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
            generator_mut!(gen, |co| {
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
    use genawaiter::{generator_mut, stack::Co};

    async fn producer(co: Co<'static, i32>) {}

    generator_mut!(gen, producer);
    ```
    */
    fn co_is_not_static() {}

    /**
    This test is exactly the same as above, but doesn't trigger the failure.

    ```
    use genawaiter::{generator_mut, stack::Co};

    async fn producer(co: Co<'_, i32>) {}

    generator_mut!(gen, producer);
    ```
    */
    fn co_is_not_static_baseline() {}
}
