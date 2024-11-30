use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;

#[pin_project(project=Proj)]
pub enum Fuse<Fut> {
    First {
        #[pin]
        fut: Fut,
    },
    Done,
}

impl<Fut> Fuse<Fut> {
    pub fn new(fut: Fut) -> Self {
        Self::First { fut }
    }
}

impl<Fut> Future for Fuse<Fut>
where
    Fut: Future,
{
    type Output = Fut::Output;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        match this {
            Proj::First { fut } => match fut.poll(cx) {
                Poll::Ready(v) => {
                    self.set(Self::Done);
                    Poll::Ready(v)
                }
                Poll::Pending => Poll::Pending,
            },
            Proj::Done => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::futures::FuturesExt;

    #[tokio::test]
    async fn test_fuse() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let mut fut = crate::futures::FuturesExt::fuse(async move {
            let _ = rx.recv().await;
            42
        });
        let waker = futures::task::noop_waker_ref();
        let mut cx = Context::from_waker(waker);

        tokio::pin!(fut);
        let mut fut = Pin::new(&mut fut);

        assert_eq!(fut.as_mut().poll(&mut cx), Poll::Pending);
        tx.send(()).await.unwrap();
        assert_eq!(fut.as_mut().poll(&mut cx), Poll::Ready(42));
        assert_eq!(fut.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(fut.as_mut().poll(&mut cx), Poll::Pending);
    }
}
