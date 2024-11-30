use std::future::Future;

mod map;
mod map_into;

trait FuturesExt: Future {
    fn map<F, U>(self, f: F) -> map::Map<Self, F>
    where
        F: FnOnce(Self::Output) -> U,
        Self: Sized,
    {
        map::Map::new(self, f)
    }

    fn map_into<U>(self) -> map_into::MapInto<Self, U>
    where
        Self::Output: Into<U>,
        Self: Sized,
    {
        map_into::MapInto::new(self)
    }
}

impl<T: ?Sized> FuturesExt for T where T: Future {}
