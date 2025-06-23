//! Utilities for working with frunk.

use std::future::Future;

use frunk::{
    from_generic, from_labelled_generic,
    hlist::{HMappable, HZippable},
    into_generic, into_labelled_generic,
    prelude::HList,
    Generic, HCons, HNil, LabelledGeneric,
};

pub mod cons_list;
pub mod futures;

use self::futures::{
    for_each::{AsyncForEach, AsyncLocalForEach, AsyncLocalParForEach, AsyncParForEach},
    hmappable::{AsyncHMappable, AsyncLocalHMappable, AsyncLocalParHMappable, AsyncParHMappable},
    map_to_list::{AsyncLocalMapToList, AsyncLocalParMapToList, AsyncMapToList, AsyncParMapToList},
};

pub use self::cons_list::{ConsList, ConsListT};

/// The Func trait from frunk doesn't take `self` as a parameter to `call` so there isn't an easy way to get context
/// from the surrounding scope. Here we define our own `Poly` wrapper and `Func` trait that does take `self` as a
/// parameter so the caller can include whatever context they need.
pub struct Poly<F>(pub F);

pub trait Func<I> {
    type Output;

    fn call(&mut self, i: I) -> Self::Output;
}

impl<F: Func<I>, I> Func<I> for &mut F {
    type Output = F::Output;

    fn call(&mut self, i: I) -> Self::Output {
        (*self).call(i)
    }
}

impl<F: Func<Head>, Head, Tail: HMappable<Poly<F>>> HMappable<Poly<F>> for HCons<Head, Tail> {
    type Output = HCons<F::Output, Tail::Output>;

    fn map(self, mut mapper: Poly<F>) -> Self::Output {
        let HCons { head, tail } = self;
        HCons {
            head: mapper.0.call(head),
            tail: tail.map(mapper),
        }
    }
}

/// Convenience functions for the caller to map between similarly-shaped types implementing [Generic] without having to
/// explicitly call [from_generic] and [into_generic]
pub trait WithGeneric: Generic {
    fn hmap<U: Generic, F>(self, f: F) -> U
    where
        Self::Repr: HMappable<Poly<F>, Output = U::Repr>;

    fn hmap_async<U: Generic, F: Send>(self, f: F) -> impl Future<Output = U> + Send
    where
        Self: Send,
        Self::Repr: AsyncHMappable<Poly<F>, Output = U::Repr>;

    fn hmap_async_local<U: Generic, F>(self, f: F) -> impl Future<Output = U>
    where
        Self::Repr: AsyncLocalHMappable<Poly<F>, Output = U::Repr>;

    fn hmap_async_par<U: Generic, F: Send>(self, f: F) -> impl Future<Output = U> + Send
    where
        Self: Send,
        Self::Repr: AsyncParHMappable<Poly<F>, Output = U::Repr>;

    fn hmap_async_local_par<U: Generic, F>(self, f: F) -> impl Future<Output = U>
    where
        Self::Repr: AsyncLocalParHMappable<Poly<F>, Output = U::Repr>;

    fn hzip<U: Generic, TU: Generic<Repr = <Self::Repr as HZippable<U::Repr>>::Zipped>>(
        self,
        other: U,
    ) -> TU
    where
        Self::Repr: HZippable<U::Repr>;

    fn map_to_list<F, U>(self, f: F) -> ConsList<U, <Self::Repr as MapToList<F, U>>::Output>
    where
        Self::Repr: MapToList<F, U>;

    fn map_to_list_async<U, F: Send>(
        self,
        f: F,
    ) -> impl Future<Output = ConsList<U, <Self::Repr as AsyncMapToList<Poly<F>, U>>::Output>> + Send
    where
        Self: Send,
        Self::Repr: AsyncMapToList<Poly<F>, U>;

    fn map_to_list_async_local<U, F>(
        self,
        f: F,
    ) -> impl Future<Output = ConsList<U, <Self::Repr as AsyncLocalMapToList<Poly<F>, U>>::Output>>
    where
        Self::Repr: AsyncLocalMapToList<Poly<F>, U>;

    fn map_to_list_async_par<U, F: Send>(
        self,
        f: F,
    ) -> impl Future<Output = ConsList<U, <Self::Repr as AsyncParMapToList<Poly<F>, U>>::Output>> + Send
    where
        Self: Send,
        Self::Repr: AsyncParMapToList<Poly<F>, U>;

    fn map_to_list_async_local_par<U, F>(
        self,
        f: F,
    ) -> impl Future<Output = ConsList<U, <Self::Repr as AsyncLocalParMapToList<Poly<F>, U>>::Output>>
    where
        Self::Repr: AsyncLocalParMapToList<Poly<F>, U>;

    fn for_each<F>(self, f: F)
    where
        Self::Repr: ForEach<F>;

    fn for_each_async<F: Send>(self, f: F) -> impl Future<Output = ()> + Send
    where
        Self: Send,
        Self::Repr: AsyncForEach<Poly<F>>;

    fn for_each_async_local<F>(self, f: F) -> impl Future<Output = ()>
    where
        Self::Repr: AsyncLocalForEach<Poly<F>>;

    fn for_each_async_par<F: Send>(self, f: F) -> impl Future<Output = ()> + Send
    where
        Self: Send,
        Self::Repr: AsyncParForEach<Poly<F>>;

    fn for_each_async_local_par<F>(self, f: F) -> impl Future<Output = ()>
    where
        Self::Repr: AsyncLocalParForEach<Poly<F>>;

    /// Allows getting an iterator over the fields of a struct if they all have the same type
    fn fields_into_iter<U>(self) -> impl Iterator<Item = U>
    where
        Self::Repr: MapToList<Identity, U>;
}

pub struct Identity;

impl<T> Func<T> for Identity {
    type Output = T;

    fn call(&mut self, i: T) -> Self::Output {
        i
    }
}

impl<T: Generic> WithGeneric for T {
    fn hmap<U: Generic, F>(self, f: F) -> U
    where
        Self::Repr: HMappable<Poly<F>, Output = U::Repr>,
    {
        from_generic(into_generic(self).map(Poly(f)))
    }

    async fn hmap_async<U: Generic, F: Send>(self, f: F) -> U
    where
        Self: Send,
        Self::Repr: AsyncHMappable<Poly<F>, Output = U::Repr>,
    {
        from_generic(into_generic(self).map(Poly(f)).await)
    }

    async fn hmap_async_local<U: Generic, F>(self, f: F) -> U
    where
        Self::Repr: AsyncLocalHMappable<Poly<F>, Output = U::Repr>,
    {
        from_generic(into_generic(self).map_local(Poly(f)).await)
    }

    async fn hmap_async_par<U: Generic, F: Send>(self, f: F) -> U
    where
        Self: Send,
        Self::Repr: AsyncParHMappable<Poly<F>, Output = U::Repr>,
    {
        from_generic(into_generic(self).par_map(&Poly(f)).await)
    }

    async fn hmap_async_local_par<U: Generic, F>(self, f: F) -> U
    where
        Self::Repr: AsyncLocalParHMappable<Poly<F>, Output = U::Repr>,
    {
        from_generic(into_generic(self).par_map_local(&Poly(f)).await)
    }

    fn hzip<U: Generic, TU: Generic<Repr = <Self::Repr as HZippable<U::Repr>>::Zipped>>(
        self,
        other: U,
    ) -> TU
    where
        Self::Repr: HZippable<U::Repr>,
    {
        from_generic(into_generic(self).zip(into_generic(other)))
    }

    fn map_to_list<F, U>(self, f: F) -> ConsList<U, <Self::Repr as MapToList<F, U>>::Output>
    where
        Self::Repr: MapToList<F, U>,
    {
        into_generic(self).map_to_list(f)
    }

    async fn map_to_list_async<U, F: Send>(
        self,
        f: F,
    ) -> ConsList<U, <Self::Repr as AsyncMapToList<Poly<F>, U>>::Output>
    where
        Self: Send,
        Self::Repr: AsyncMapToList<Poly<F>, U>,
    {
        into_generic(self).map_to_list(Poly(f)).await
    }

    async fn map_to_list_async_local<U, F>(
        self,
        f: F,
    ) -> ConsList<U, <Self::Repr as AsyncLocalMapToList<Poly<F>, U>>::Output>
    where
        Self::Repr: AsyncLocalMapToList<Poly<F>, U>,
    {
        into_generic(self).map_to_list_local(Poly(f)).await
    }

    async fn map_to_list_async_par<U, F: Send>(
        self,
        f: F,
    ) -> ConsList<U, <Self::Repr as AsyncParMapToList<Poly<F>, U>>::Output>
    where
        Self: Send,
        Self::Repr: AsyncParMapToList<Poly<F>, U>,
    {
        into_generic(self).par_map_to_list(&Poly(f)).await
    }

    async fn map_to_list_async_local_par<U, F>(
        self,
        f: F,
    ) -> ConsList<U, <Self::Repr as AsyncLocalParMapToList<Poly<F>, U>>::Output>
    where
        Self::Repr: AsyncLocalParMapToList<Poly<F>, U>,
    {
        into_generic(self).par_map_to_list_local(&Poly(f)).await
    }

    fn for_each<F>(self, f: F)
    where
        Self::Repr: ForEach<F>,
    {
        into_generic(self).for_each(f)
    }

    async fn for_each_async<F: Send>(self, f: F)
    where
        Self: Send,
        Self::Repr: AsyncForEach<Poly<F>>,
    {
        into_generic(self).for_each(Poly(f)).await
    }

    async fn for_each_async_local<F>(self, f: F)
    where
        Self::Repr: AsyncLocalForEach<Poly<F>>,
    {
        into_generic(self).for_each_local(Poly(f)).await
    }

    async fn for_each_async_par<F: Send>(self, f: F)
    where
        Self: Send,
        Self::Repr: AsyncParForEach<Poly<F>>,
    {
        into_generic(self).par_for_each(&Poly(f)).await
    }

    async fn for_each_async_local_par<F>(self, f: F)
    where
        Self::Repr: AsyncLocalParForEach<Poly<F>>,
    {
        into_generic(self).par_for_each_local(&Poly(f)).await
    }

    fn fields_into_iter<U>(self) -> impl Iterator<Item = U>
    where
        Self::Repr: MapToList<Identity, U>,
    {
        self.map_to_list(Identity).into_iter()
    }
}

/// Convenience functions for the caller to map between similarly-shaped types implementing [LabelledGeneric] without
/// having to explicitly call [from_labelled_generic] and [into_labelled_generic]
pub trait WithLabelledGeneric: LabelledGeneric {
    fn hmap<U: LabelledGeneric, F>(self, f: F) -> U
    where
        Self::Repr: HMappable<Poly<F>, Output = U::Repr>;

    fn hzip<
        U: LabelledGeneric,
        TU: LabelledGeneric<Repr = <Self::Repr as HZippable<U::Repr>>::Zipped>,
    >(
        self,
        other: U,
    ) -> TU
    where
        Self::Repr: HZippable<U::Repr>;

    fn map_to_list<F, U>(self, f: F) -> ConsList<U, <Self::Repr as MapToList<F, U>>::Output>
    where
        Self::Repr: MapToList<F, U>;

    fn for_each<F>(self, f: F)
    where
        Self::Repr: ForEach<F>;
}

impl<T: LabelledGeneric> WithLabelledGeneric for T {
    fn hmap<U: LabelledGeneric, F>(self, f: F) -> U
    where
        Self::Repr: HMappable<Poly<F>, Output = U::Repr>,
    {
        from_labelled_generic(into_labelled_generic(self).map(Poly(f)))
    }

    fn hzip<
        U: LabelledGeneric,
        TU: LabelledGeneric<Repr = <Self::Repr as HZippable<U::Repr>>::Zipped>,
    >(
        self,
        other: U,
    ) -> TU
    where
        Self::Repr: HZippable<U::Repr>,
    {
        from_labelled_generic(into_labelled_generic(self).zip(into_labelled_generic(other)))
    }

    fn map_to_list<F, U>(self, f: F) -> ConsList<U, <Self::Repr as MapToList<F, U>>::Output>
    where
        Self::Repr: MapToList<F, U>,
    {
        into_labelled_generic(self).map_to_list(f)
    }

    fn for_each<F>(self, f: F)
    where
        Self::Repr: ForEach<F>,
    {
        into_labelled_generic(self).for_each(f)
    }
}

pub trait MapToList<F, U>: HList {
    type Output: ConsListT<U>;

    /// Map a monomorphizing function over the HList to produce an [iterable](`ConsList::into_iter`) datastructure which
    /// lives fully on stack
    fn map_to_list(self, f: F) -> ConsList<U, Self::Output>;
}

impl<F, U> MapToList<F, U> for HNil {
    type Output = cons_list::Nil;

    fn map_to_list(self, _f: F) -> ConsList<U, Self::Output> {
        ConsList::nil()
    }
}

impl<F: Func<Head, Output = U>, U, Head, Tail: MapToList<F, U>> MapToList<F, U>
    for HCons<Head, Tail>
{
    type Output = cons_list::Cons<U, <Tail as MapToList<F, U>>::Output>;

    fn map_to_list(self, mut f: F) -> ConsList<U, Self::Output> {
        let HCons { head, tail } = self;
        ConsList::cons(f.call(head), tail.map_to_list(f))
    }
}

pub trait ForEach<F>: HList {
    fn for_each(self, f: F);
}

impl<F> ForEach<F> for HNil {
    fn for_each(self, _: F) {}
}

impl<F: Func<Head, Output = ()>, Head, Tail: ForEach<F>> ForEach<F> for HCons<Head, Tail> {
    fn for_each(self, mut f: F) {
        let HCons { head, tail } = self;
        f.call(head);
        tail.for_each(f)
    }
}
