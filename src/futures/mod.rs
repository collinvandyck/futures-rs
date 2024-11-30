use std::future::Future;

use either::Either;
use flatten::Flatten;
use flatten_stream::FlattenStream;
use futures_util::Stream;
use into_stream::IntoStream;
use map::Map;
use map_into::MapInto;

mod either;
mod flatten;
mod flatten_stream;
mod into_stream;
mod map;
mod map_into;
mod then;

trait FuturesExt: Future {
    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Output) -> U,
        Self: Sized,
    {
        Map::new(self, f)
    }

    fn map_into<U>(self) -> MapInto<Self, U>
    where
        Self::Output: Into<U>,
        Self: Sized,
    {
        MapInto::new(self)
    }

    fn then<Fut, F>(self, f: F) -> then::Then<Self, F, Fut>
    where
        F: FnOnce(Self::Output) -> Fut,
        Self: Sized,
    {
        then::Then::new(self, f)
    }

    fn left_future<B>(self) -> Either<Self, B>
    where
        B: Future<Output = Self::Output>,
        Self: Sized,
    {
        Either::Left { fut: self }
    }

    fn right_future<A>(self) -> Either<A, Self>
    where
        A: Future<Output = Self::Output>,
        Self: Sized,
    {
        Either::Right { fut: self }
    }

    fn into_stream(self) -> IntoStream<Self>
    where
        Self: Sized,
    {
        IntoStream::new(self)
    }

    fn flatten(self) -> Flatten<Self>
    where
        Self: Sized,
        Self::Output: Future,
    {
        Flatten::new(self)
    }

    fn flatten_stream(self) -> FlattenStream<Self>
    where
        Self: Sized,
        Self::Output: Stream,
    {
        FlattenStream::new(self)
    }
}

impl<T: ?Sized> FuturesExt for T where T: Future {}
