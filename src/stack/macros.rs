/**
Creates a generator on the stack.

This macro is deprecated. Use [`let_gen!`] or [`let_gen_using!`] instead.

[`let_gen!`]: stack/macro.let_gen.html
[`let_gen_using!`]: stack/macro.let_gen_using.html
*/
#[macro_export]
#[deprecated = "Use `let_gen_using!()` instead."]
macro_rules! generator_mut {
    ($name:ident, $producer:expr $(,)?) => {
        $crate::stack::let_gen_using!($name, $producer);
    };
}

/**
Creates a generator on the stack unsafely.

This macro is deprecated. Use [`let_gen!`] or [`let_gen_using!`] instead.

[`let_gen!`]: stack/macro.let_gen.html
[`let_gen_using!`]: stack/macro.let_gen_using.html
*/
#[macro_export]
#[deprecated = "Use `let_gen_using!()` instead."]
macro_rules! unsafe_create_generator {
    ($name:ident, $producer:expr $(,)?) => {
        let mut generator_state = $crate::stack::Shelf::new();
        #[allow(unused_mut)]
        let mut $name =
            unsafe { $crate::stack::Gen::new(&mut generator_state, $producer) };
    };
}

#[cfg(test)]
mod tests {
    use crate::{
        ops::GeneratorState,
        stack::{Co, Gen, Shelf},
    };

    /// This test proves that `Gen::new` is actually unsafe.
    #[test]
    #[ignore = "compile-only test"]
    fn unsafety() {
        async fn shenanigans(co: Co<'_, i32>) -> Co<'_, i32> {
            co
        }

        let mut shelf = Shelf::new();
        let mut gen = unsafe { Gen::new(&mut shelf, shenanigans) };

        // Get the `co` out of the generator (don't try this at home).
        let mut escaped_co = match gen.resume(()) {
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
