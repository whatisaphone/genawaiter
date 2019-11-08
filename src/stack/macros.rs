/**
Creates a generator without allocating.

Because of Rust's macro hygiene limitations, you must assign a name immediately:

```ignore
unsafe_create_generator!(my_name, ...)
// Think of this as:
let my_name = unsafe_create_generator!(...)
```

The created variable has type `Gen<'_, Y, R, impl Future>`. `Y` is the type yielded from
the generator. `R` is the type of the resume argument. `Future::Output` is the type
returned upon completion of the generator.

The generator's state is stored on the stack of the current function. The state is
pinned in place, so it cannot escape the scope of the function:

```compile_fail
# use genawaiter::{stack::Co, unsafe_create_generator, Generator};
#
# async fn odd_numbers_less_than_ten(co: Co<'_, i32>) {
#     for n in (1..).step_by(2).take_while(|&n| n < 10) { co.yield_(n).await; }
# }
#
fn create_generator() -> impl Generator {
    unsafe_create_generator!(gen, odd_numbers_less_than_ten);
    // error[E0597]: `generator_state` does not live long enough
    gen
}
```

However, you _can_ pass the generator to other functions, because moving the generator
does not move its state:

```
# use genawaiter::{stack::{Co, Gen}, unsafe_create_generator, Generator};
# use std::future::Future;
#
# async fn odd_numbers_less_than_ten(co: Co<'_, i32>) {
#     for n in (1..).step_by(2).take_while(|&n| n < 10) { co.yield_(n).await; }
# }
#
fn consume_generator(mut gen: Gen<'_, i32, (), impl Future>) {
    gen.resume();
}

unsafe_create_generator!(gen, odd_numbers_less_than_ten);
consume_generator(gen);
```
*/
#[macro_export]
macro_rules! unsafe_create_generator {
    ($name:ident, $start:expr) => {
        let mut generator_state = $crate::stack::Shelf::new();
        #[allow(unused_mut)]
        let mut $name =
            unsafe { $crate::stack::Gen::new(&mut generator_state, $start) };
    };
}

#[allow(dead_code)]
mod doctests {
    /**
    ```compile_fail
    use genawaiter::{stack::Co, unsafe_create_generator, Generator};

    async fn producer(co: Co<'_, i32>) {}

    fn create_generator() -> impl Generator {
        unsafe_create_generator!(gen, producer);
        gen
    }
    ```
    */
    fn generator_cannot_escape() {}

    /**
    This test is exactly the same as above, but doesn't trigger the failure.

    ```
    use genawaiter::{stack::Co, unsafe_create_generator, Generator};

    async fn producer(co: Co<'_, i32>) {}

    fn create_generator() {
        unsafe_create_generator!(gen, producer);
        // gen
    }
    ```
    */
    fn generator_cannot_escape_baseline() {}
}
