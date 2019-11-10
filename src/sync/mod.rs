/*!
This module implements a generator which can be shared between threads.

You can create a generator with [`Gen::new`](struct.Gen.html#method.new). Pass it a
function that bootstraps the generator:

```rust
# use genawaiter::sync::{Co, Gen};
#
async fn producer(co: Co<i32>) { /* ... */ }

let mut generator = Gen::new(producer);
```

# Remarks

Do not let the `Co` object escape the scope of the generator. Once the starting future
returns `Poll::Ready`, the `Co` object should already have been dropped. If this
invariant is not upheld, the result will be memory-safe but otherwise left unspecified.

# Examples

## Using `Iterator`

Generators implement `Iterator`, so you can use them in a for loop:

```rust
use genawaiter::sync::{Co, Gen};

async fn odd_numbers_less_than_ten(co: Co<i32>) {
    let mut n = 1;
    while n < 10 {
        co.yield_(n).await;
        n += 2;
    }
}

# let mut test = Vec::new();
for num in Gen::new(odd_numbers_less_than_ten) {
    println!("{}", num);
    # test.push(num);
}
# assert_eq!(test, [1, 3, 5, 7, 9]);
```

## Collecting into a `Vec`

```rust
# use genawaiter::{sync::{Co, Gen}, GeneratorState};
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
# use genawaiter::{sync::{Co, Gen}, GeneratorState};
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

## Using an async closure (nightly Rust only)

```ignore
# use genawaiter::{sync::{Co, Gen}, GeneratorState};
#
let mut gen = Gen::new(async move |co| {
    co.yield_(10).await;
    co.yield_(20).await;
});
assert_eq!(gen.resume(), GeneratorState::Yielded(10));
assert_eq!(gen.resume(), GeneratorState::Yielded(20));
assert_eq!(gen.resume(), GeneratorState::Complete(()));
```

## Using an async <del>closure</del> faux·sure (works on stable Rust)

```
# use genawaiter::{sync::{Co, Gen}, GeneratorState};
#
let mut gen = Gen::new(|co| async move {
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
# use genawaiter::{sync::{Co, Gen}, GeneratorState};
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

## Passing resume arguments

You can pass values into the generator.

Note that the first resume argument will be lost. This is because at the time the first
value is sent, there is no future being awaited inside the generator, so there is no
place the value could go where the generator could observe it.

```rust
# use genawaiter::{sync::{Co, Gen}, GeneratorState};
#
async fn check_numbers(co: Co<(), i32>) {
    let num = co.yield_(()).await;
    assert_eq!(num, 1);

    let num = co.yield_(()).await;
    assert_eq!(num, 2);
}

let mut gen = Gen::new(check_numbers);
gen.resume_with(0);
gen.resume_with(1);
gen.resume_with(2);
```

## Returning a completion value

You can return a completion value with a different type than the values that are
yielded.

```rust
# use genawaiter::{sync::{Co, Gen}, GeneratorState};
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

pub use box_dyn::GenBoxDyn;
pub use engine::Co;
pub use generator::Gen;

mod box_dyn;
mod engine;
mod generator;
mod iterator;

#[cfg(feature = "nightly")]
#[cfg(test)]
mod nightly_tests;

#[cfg(test)]
mod tests {
    use crate::{
        sync::{Co, Gen},
        testing::DummyFuture,
        GeneratorState,
    };
    use std::{
        cell::{Cell, RefCell},
        future::Future,
    };

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
    #[should_panic(expected = "Co::yield_")]
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
