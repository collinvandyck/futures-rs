use core::panic;
use std::{
    borrow::BorrowMut,
    future::Future,
    ops::DerefMut,
    pin::{Pin, pin},
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use pin_project::pin_project;

#[pin_project]
pub struct Shared<Fut: Future<Output: Clone>> {
    #[pin]
    inner: Inner<Fut>,
}

#[pin_project(project = InnerProj)]
enum Inner<Fut: Future<Output: Clone>> {
    Initial {
        #[pin]
        state: Arc<Mutex<State<Fut>>>,
    },
    Done,
}

pub struct State<Fut: Future> {
    fut: Pin<Box<Fut>>,
    val: Option<Fut::Output>,
}

impl<Fut: Future<Output: Clone>> Future for Shared<Fut> {
    type Output = Fut::Output;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        let mut inner = this.inner.project();
        match inner {
            InnerProj::Done => panic!("already polled"),
            InnerProj::Initial { state } => {
                let mut state = state.lock().unwrap();
                let State { fut, val } = &mut *state;
                if let Some(val) = val {
                    let val = val.clone();
                    inner = InnerProj::Done;
                    return Poll::Ready(val);
                }
                match Pin::new(fut).poll(cx) {
                    Poll::Ready(v) => {
                        val.replace(v.clone());
                        return Poll::Ready(v);
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }
        }
    }
}

impl<Fut: Future<Output: Clone>> Shared<Fut> {
    pub fn new(fut: Fut) -> Self {
        let fut = Box::pin(fut);
        let state = State { fut, val: None };
        let inner = Inner::Initial {
            state: Arc::new(Mutex::new(state)),
        };
        Self { inner }
    }
}

impl<Fut: Future<Output: Clone>> Clone for Shared<Fut> {
    fn clone(&self) -> Self {
        let inner = self.inner.clone();
        Self { inner }
    }
}

impl<Fut: Future<Output: Clone>> Clone for Inner<Fut> {
    fn clone(&self) -> Self {
        match self {
            Inner::Initial { state } => Self::Initial { state: state.clone() },
            Inner::Done => Inner::Done,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::futures::FuturesExt;

    #[tokio::test]
    async fn test_shared() {
        let f1 = crate::futures::FuturesExt::shared(async { 42 });
        let f2 = f1.clone();
        let f3 = f2.clone();
        assert_eq!(f1.await, 42);
        assert_eq!(f2.await, 42);
        assert_eq!(f3.await, 42);
    }

    #[tokio::test]
    async fn test_shared_threads() {
        let c1 = Arc::new(Mutex::new(0));
        let c2 = c1.clone();
        let f1 = crate::futures::FuturesExt::shared(async move {
            *c2.lock().unwrap() += 1;
            42
        });
        let f2 = f1.clone();
        let jh = std::thread::spawn(move || {
            let f = futures::executor::block_on(f2);
            assert_eq!(f, 42);
        });
        assert_eq!(f1.await, 42);
        jh.join().unwrap();
        assert_eq!(*c1.lock().unwrap(), 1);
    }
}
