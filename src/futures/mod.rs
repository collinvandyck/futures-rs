use std::future::Future;

mod map;

trait FuturesExt: Future {
    fn map<F, U>(self, f: F) -> map::Map<Self, F>
    where
        F: FnOnce(Self::Output) -> U,
        Self: Sized,
    {
        map::Map::new(self, f)
    }
}

impl<T: ?Sized> FuturesExt for T where T: Future {}
