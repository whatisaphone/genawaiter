use crate::{core, core::Next};
use std::{cell::UnsafeCell, ptr};

/// This type holds the value that is pending being returned from the generator.
///
/// # Safety
///
/// This type is `!Sync` (so, single-thread), never exposed to user-land code,
/// and never borrowed across a function call, so safety can be verified locally
/// at each use site.
pub struct Airlock<Y, R>(UnsafeCell<Next<Y, R>>);

impl<Y, R> Default for Airlock<Y, R> {
    fn default() -> Self {
        Self(UnsafeCell::new(Next::Empty))
    }
}

impl<'s, Y, R> core::Airlock for &'s Airlock<Y, R> {
    type Yield = Y;
    type Resume = R;

    fn peek(&self) -> Next<(), ()> {
        // Safety: This follows the safety rules above.
        let inner = unsafe { &*self.0.get() };
        inner.without_values()
    }

    fn replace(
        &self,
        next: Next<Self::Yield, Self::Resume>,
    ) -> Next<Self::Yield, Self::Resume> {
        // Safety: This follows the safety rules above.
        unsafe { ptr::replace(self.0.get(), next) }
    }
}

/// This object lets you yield values from the generator by calling the `yield_`
/// method.
///
/// "Co" can stand for either _controller_ or _coroutine_, depending on how
/// theoretical you are feeling.
///
/// [_See the module-level docs for examples._](.)
pub type Co<'y, Y, R = ()> = core::Co<&'y Airlock<Y, R>>;
