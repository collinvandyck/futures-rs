use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;

#[pin_project]
pub struct Inspect<Fut, F> {
    #[pin]
    fut: Fut,
    f: Option<F>,
}

impl<Fut, F> Inspect<Fut, F> {
    pub fn new(fut: Fut, f: F) -> Self {
        Self { fut, f: Some(f) }
    }
}

impl<Fut, F> Future for Inspect<Fut, F>
where
    Fut: Future,
    F: FnOnce(&Fut::Output),
{
    type Output = Fut::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        match this.fut.poll(cx) {
            Poll::Ready(o) => {
                let f = this.f.take().expect("no fn set");
                (f)(&o);
                Poll::Ready(o)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::futures::FuturesExt;

    #[tokio::test]
    async fn test_inspect() {
        let mut res = 0;
        let fut = async { 42 };
        let fut = fut.inspect(|num| res = *num);
        assert_eq!(fut.await, 42);
        assert_eq!(res, 42);
    }
}
