use crate::{ops::GeneratorState, stack::Gen};
use futures_core::{
    task::{Context, Poll},
    Stream,
};
use std::{future::Future, pin::Pin};

impl<'s, Y, F: Future<Output = ()>> Stream for Gen<'s, Y, (), F> {
    type Item = Y;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let fut = self.async_resume();
        pin_mut!(fut);
        match fut.poll(cx) {
            Poll::Ready(GeneratorState::Yielded(x)) => Poll::Ready(Some(x)),
            Poll::Ready(GeneratorState::Complete(())) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        stack::{let_gen_using, Co},
        testing::SlowFuture,
    };
    use futures_executor::block_on_stream;
    use futures_util::stream;

    #[test]
    fn blocking() {
        async fn produce(mut co: Co<'_, i32>) {
            co.yield_(10).await;
            co.yield_(20).await;
        }

        let_gen_using!(gen, produce);
        let stream = stream::iter(gen);
        let items: Vec<_> = block_on_stream(stream).collect();
        assert_eq!(items, [10, 20]);
    }

    #[test]
    fn non_blocking() {
        async fn produce(mut co: Co<'_, i32>) {
            SlowFuture::new().await;
            co.yield_(10).await;
            SlowFuture::new().await;
            co.yield_(20).await;
        }

        let_gen_using!(gen, produce);
        let items: Vec<_> = block_on_stream(gen).collect();
        assert_eq!(items, [10, 20]);
    }
}
