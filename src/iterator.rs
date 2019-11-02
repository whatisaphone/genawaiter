use crate::{safe_rc::Generator, state::GeneratorState};
use std::future::Future;

impl<Y, F: Future<Output = ()>> IntoIterator for Generator<Y, F> {
    type Item = Y;
    type IntoIter = IntoIter<Y, F>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { generator: self }
    }
}

pub struct IntoIter<Y, F: Future<Output = ()>> {
    generator: Generator<Y, F>,
}

impl<Y, F: Future<Output = ()>> Iterator for IntoIter<Y, F> {
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
    use crate::{safe_rc::Generator, Co};
    use std::iter::IntoIterator;

    async fn produce(c: Co<i32>) {
        c.yield_(10).await;
        c.yield_(20).await;
    }

    #[test]
    fn into_iter() {
        let gen = Generator::new(produce);
        let items: Vec<_> = gen.into_iter().collect();
        assert_eq!(items, [10, 20]);
    }

    #[test]
    fn for_loop() {
        let mut sum = 0;
        for x in Generator::new(produce) {
            sum += x;
        }
        assert_eq!(sum, 30);
    }
}
