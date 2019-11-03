pub use engine::Co;
pub use generator::Gen;

#[macro_use]
mod macros;

mod engine;
mod generator;
mod iterator;

#[cfg(test)]
mod tests {
    use crate::{stack::Co, GeneratorState};

    async fn simple_producer(c: Co<'_, i32>) -> &'static str {
        c.yield_(10).await;
        "done"
    }

    #[test]
    fn function() {
        let mut gen = unsafe_generator!(simple_producer);
        assert_eq!(gen.as_mut().resume(), GeneratorState::Yielded(10));
        assert_eq!(gen.as_mut().resume(), GeneratorState::Complete("done"));
    }

    #[test]
    fn simple_closure() {
        async fn gen(i: i32, co: Co<'_, i32>) -> &'static str {
            co.yield_(i * 2).await;
            "done"
        }

        let mut gen = unsafe_generator!(|co| gen(5, co));
        assert_eq!(gen.as_mut().resume(), GeneratorState::Yielded(10));
        assert_eq!(gen.as_mut().resume(), GeneratorState::Complete("done"));
    }

    /// This test proves that `unsafe_generator` is actually unsafe.
    #[test]
    #[ignore = "compile-only test"]
    fn unsafety() {
        async fn shenanigans(co: Co<'_, i32>) -> Co<'_, i32> {
            co
        }

        fn co_escape() -> Co<'static, i32> {
            let mut gen = unsafe_generator!(shenanigans);

            // Returning `co` from this function violates memory safety.
            match gen.as_mut().resume() {
                GeneratorState::Yielded(_) => panic!(),
                GeneratorState::Complete(co) => co,
            }
        }

        let co = co_escape();
        // `co` points to data which was on the stack of `co_escape()` and has been
        // dropped.
        let _ = co.yield_(10);
    }
}
