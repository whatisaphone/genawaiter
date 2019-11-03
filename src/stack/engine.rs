use crate::{waker, GeneratorState};
use std::{
    cell::UnsafeCell,
    future::Future,
    pin::Pin,
    ptr,
    task::{Context, Poll},
};

/// This type holds the value that is pending being returned from the generator.
pub type Airlock<Y> = UnsafeCell<Option<Y>>;

pub fn advance<Y, R>(
    future: Pin<&mut impl Future<Output = R>>,
    airlock: &Airlock<Y>,
) -> GeneratorState<Y, R> {
    let waker = waker::create();
    let mut cx = Context::from_waker(&waker);

    match future.poll(&mut cx) {
        Poll::Pending => {
            let value = unsafe { ptr::replace(airlock.get(), None) };
            GeneratorState::Yielded(value.unwrap())
        }
        Poll::Ready(value) => GeneratorState::Complete(value),
    }
}

/// This object lets you yield values from the generator by calling the `yield_`
/// method.
///
/// "Co" can stand for either _controller_ or _coroutine_, depending on how
/// theoretical you are feeling.
///
/// _See the module-level docs for more details._
pub struct Co<'y, Y> {
    pub(crate) airlock: &'y Airlock<Y>,
}

impl<'y, Y> Co<'y, Y> {
    /// Yields a value from the generator.
    ///
    /// The caller should immediately `await` the result of this function.
    ///
    /// _See the module-level docs for more details._
    pub fn yield_(&self, value: Y) -> impl Future<Output = ()> + '_ {
        unsafe {
            *self.airlock.get() = Some(value);
        }
        Barrier {
            airlock: &self.airlock,
        }
    }
}

struct Barrier<'y, Y> {
    airlock: &'y Airlock<Y>,
}

impl<'y, Y> Future for Barrier<'y, Y> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let airlock = unsafe { self.airlock.get().as_ref().unwrap() };
        if airlock.is_none() {
            // If there is no value in the airlock, resume the generator so it produces
            // one.
            Poll::Ready(())
        } else {
            // If there is a value, pause the generator so we can yield the value to the
            // caller.
            Poll::Pending
        }
    }
}
