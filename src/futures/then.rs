use core::panic;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;

#[pin_project(project=ThenProj)]
pub enum Then<Fut, F, U> {
    Start {
        #[pin]
        fut: Fut,
        f: Option<F>,
    },
    Waiting {
        #[pin]
        fut: U,
    },
}

impl<Fut, F, U> Then<Fut, F, U> {
    pub fn new(fut: Fut, f: F) -> Self {
        Self::Start { fut, f: Some(f) }
    }
}

impl<Fut, F, U> Future for Then<Fut, F, U>
where
    Fut: Future,
    U: Future,
    F: FnOnce(Fut::Output) -> U,
{
    type Output = U::Output;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let this = self.as_mut().project();
            match this {
                ThenProj::Start { fut, f } => match fut.poll(cx) {
                    Poll::Ready(o) => {
                        let f = f.take().expect("not fut fn");
                        let fut = (f)(o);
                        self.set(Self::Waiting { fut });
                    }
                    Poll::Pending => return Poll::Pending,
                },
                ThenProj::Waiting { fut } => return fut.poll(cx),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::futures::FuturesExt;

    #[tokio::test]
    async fn test_then() {
        assert_eq!(async { 42 }.then(move |x| async move { x + 1 }).await, 43);
        assert_eq!(
            async { 42 }
                .then(move |x| async move { x + 1 })
                .then(|x| async move { x.to_string() })
                .await,
            "43"
        );
    }
}
