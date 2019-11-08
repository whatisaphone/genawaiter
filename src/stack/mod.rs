/*!
This module implements a generator which doesn't require allocation.

You can create a generator with the [`unsafe_create_generator!`] macro:

```rust
# use genawaiter::{stack::Co, unsafe_create_generator};
#
async fn producer(co: Co<'_, i32>) { /* ... */ }

unsafe_create_generator!(gen, producer);
```

This is safe as long as you don't do anything silly with the `Co` object. (See below for
the fine print. If you cannot abide the `unsafe` keyword, use an [allocating
generator](../rc) instead.)

The macro is a shortcut for creating both a generator and its backing state (called a
`Shelf`, to avoid confusion with the `GeneratorState` enum). If you (or your IDE)
dislike macros, you can also do the bookkeeping by hand:

```rust
# use genawaiter::stack::{Co, Gen, Shelf};
#
# async fn producer(co: Co<'_, i32>) { /* ... */ }
#
let mut shelf = Shelf::new();
let gen = unsafe { Gen::new(&mut shelf, producer) };
```

See the crate-level docs for a guide on how to use the generator after it's been
created.

# Safety

Do not let the `Co` object escape the scope of the generator. By time the generator
completes, the `Co` object should already have been dropped. If this invariant is not
upheld, memory unsafety will result.

Afaik, the Rust compiler [is not flexible enough][hrtb-thread] to let you express this
invariant in the type system, but I would love to be proven wrong!

[hrtb-thread]: https://users.rust-lang.org/t/hrtb-on-multiple-generics/34255

# Examples

## Using `Iterator`

Generators implement `Iterator`, so you can use them in a for loop:

```rust
# use genawaiter::{stack::Co, unsafe_create_generator, GeneratorState};
#
# async fn odd_numbers_less_than_ten(co: Co<'_, i32>) {
#     for n in (1..).step_by(2).take_while(|&n| n < 10) { co.yield_(n).await; }
# }
#
unsafe_create_generator!(gen, odd_numbers_less_than_ten);
for n in gen {
    println!("{}", n);
}
```

## Collecting into a `Vec`

```rust
# use genawaiter::{stack::Co, unsafe_create_generator, GeneratorState};
#
# async fn odd_numbers_less_than_ten(co: Co<'_, i32>) {
#     for n in (1..).step_by(2).take_while(|&n| n < 10) { co.yield_(n).await; }
# }
#
unsafe_create_generator!(gen, odd_numbers_less_than_ten);
let xs: Vec<_> = gen.into_iter().collect();
assert_eq!(xs, [1, 3, 5, 7, 9]);
```

## Using `resume()`

```rust
# use genawaiter::{stack::Co, unsafe_create_generator, GeneratorState};
#
# async fn odd_numbers_less_than_ten(co: Co<'_, i32>) {
#     for n in (1..).step_by(2).take_while(|&n| n < 10) { co.yield_(n).await; }
# }
#
unsafe_create_generator!(gen, odd_numbers_less_than_ten);
assert_eq!(gen.resume(), GeneratorState::Yielded(1));
assert_eq!(gen.resume(), GeneratorState::Yielded(3));
assert_eq!(gen.resume(), GeneratorState::Yielded(5));
assert_eq!(gen.resume(), GeneratorState::Yielded(7));
assert_eq!(gen.resume(), GeneratorState::Yielded(9));
assert_eq!(gen.resume(), GeneratorState::Complete(()));
```

## Using an async closure (nightly only)

```ignore
# use genawaiter::{stack::Co, unsafe_create_generator, GeneratorState};
#
unsafe_create_generator!(gen, async move |co| {
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
# use genawaiter::{stack::Co, unsafe_create_generator, GeneratorState};
#
async fn multiples_of(num: i32, co: Co<'_, i32>) {
    let mut cur = num;
    loop {
        co.yield_(cur).await;
        cur += num;
    }
}

unsafe_create_generator!(gen, |co| multiples_of(10, co));
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
# use genawaiter::{stack::Co, GeneratorState, unsafe_create_generator};
#
async fn check_numbers(co: Co<'_, (), i32>) {
    let num = co.yield_(()).await;
    assert_eq!(num, 1);

    let num = co.yield_(()).await;
    assert_eq!(num, 2);
}

unsafe_create_generator!(gen, check_numbers);
gen.resume_with(0);
gen.resume_with(1);
gen.resume_with(2);
```

## Returning a completion value

You can return a completion value with a different type than the values that are
yielded.

```rust
# use genawaiter::{stack::Co, unsafe_create_generator, GeneratorState};
#
async fn numbers_then_string(co: Co<'_, i32>) -> &'static str {
    co.yield_(10).await;
    co.yield_(20).await;
    "done!"
}

unsafe_create_generator!(gen, numbers_then_string);
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

#[cfg(feature = "nightly")]
#[cfg(test)]
mod nightly_tests;

#[cfg(test)]
mod tests {
    use crate::{stack::Co, testing::DummyFuture, GeneratorState};
    use std::cell::RefCell;

    async fn simple_producer(c: Co<'_, i32>) -> &'static str {
        c.yield_(10).await;
        "done"
    }

    #[test]
    fn function() {
        unsafe_create_generator!(gen, simple_producer);
        assert_eq!(gen.resume(), GeneratorState::Yielded(10));
        assert_eq!(gen.resume(), GeneratorState::Complete("done"));
    }

    #[test]
    fn simple_closure() {
        async fn gen(i: i32, co: Co<'_, i32>) -> &'static str {
            co.yield_(i * 2).await;
            "done"
        }

        unsafe_create_generator!(gen, |co| gen(5, co));
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
        unsafe_create_generator!(gen, |co| gen(&resumes, co));
        assert_eq!(*resumes.borrow(), &[] as &[&str]);

        assert_eq!(gen.resume_with("ignored"), GeneratorState::Yielded(10));
        assert_eq!(*resumes.borrow(), &[] as &[&str]);

        assert_eq!(gen.resume_with("abc"), GeneratorState::Yielded(20));
        assert_eq!(*resumes.borrow(), &["abc"]);

        assert_eq!(gen.resume_with("def"), GeneratorState::Complete(()));
        assert_eq!(*resumes.borrow(), &["abc", "def"]);
    }

    #[test]
    #[should_panic(expected = "Co::yield_")]
    fn forbidden_await_helpful_message() {
        async fn wrong(_: Co<'_, i32>) {
            DummyFuture.await;
        }

        unsafe_create_generator!(gen, wrong);
        gen.resume();
    }

    #[test]
    #[should_panic(expected = "Co::yield_")]
    fn multiple_yield_helpful_message() {
        async fn wrong(co: Co<'_, i32>) {
            let _ = co.yield_(10);
            let _ = co.yield_(20);
        }

        unsafe_create_generator!(gen, wrong);
        gen.resume();
    }

    /// This test proves that `unsafe_create_generator` is actually unsafe.
    #[test]
    #[ignore = "compile-only test"]
    fn unsafety() {
        async fn shenanigans(co: Co<'_, i32>) -> Co<'_, i32> {
            co
        }

        unsafe_create_generator!(gen, shenanigans);

        // Get the `co` out of the generator (don't try this at home).
        let escaped_co = match gen.resume() {
            GeneratorState::Yielded(_) => panic!(),
            GeneratorState::Complete(co) => co,
        };
        // Drop the generator. This drops the airlock (inside the state), but `co` still
        // holds a reference to the airlock.
        drop(gen);
        // Now we're able to use an invalidated reference.
        let _ = escaped_co.yield_(10);
    }
}

#[allow(dead_code)]
mod doctests {
    /**
    Make sure `co` cannot escape to the `'static` lifetime.

    ```compile_fail
    use genawaiter::{stack::Co, unsafe_create_generator};

    async fn producer(co: Co<'static, i32>) {}

    unsafe_create_generator!(gen, producer);
    ```
    */
    fn co_is_not_static() {}

    /**
    This test is exactly the same as above, but doesn't trigger the failure.

    ```
    use genawaiter::{stack::Co, unsafe_create_generator};

    async fn producer(co: Co<'_, i32>) {}

    unsafe_create_generator!(gen, producer);
    ```
    */
    fn co_is_not_static_baseline() {}
}
