use crate::{ops::GeneratorState, stack::generator::Gen};
use std::future::Future;

impl<'s, Y, F: Future<Output = ()>> IntoIterator for Gen<'s, Y, (), F> {
    type Item = Y;
    type IntoIter = IntoIter<'s, Y, F>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { generator: self }
    }
}

pub struct IntoIter<'s, Y, F: Future<Output = ()>> {
    generator: Gen<'s, Y, (), F>,
}

impl<'s, Y, F: Future<Output = ()>> Iterator for IntoIter<'s, Y, F> {
    type Item = Y;

    fn next(&mut self) -> Option<Self::Item> {
        match self.generator.resume() {
            GeneratorState::Yielded(x) => Some(x),
            GeneratorState::Complete(()) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::stack::Co;
    use std::iter::IntoIterator;

    async fn produce(c: Co<'_, i32>) {
        c.yield_(10).await;
        c.yield_(20).await;
    }

    #[test]
    fn into_iter() {
        unsafe_create_generator!(gen, produce);
        let items: Vec<_> = gen.into_iter().collect();
        assert_eq!(items, [10, 20]);
    }

    #[test]
    fn for_loop() {
        let mut sum = 0;
        unsafe_create_generator!(gen, produce);
        for x in gen {
            sum += x;
        }
        assert_eq!(sum, 30);
    }
}
