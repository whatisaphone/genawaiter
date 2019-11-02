use crate::{waker, GeneratorState};
use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

type Airlock<Y> = Rc<RefCell<Option<Y>>>;

pub fn advance<Y, R>(
    future: Pin<&mut impl Future<Output = R>>,
    airlock: &Airlock<Y>,
) -> GeneratorState<Y, R> {
    let waker = waker::create();
    let mut cx = Context::from_waker(&waker);

    match future.poll(&mut cx) {
        Poll::Pending => {
            let value = airlock.borrow_mut().take().unwrap();
            GeneratorState::Yielded(value)
        }
        Poll::Ready(value) => GeneratorState::Complete(value),
    }
}

pub struct Co<Y> {
    pub(crate) airlock: Airlock<Y>,
}

impl<Y> Co<Y> {
    pub fn yield_(&self, value: Y) -> impl Future<Output = ()> + '_ {
        *self.airlock.borrow_mut() = Some(value);
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
        if self.airlock.borrow().is_none() {
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
