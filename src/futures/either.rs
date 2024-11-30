use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;

#[pin_project(project=EitherProj)]
pub enum Either<L, R> {
    Left {
        #[pin]
        fut: L,
    },
    Right {
        #[pin]
        fut: R,
    },
}

impl<L, R> Future for Either<L, R>
where
    L: Future<Output = R::Output>,
    R: Future,
{
    type Output = R::Output;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this {
            EitherProj::Left { fut } => fut.poll(cx),
            EitherProj::Right { fut } => fut.poll(cx),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::futures::FuturesExt;

    #[tokio::test]
    async fn test_either() {
        // i am amazed that the type system cobbles together these two types into an Either<L,R>
        let fut = if true {
            async { 42 }.left_future()
        } else {
            async { 43 }.right_future()
        };
        assert_eq!(fut.await, 42);

        let fut = if false {
            async { 42 }.left_future()
        } else {
            async { 43 }.right_future()
        };
        assert_eq!(fut.await, 43);
    }
}
