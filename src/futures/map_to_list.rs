use std::future::Future;

use frunk::{HCons, HNil};
use futures::join;

use crate::cons_list::{Cons, ConsList, ConsListT, Nil};
use crate::Poly;

use super::funcs::{AsyncFunc, AsyncLocalFunc, AsyncLocalParFunc, AsyncParFunc};

pub trait AsyncLocalMapToList<F, U> {
    type Output: ConsListT<U>;

    fn map_to_list_local(self, f: F) -> impl Future<Output = ConsList<U, Self::Output>>;
}

impl<F, U> AsyncLocalMapToList<F, U> for HNil {
    type Output = Nil;

    async fn map_to_list_local(self, _f: F) -> ConsList<U, Self::Output> {
        ConsList::nil()
    }
}

impl<F: AsyncLocalFunc<Head, Output = U>, U, Head, Tail: AsyncLocalMapToList<Poly<F>, U>>
    AsyncLocalMapToList<Poly<F>, U> for HCons<Head, Tail>
{
    type Output = Cons<U, Tail::Output>;

    async fn map_to_list_local(self, mut f: Poly<F>) -> ConsList<U, Self::Output> {
        let HCons { head, tail } = self;
        ConsList::cons(f.0.call(head).await, tail.map_to_list_local(f).await)
    }
}

pub trait AsyncLocalParMapToList<F, U> {
    type Output: ConsListT<U>;

    fn par_map_to_list_local(self, f: &F) -> impl Future<Output = ConsList<U, Self::Output>>;
}

impl<F, U> AsyncLocalParMapToList<F, U> for HNil {
    type Output = Nil;

    async fn par_map_to_list_local(self, _f: &F) -> ConsList<U, Self::Output> {
        ConsList::nil()
    }
}

impl<F: AsyncLocalParFunc<Head, Output = U>, U, Head, Tail: AsyncLocalParMapToList<Poly<F>, U>>
    AsyncLocalParMapToList<Poly<F>, U> for HCons<Head, Tail>
{
    type Output = Cons<U, Tail::Output>;

    async fn par_map_to_list_local(self, f: &Poly<F>) -> ConsList<U, Self::Output> {
        let HCons { head, tail } = self;
        let (head, tail) = join! {
            f.0.call(head),
            tail.par_map_to_list_local(f),
        };
        ConsList::cons(head, tail)
    }
}

pub trait AsyncMapToList<F, U>: Send {
    type Output: ConsListT<U>;

    fn map_to_list(self, f: F) -> impl Future<Output = ConsList<U, Self::Output>> + Send;
}

impl<F: Send, U> AsyncMapToList<F, U> for HNil {
    type Output = Nil;

    async fn map_to_list(self, _f: F) -> ConsList<U, Self::Output> {
        ConsList::nil()
    }
}

impl<F: AsyncFunc<Head, Output = U>, U: Send, Head: Send, Tail: AsyncMapToList<Poly<F>, U>>
    AsyncMapToList<Poly<F>, U> for HCons<Head, Tail>
{
    type Output = Cons<U, Tail::Output>;

    async fn map_to_list(self, mut f: Poly<F>) -> ConsList<U, Self::Output> {
        let HCons { head, tail } = self;
        ConsList::cons(f.0.call(head).await, tail.map_to_list(f).await)
    }
}

pub trait AsyncParMapToList<F, U>: Send {
    type Output: ConsListT<U> + Send;

    fn par_map_to_list(self, f: &F) -> impl Future<Output = ConsList<U, Self::Output>> + Send;
}

impl<F: Sync, U> AsyncParMapToList<F, U> for HNil {
    type Output = Nil;

    async fn par_map_to_list(self, _f: &F) -> ConsList<U, Self::Output> {
        ConsList::nil()
    }
}

impl<
        F: AsyncParFunc<Head, Output = U>,
        U: Send,
        Head: Send,
        Tail: AsyncParMapToList<Poly<F>, U>,
    > AsyncParMapToList<Poly<F>, U> for HCons<Head, Tail>
{
    type Output = Cons<U, Tail::Output>;

    async fn par_map_to_list(self, f: &Poly<F>) -> ConsList<U, Self::Output> {
        let HCons { head, tail } = self;
        let (head, tail) = join! {
            f.0.call(head),
            tail.par_map_to_list(f),
        };
        ConsList::cons(head, tail)
    }
}
