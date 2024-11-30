use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;

#[pin_project(project=IntoStreamProj)]
pub enum IntoStream<Fut> {
    Active {
        #[pin]
        fut: Fut,
    },
    Inactive,
}

impl<Fut> IntoStream<Fut> {
    pub fn new(fut: Fut) -> Self {
        Self::Active { fut }
    }
}

impl<Fut> futures_util::Stream for IntoStream<Fut>
where
    Fut: Future,
{
    type Item = Fut::Output;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.as_mut().project();
        match this {
            IntoStreamProj::Active { fut } => match fut.poll(cx) {
                Poll::Ready(v) => {
                    self.set(Self::Inactive);
                    Poll::Ready(Some(v))
                }
                Poll::Pending => Poll::Pending,
            },
            IntoStreamProj::Inactive => Poll::Ready(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use futures_util::StreamExt;

    use super::*;
    use crate::futures::FuturesExt;

    #[tokio::test]
    async fn test_into_stream() {
        let v = async { 42 }.into_stream().collect::<Vec<_>>().await;
        assert_eq!(v, vec![42]);
    }
}
