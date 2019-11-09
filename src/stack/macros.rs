/**
Creates a generator on the stack.

This macro creates a generator with the name you pass in:

```ignore
generator_mut!(my_name, ...)
// Think of this as:
let my_name = generator_mut!(...)
```

The full type of the new variable is `&mut Gen<'_, Y, R, impl Future>`. `Y` is the type
yielded from the generator. `R` is the type of the resume argument. `Future::Output` is
the type returned upon completion of the generator.

The generator's state is stored on the stack of the current function. The state is
pinned in place, so it cannot escape the scope of the function. However, since the
generator is a reference, you can pass it around to other functions:

```
# use genawaiter::{stack::{Co, Gen}, generator_mut, Generator};
# use std::future::Future;
#
# async fn odd_numbers_less_than_ten(co: Co<'_, i32>) {
#     for n in (1..).step_by(2).take_while(|&n| n < 10) { co.yield_(n).await; }
# }
#
fn consume_generator(gen: &mut Gen<'_, i32, (), impl Future>) {
    gen.resume();
}

generator_mut!(gen, odd_numbers_less_than_ten);
consume_generator(gen);
```
*/
#[macro_export]
macro_rules! generator_mut {
    ($name:ident, $start:expr) => {
        // The goal here is to ensure the safety invariants of `Gen::new`, i.e., the
        // lifetime of the `Co` argument (in `$start`) must not outlive
        // `generator_state`.
        //
        // We do this by creating two variables that cannot be named by user-land code -
        // `generator_state`, and `generator`. Because they are declared in the same
        // scope, and cannot be dropped before the end of the scope, they have
        // equivalent lifetimes. The type signature of `Gen::new` ties the lifetime of
        // the `Co` to that of `generator_state`. This gives `co` the same lifetime as
        // the state. Therefore, it will never outlive `generator_state`, and will
        // always be pointing somewhere valid.
        let mut generator_state = $crate::stack::Shelf::new();
        let mut generator =
            unsafe { $crate::stack::Gen::new(&mut generator_state, $start) };
        let $name = &mut generator;
    };
}

/**
Creates a generator on the stack. By-value, but not 100% safe.

⚠ This macro only exists for backwards compatibility. Prefer the safe [`generator_mut!`]
whenever possible.

This macro creates a generator with the name you pass in:

```ignore
unsafe_create_generator!(my_name, ...)
// Think of this as:
let my_name = unsafe_create_generator!(...)
```

The full type of the new variable is `Gen<'_, Y, R, impl Future>`. `Y` is the type
yielded from the generator. `R` is the type of the resume argument. `Future::Output` is
the type returned upon completion of the generator.

Compared to [`generator_mut!`], this macro gives by-value access to the generator
(instead of a reference). In practice, this distinction does not matter at all, and you
pay for it by opening a small window for memory unsafety.

# Safety

This macro has the same safety invariants as
[`Gen::new`](stack/struct.Gen.html#method.new).
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
