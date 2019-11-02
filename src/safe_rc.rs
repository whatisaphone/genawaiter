use crate::{engine::advance, Co, GeneratorState};
use std::{cell::RefCell, future::Future, pin::Pin, rc::Rc};

pub struct Generator<Y, F: Future> {
    airlock: Rc<RefCell<Option<Y>>>,
    future: Pin<Box<F>>,
}

impl<Y, F: Future> Generator<Y, F> {
    pub fn new(start: impl FnOnce(Co<Y>) -> F) -> Self {
        let airlock = Rc::new(RefCell::new(None));
        let future = {
            let airlock = airlock.clone();
            Box::pin(start(Co { airlock }))
        };
        Self { airlock, future }
    }

    pub fn resume(&mut self) -> GeneratorState<Y, F::Output> {
        advance(self.future.as_mut(), &self.airlock)
    }
}

#[cfg(test)]
mod tests {
    use crate::{safe_rc::Generator, Co, GeneratorState};
    use std::future::Future;

    async fn simple_producer(c: Co<i32>) -> &'static str {
        c.yield_(10).await;
        "done"
    }

    #[test]
    fn function() {
        let mut gen = Generator::new(simple_producer);
        assert_eq!(gen.resume(), GeneratorState::Yielded(10));
        assert_eq!(gen.resume(), GeneratorState::Complete("done"));
    }

    #[test]
    fn simple_closure() {
        async fn gen(i: i32, co: Co<i32>) -> &'static str {
            co.yield_(i * 2).await;
            "done"
        }

        let mut gen = Generator::new(|co| gen(5, co));
        assert_eq!(gen.resume(), GeneratorState::Yielded(10));
        assert_eq!(gen.resume(), GeneratorState::Complete("done"));
    }

    #[test]
    fn async_closure() {
        let mut gen = Generator::new(async move |co| {
            co.yield_(10).await;
            "done"
        });
        assert_eq!(gen.resume(), GeneratorState::Yielded(10));
        assert_eq!(gen.resume(), GeneratorState::Complete("done"));
    }

    #[test]
    fn pinned() {
        #[inline(never)]
        async fn produce(addrs: &mut Vec<*const i32>, co: Co<i32>) -> &'static str {
            use std::cell::Cell;

            let sentinel: Cell<i32> = Cell::new(0x8001);
            let sentinel_ref: &Cell<i32> = &sentinel;

            assert_eq!(sentinel.get(), 0x8001);
            sentinel_ref.set(0x8002);
            assert_eq!(sentinel.get(), 0x8002);
            addrs.push(sentinel.as_ptr());

            co.yield_(10).await;

            assert_eq!(sentinel.get(), 0x8002);
            sentinel_ref.set(0x8003);
            assert_eq!(sentinel.get(), 0x8003);
            addrs.push(sentinel.as_ptr());

            co.yield_(20).await;

            assert_eq!(sentinel.get(), 0x8003);
            sentinel_ref.set(0x8004);
            assert_eq!(sentinel.get(), 0x8004);
            addrs.push(sentinel.as_ptr());

            "done"
        }

        fn create_generator(
            addrs: &mut Vec<*const i32>,
        ) -> Generator<i32, impl Future<Output = &'static str> + '_> {
            let mut gen = Generator::new(move |co| produce(addrs, co));
            assert_eq!(gen.resume(), GeneratorState::Yielded(10));
            gen
        }

        let mut addrs = Vec::new();
        let mut gen = create_generator(&mut addrs);

        assert_eq!(gen.resume(), GeneratorState::Yielded(20));
        assert_eq!(gen.resume(), GeneratorState::Complete("done"));
        drop(gen);

        assert!(addrs.iter().all(|&p| p == addrs[0]));
    }
}
