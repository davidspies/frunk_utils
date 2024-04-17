//! Utilities for working with frunk.

use frunk::{
    hlist::{HMappable, HZippable},
    prelude::HList,
    Generic, HCons, HNil, LabelledGeneric,
};

pub use self::cons_list::{ConsList, ConsListT};

pub mod cons_list;

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
    type Output = HCons<<F as Func<Head>>::Output, <Tail as HMappable<Poly<F>>>::Output>;

    fn map(self, mut mapper: Poly<F>) -> Self::Output {
        let HCons { head, tail } = self;
        HCons {
            head: mapper.0.call(head),
            tail: tail.map(mapper),
        }
    }
}

/// Convenience functions for the caller to map between similarly-shaped types implementing [Generic] without having to
/// explicitly call [Generic::from] and [Generic::into]
pub trait WithGeneric: Generic {
    fn hmap<U: Generic, F>(self, f: F) -> U
    where
        Self::Repr: HMappable<Poly<F>, Output = U::Repr>;

    fn hzip<U: Generic, TU: Generic<Repr = <Self::Repr as HZippable<U::Repr>>::Zipped>>(
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
        Generic::from(Generic::into(self).map(Poly(f)))
    }

    fn hzip<U: Generic, TU: Generic<Repr = <Self::Repr as HZippable<U::Repr>>::Zipped>>(
        self,
        other: U,
    ) -> TU
    where
        Self::Repr: HZippable<U::Repr>,
    {
        Generic::from(Generic::into(self).zip(Generic::into(other)))
    }

    fn map_to_list<F, U>(self, f: F) -> ConsList<U, <Self::Repr as MapToList<F, U>>::Output>
    where
        Self::Repr: MapToList<F, U>,
    {
        Generic::into(self).map_to_list(f)
    }

    fn for_each<F>(self, f: F)
    where
        Self::Repr: ForEach<F>,
    {
        Generic::into(self).for_each(f)
    }

    fn fields_into_iter<U>(self) -> impl Iterator<Item = U>
    where
        Self::Repr: MapToList<Identity, U>,
    {
        self.map_to_list(Identity).into_iter()
    }
}

/// Convenience functions for the caller to map between similarly-shaped types implementing [LabelledGeneric] without
/// having to explicitly call [LabelledGeneric::from] and [LabelledGeneric::into]
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
        LabelledGeneric::from(LabelledGeneric::into(self).map(Poly(f)))
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
        LabelledGeneric::from(LabelledGeneric::into(self).zip(LabelledGeneric::into(other)))
    }

    fn map_to_list<F, U>(self, f: F) -> ConsList<U, <Self::Repr as MapToList<F, U>>::Output>
    where
        Self::Repr: MapToList<F, U>,
    {
        LabelledGeneric::into(self).map_to_list(f)
    }

    fn for_each<F>(self, f: F)
    where
        Self::Repr: ForEach<F>,
    {
        LabelledGeneric::into(self).for_each(f)
    }
}

pub trait MapToList<F, U>: HList {
    type Output: ConsListT<U>;

    /// Map a monomorphizing function over the HList to produce an [iterable](`ConsList::into_iter`) datastructure which
    /// lives fully on stack
    fn map_to_list(self, f: F) -> ConsList<U, Self::Output>;
}

impl<F, U> MapToList<F, U> for HNil {
    type Output = cons_list::Nil<U>;

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
