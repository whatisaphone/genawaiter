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
        let gen = unsafe_generator!(simple_producer);

        assert_eq!(resume!(gen), GeneratorState::Yielded(10));
        assert_eq!(resume!(gen), GeneratorState::Complete("done"));
    }

    #[test]
    fn unsafety() {
        async fn shenanigans(co: Co<'_, i32>) -> Co<'_, i32> {
            co
        }

        let gen = unsafe_generator!(shenanigans);
        let co = match resume!(gen) {
            GeneratorState::Yielded(_) => panic!(),
            GeneratorState::Complete(co) => co,
        };
        // As long as this is possible, this method of creating a generator is `unsafe`,
        // because the `Co` points at dropped memory.
        let _ = co.yield_(10);
    }
}
