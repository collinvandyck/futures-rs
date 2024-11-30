use core::panic;
use std::{
    future::Future,
    marker::{PhantomData, PhantomPinned},
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;

#[pin_project(project=MapIntoProj)]
pub enum MapInto<Fut, U> {
    Active {
        #[pin]
        fut: Fut,
        u: PhantomData<U>,
    },
    Complete,
}

impl<Fut, U> MapInto<Fut, U>
where
    Fut: Future,
    Fut::Output: Into<U>,
{
    pub fn new(fut: Fut) -> Self {
        Self::Active { fut, u: PhantomData }
    }
}

impl<Fut, U> Future for MapInto<Fut, U>
where
    Fut: Future,
    U: From<Fut::Output>,
{
    type Output = U;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        match this {
            MapIntoProj::Active { fut, .. } => match fut.poll(cx) {
                Poll::Ready(o) => {
                    self.set(MapInto::Complete);
                    Poll::Ready(o.into())
                }
                Poll::Pending => Poll::Pending,
            },
            MapIntoProj::Complete => {
                panic!("future already polled")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::futures::FuturesExt;

    #[tokio::test]
    async fn test_map_into() {
        let val: String = async { "42" }.map_into().await;
        assert_eq!(val, "42");
    }
}
