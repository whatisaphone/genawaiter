extern crate alloc;

use crate::{core, core::Next};
use ::core::cell::Cell;
use alloc::rc::Rc;

pub struct Airlock<Y, R>(Rc<Cell<Next<Y, R>>>);

impl<Y, R> Default for Airlock<Y, R> {
    fn default() -> Self {
        Self(Rc::new(Cell::new(Next::Empty)))
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
        // Safety: `Rc` is `!Send + !Sync`, and control does not leave this function
        // while the reference is taken, so concurrent access is not possible. The value
        // is not modified, so no shared references elsewhere can be invalidated.
        let inner = unsafe { &*self.0.as_ptr() };
        inner.without_values()
    }

    fn replace(&self, next: Next<Y, R>) -> Next<Y, R> {
        self.0.replace(next)
    }
}

/// This object lets you yield values from the generator by calling the `yield_`
/// method.
///
/// "Co" can stand for either _controller_ or _coroutine_, depending on how
/// theoretical you are feeling.
///
/// [_See the module-level docs for examples._](.)
pub type Co<Y, R = ()> = core::Co<Airlock<Y, R>>;
