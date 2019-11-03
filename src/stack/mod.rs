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

    #[test]
    fn unsafety() {
        async fn shenanigans(co: Co<'_, i32>) -> Co<'_, i32> {
            co
        }

        fn co_escape() -> Co<'static, i32> {
            let mut gen = unsafe_generator!(shenanigans);

            #[allow(clippy::let_and_return)]
            let co = match gen.as_mut().resume() {
                GeneratorState::Yielded(_) => panic!(),
                GeneratorState::Complete(co) => co,
            };

            co
        }

        // As long as this compiles, this method of creating a generator is `unsafe`,
        // because `co` points at dropped memory.
        let co = co_escape();
        let _ = co.yield_(10);
    }
}
