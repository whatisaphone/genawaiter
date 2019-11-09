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

impl<'r, 's, Y, F: Future<Output = ()>> IntoIterator for &'r mut Gen<'s, Y, (), F> {
    type Item = Y;
    type IntoIter = MutIntoIter<'r, 's, Y, F>;

    fn into_iter(self) -> Self::IntoIter {
        MutIntoIter { generator: self }
    }
}

pub struct MutIntoIter<'r, 's, Y, F: Future<Output = ()>> {
    generator: &'r mut Gen<'s, Y, (), F>,
}

impl<'r, 's, Y, F: Future<Output = ()>> Iterator for MutIntoIter<'r, 's, Y, F> {
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
    use crate::stack::{Co, Gen, Shelf};
    use std::iter::IntoIterator;

    async fn produce(c: Co<'_, i32>) {
        c.yield_(10).await;
        c.yield_(20).await;
    }

    #[test]
    fn generator_mut_into_iter() {
        generator_mut!(gen, produce);

        let items: Vec<_> = gen.into_iter().collect();
        assert_eq!(items, [10, 20]);
    }

    #[test]
    fn generator_mut_for_loop() {
        generator_mut!(gen, produce);

        let mut sum = 0;
        for x in gen {
            sum += x;
        }
        assert_eq!(sum, 30);
    }

    #[test]
    fn shelf_generator_into_iter() {
        let mut shelf = Shelf::new();
        let gen = unsafe { Gen::new(&mut shelf, produce) };

        let items: Vec<_> = gen.into_iter().collect();
        assert_eq!(items, [10, 20]);
    }

    #[test]
    fn shelf_generator_for_loop() {
        let mut shelf = Shelf::new();
        let gen = unsafe { Gen::new(&mut shelf, produce) };

        let mut sum = 0;
        for x in gen {
            sum += x;
        }
        assert_eq!(sum, 30);
    }
}
