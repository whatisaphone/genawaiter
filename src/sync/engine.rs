use crate::{core, core::Next};
use std::{
    mem,
    sync::{Arc, Mutex},
};

pub struct Airlock<Y, R>(Arc<Mutex<Next<Y, R>>>);

impl<Y, R> Default for Airlock<Y, R> {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(Next::Empty)))
    }
}

impl<Y, R> Clone for Airlock<Y, R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Y, R> core::Airlock for Airlock<Y, R> {
    type Yield = Y;
    type Resume = R;

    fn peek(&self) -> Next<(), ()> {
        self.0.lock().unwrap().without_values()
    }

    fn replace(&self, next: Next<Y, R>) -> Next<Y, R> {
        mem::replace(&mut self.0.lock().unwrap(), next)
    }
}

/// This object lets you yield values from the generator by calling the `yield_`
/// method.
///
/// "Co" can stand for either _controller_ or _coroutine_, depending on how
/// theoretical you are feeling.
///
/// _See the module-level docs for examples._
pub type Co<Y, R = ()> = core::Co<Airlock<Y, R>>;
