/**
Creates a generator without allocating.

Because of Rust's macro hygiene limitations, you must assign a name immediately:

```ignore
unsafe_create_generator!(my_name, ...)
// Think of this as:
let my_name = unsafe_create_generator!(...)
```

The created variable has type `Pin<&mut Gen<Y, R, impl Future>>`. `Y` is the type
yielded from the generator. `R` is the type of the resume argument. `Future::Output` is
the type returned upon completion of the generator.

The generator's state is stored on the stack of the current function. The state is
pinned in place, so it cannot escape the scope of the function:

```compile_fail
# use genawaiter::{stack::Co, unsafe_create_generator, Generator};
# use std::pin::Pin;
#
# async fn odd_numbers_less_than_ten(co: Co<'_, i32>) {
#     for n in (1..).step_by(2).take_while(|&n| n < 10) { co.yield_(n).await; }
# }
#
fn create_generator() -> Pin<&'static mut impl Generator<Yield = i32>> {
    unsafe_create_generator!(gen, odd_numbers_less_than_ten);
    // error[E0515]: cannot return value referencing local variable `generator_state`
    gen
}
```

However, you _can_ pass the generator to other functions, because moving the generator
does not move its state:

```
# use genawaiter::{stack::Co, unsafe_create_generator, Generator};
# use std::pin::Pin;
#
# async fn odd_numbers_less_than_ten(co: Co<'_, i32>) {
#     for n in (1..).step_by(2).take_while(|&n| n < 10) { co.yield_(n).await; }
# }
#
fn consume_generator(gen: Pin<&mut impl Generator<Yield = i32>>) {
    gen.resume();
}

unsafe_create_generator!(gen, odd_numbers_less_than_ten);
consume_generator(gen);
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
