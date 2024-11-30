use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;

#[pin_project(project = FlattenProj)]
pub enum Flatten<Fut>
where
    Fut: Future,
    Fut::Output: Future,
{
    First {
        #[pin]
        fut: Fut,
    },
    Last {
        #[pin]
        fut: Fut::Output,
    },
}

impl<Fut> Flatten<Fut>
where
    Fut: Future,
    Fut::Output: Future,
{
    pub fn new(fut: Fut) -> Self {
        Self::First { fut }
    }
}

impl<Fut, O> Future for Flatten<Fut>
where
    Fut: Future,
    Fut::Output: Future<Output = O>,
{
    type Output = O;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();
            match this {
                FlattenProj::First { fut } => match fut.poll(cx) {
                    Poll::Ready(fut) => {
                        self.set(Self::Last { fut });
                    }
                    Poll::Pending => return Poll::Pending,
                },
                FlattenProj::Last { fut } => match fut.poll(cx) {
                    Poll::Ready(o) => return Poll::Ready(o),
                    Poll::Pending => return Poll::Pending,
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::futures::FuturesExt;

    #[tokio::test]
    async fn test_flatten() {
        let fut = async { async { 42 } };
        let fut = fut.flatten();
        assert_eq!(fut.await, 42);
    }
}
