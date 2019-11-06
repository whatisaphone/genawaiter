use crate::{waker, GeneratorState};
use std::{
    cell::RefCell,
    future::Future,
    mem,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

/// This type holds the value that is pending being returned from the generator.
pub type Airlock<Y, R> = Rc<RefCell<Next<Y, R>>>;

pub enum Next<Y, R> {
    Empty,
    Yield(Y),
    Resume(R),
}

pub fn advance<Y, R, F: Future>(
    future: Pin<&mut F>,
    airlock: &Airlock<Y, R>,
    resume_arg: R,
) -> GeneratorState<Y, F::Output> {
    *airlock.borrow_mut() = Next::Resume(resume_arg);

    let waker = waker::create();
    let mut cx = Context::from_waker(&waker);

    match future.poll(&mut cx) {
        Poll::Pending => {
            let opened_airlock = &mut *airlock.borrow_mut();
            let value = mem::replace(opened_airlock, Next::Empty);
            match value {
                Next::Empty => unreachable!(),
                Next::Yield(y) => GeneratorState::Yielded(y),
                Next::Resume(_) => {
                    #[cfg(debug_assertions)]
                    panic!(
                        "A generator was awaited without first yielding a value. \
                         Inside a generator, do not await any futures other than the \
                         one returned by `Co::yield_`."
                    );

                    #[cfg(not(debug_assertions))]
                    panic!("invalid await");
                }
            }
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
/// _See the module-level docs for examples._
pub struct Co<Y, R = ()> {
    pub(crate) airlock: Airlock<Y, R>,
}

impl<Y, R> Co<Y, R> {
    /// Yields a value from the generator.
    ///
    /// The caller should immediately `await` the result of this function.
    ///
    /// _See the module-level docs for examples._
    pub fn yield_(&self, value: Y) -> impl Future<Output = R> + '_ {
        let mut opened_airlock = self.airlock.borrow_mut();

        #[cfg(debug_assertions)]
        {
            if let Next::Yield(_) = *opened_airlock {
                panic!(
                    "Multiple values were yielded without an intervening await. Make \
                     sure to immediately await the result of `Co::yield_`."
                );
            }
        }

        *opened_airlock = Next::Yield(value);
        Barrier {
            airlock: &self.airlock,
        }
    }
}

struct Barrier<'y, Y, R> {
    airlock: &'y Airlock<Y, R>,
}

impl<'y, Y, R> Future for Barrier<'y, Y, R> {
    type Output = R;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut opened_airlock = self.airlock.borrow_mut();
        match *opened_airlock {
            Next::Empty => unreachable!(),
            Next::Yield(_) => Poll::Pending,
            Next::Resume(_) => {
                let value = mem::replace(&mut *opened_airlock, Next::Empty);
                match value {
                    Next::Empty => unreachable!(),
                    Next::Yield(_) => unreachable!(),
                    Next::Resume(arg) => Poll::Ready(arg),
                }
            }
        }
    }
}
