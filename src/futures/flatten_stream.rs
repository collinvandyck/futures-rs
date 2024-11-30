use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::Stream;
use pin_project::pin_project;

#[pin_project(project=Proj)]
pub enum FlattenStream<Fut>
where
    Fut: Future,
    Fut::Output: Stream,
{
    Fut {
        #[pin]
        fut: Fut,
    },
    StreamFut {
        #[pin]
        s: Fut::Output,
    },
}

impl<Fut> FlattenStream<Fut>
where
    Fut: Future,
    Fut::Output: Stream,
{
    pub fn new(fut: Fut) -> Self {
        Self::Fut { fut }
    }
}

impl<Fut, S> Stream for FlattenStream<Fut>
where
    Fut: Future<Output = S>,
    S: Stream,
{
    type Item = S::Item;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let this = self.as_mut().project();
            match this {
                Proj::Fut { fut } => match fut.poll(cx) {
                    Poll::Ready(s) => {
                        self.set(Self::StreamFut { s });
                    }
                    Poll::Pending => return Poll::Pending,
                },
                Proj::StreamFut { s } => return s.poll_next(cx),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use futures_util::{StreamExt, stream};

    use super::*;
    use crate::futures::FuturesExt;

    #[tokio::test]
    async fn test_flatten_stream() {
        let items = vec![1, 2, 3];
        let fut = async { stream::iter(items) };
        let mut s = fut.flatten_stream().collect::<Vec<_>>().await;
        assert_eq!(s, vec![1, 2, 3]);
    }
}
