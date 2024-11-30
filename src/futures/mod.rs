use std::future::Future;

use either::Either;
use map::Map;
use map_into::MapInto;

mod either;
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
}

impl<T: ?Sized> FuturesExt for T where T: Future {}
