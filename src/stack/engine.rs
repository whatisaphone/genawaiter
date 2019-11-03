use crate::{waker, GeneratorState};
use std::{
    cell::UnsafeCell,
    future::Future,
    pin::Pin,
    ptr,
    task::{Context, Poll},
};

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

pub struct Co<'y, Y> {
    pub(crate) airlock: &'y Airlock<Y>,
}

impl<'y, Y> Co<'y, Y> {
    pub fn yield_(&self, value: Y) -> impl Future<Output = ()> + '_ {
        unsafe {
            *self.airlock.get() = Some(value);
        }
        Barrier {
            airlock: &self.airlock,
        }
    }
}

pub struct Barrier<'y, Y> {
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
