use crate::{ops::GeneratorState, stack::generator::Gen};
use std::{future::Future, pin::Pin, ptr};

impl<'g, Y, F: Future<Output = ()>> IntoIterator for Pin<&'g mut Gen<Y, F>> {
    type Item = Y;
    type IntoIter = IntoIter<'g, Y, F>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { generator: self }
    }
}

pub struct IntoIter<'g, Y, F: Future<Output = ()>> {
    generator: Pin<&'g mut Gen<Y, F>>,
}

impl<'g, Y, F: Future<Output = ()>> Iterator for IntoIter<'g, Y, F> {
    type Item = Y;

    fn next(&mut self) -> Option<Self::Item> {
        let generator = unsafe { ptr::read(&self.generator) };
        match generator.__macro_internal_resume() {
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
        let gen = unsafe_generator!(produce);
        let items: Vec<_> = gen.into_iter().collect();
        assert_eq!(items, [10, 20]);
    }

    #[test]
    fn for_loop() {
        let mut sum = 0;
        for x in unsafe_generator!(produce) {
            sum += x;
        }
        assert_eq!(sum, 30);
    }
}
