use std::{
    borrow::BorrowMut,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;

use super::*;

#[pin_project(project=MapProj)]
pub enum Map<Fut, F> {
    Incomplete {
        #[pin]
        fut: Fut,
        f: Option<F>,
    },
    Complete,
}

impl<Fut, F> Map<Fut, F> {
    pub fn new(fut: Fut, f: F) -> Self {
        Self::Incomplete { fut, f: Some(f) }
    }
}

impl<Fut, F, U> Future for Map<Fut, F>
where
    Fut: Future,
    F: FnOnce(Fut::Output) -> U,
{
    type Output = U;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();
        match this {
            MapProj::Incomplete { fut, f } => match fut.poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(val) => {
                    let f = f.take().expect("no mapping fn");
                    self.set(Self::Complete);
                    Poll::Ready((f)(val))
                }
            },
            MapProj::Complete => {
                panic!("future already complete");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_map() {
        assert_eq!(43, async { 42 }.map(|x| x + 1).await);
        assert_eq!("43", async { 42 }.map(|x| x + 1).map(|i| i.to_string()).await);
    }
}
