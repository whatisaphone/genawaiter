/**
Creates a generator without allocating.

Because of Rust's macro hygiene limitations, you must assign a name immediately:

```ignore
unsafe_create_generator!(my_name, ...)
// Think of this as:
let my_name = unsafe_create_generator!(...)
```

The created variable has type `Pin<&mut Gen<Y, impl Future>>`. `Y` is the type yielded
from the generator (`GeneratorState::Yielded`). `Future::Output` is the type returned
from the generator (`GeneratorState::Complete`).

The generator's state is stored on the stack of the current function. The state is
pinned in place, so you cannot return it up the stack:

```compile_fail
# use genawaiter::{stack::{Co, Gen}, unsafe_create_generator};
# use std::{future::Future, pin::Pin};
#
# async fn odd_numbers_less_than_ten(co: Co<'_, i32>) {
#     for n in (1..).step_by(2).take_while(|&n| n < 10) { co.yield_(n).await; }
# }
#
fn create_generator() -> Pin<&'static mut Gen<i32, impl Future<Output = ()>>> {
    unsafe_create_generator!(gen, odd_numbers_less_than_ten);
    gen
}
```

However, you _can_ pass it to other functions, because the pinned generator is distinct
from its state:

```
# use genawaiter::{stack::{Co, Gen}, unsafe_create_generator};
# use std::{future::Future, pin::Pin};
#
# async fn odd_numbers_less_than_ten(co: Co<'_, i32>) {
#     for n in (1..).step_by(2).take_while(|&n| n < 10) { co.yield_(n).await; }
# }
#
fn exhaust_generator(gen: Pin<&mut Gen<i32, impl Future<Output = ()>>>) {
    let _: Vec<_> = gen.into_iter().collect();
}

unsafe_create_generator!(gen, odd_numbers_less_than_ten);
exhaust_generator(gen);
```
*/
#[macro_export]
macro_rules! unsafe_create_generator {
    ($name:ident, $start:expr) => {
        // Pull this into its own function so the lifetimes are not lost.
        unsafe fn mu_as_mut<T>(mu: &mut ::std::mem::MaybeUninit<T>) -> &mut T {
            mu.as_mut_ptr().as_mut().unwrap()
        }

        let mut generator_state = ::std::mem::MaybeUninit::uninit();
        #[allow(unused_mut)]
        let mut $name = unsafe {
            $crate::stack::Gen::__macro_internal_popuate(&mut generator_state, $start);
            ::std::pin::Pin::new_unchecked(mu_as_mut(&mut generator_state))
        };
    };
}

#[allow(dead_code)]
mod doctests {
    /**
    ```compile_fail
    use genawaiter::{stack::Co, unsafe_create_generator, Generator};
    use std::pin::Pin;

    async fn odd_numbers_less_than_ten(co: Co<'_, i32>) {
        for n in (1..).step_by(2).take_while(|&n| n < 10) {
            co.yield_(n).await;
        }
    }

    fn create_generator() -> Pin<&'static mut impl Generator> {
        unsafe_create_generator!(gen, odd_numbers_less_than_ten);
        gen
    }
    ```
    */
    fn generator_cannot_escape() {}

    /**
    This test is exactly the same as above, but doesn't trigger the failure.

    ```
    use genawaiter::{stack::Co, unsafe_create_generator, Generator};
    use std::pin::Pin;

    async fn odd_numbers_less_than_ten(co: Co<'_, i32>) {
        for n in (1..).step_by(2).take_while(|&n| n < 10) {
            co.yield_(n).await;
        }
    }

    fn create_generator() {
        unsafe_create_generator!(gen, odd_numbers_less_than_ten);
        // gen
    }
    ```
    */
    fn generator_cannot_escape_baseline() {}
}
