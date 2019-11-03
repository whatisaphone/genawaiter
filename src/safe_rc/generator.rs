use crate::{
    generator::Generator,
    safe_rc::{
        engine::{advance, Airlock},
        Co,
    },
    GeneratorState,
};
use std::{cell::RefCell, future::Future, pin::Pin, rc::Rc};

pub struct Gen<Y, F: Future> {
    airlock: Airlock<Y>,
    future: Pin<Box<F>>,
}

impl<Y, F: Future> Gen<Y, F> {
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

impl<Y, F: Future> Generator for Gen<Y, F> {
    type Yield = Y;
    type Return = F::Output;

    fn resume(mut self: Pin<&mut Self>) -> GeneratorState<Self::Yield, Self::Return> {
        let this: &mut Self = &mut *self;
        advance(this.future.as_mut(), &this.airlock)
    }
}
