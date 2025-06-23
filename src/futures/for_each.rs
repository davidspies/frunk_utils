use std::future::Future;

use frunk::{HCons, HNil};
use futures::join;

use crate::Poly;

use super::funcs::{AsyncFunc, AsyncLocalFunc, AsyncLocalParFunc, AsyncParFunc};

pub trait AsyncLocalForEach<F> {
    fn for_each_local(self, f: F) -> impl Future<Output = ()>;
}

impl<F> AsyncLocalForEach<F> for HNil {
    async fn for_each_local(self, _f: F) {}
}

impl<F: AsyncLocalFunc<Head, Output = ()>, Head, Tail: AsyncLocalForEach<Poly<F>>>
    AsyncLocalForEach<Poly<F>> for HCons<Head, Tail>
{
    async fn for_each_local(self, mut f: Poly<F>) {
        let HCons { head, tail } = self;
        f.0.call(head).await;
        tail.for_each_local(f).await
    }
}

pub trait AsyncLocalParForEach<F> {
    fn par_for_each_local(self, f: &F) -> impl Future<Output = ()>;
}

impl<F> AsyncLocalParForEach<F> for HNil {
    async fn par_for_each_local(self, _f: &F) {}
}

impl<F: AsyncLocalParFunc<Head, Output = ()>, Head, Tail: AsyncLocalParForEach<Poly<F>>>
    AsyncLocalParForEach<Poly<F>> for HCons<Head, Tail>
{
    async fn par_for_each_local(self, f: &Poly<F>) {
        let HCons { head, tail } = self;
        ((), ()) = join! {
            f.0.call(head),
            tail.par_for_each_local(f),
        };
    }
}

pub trait AsyncForEach<F>: Send {
    fn for_each(self, f: F) -> impl Future<Output = ()> + Send;
}

impl<F: Send> AsyncForEach<F> for HNil {
    async fn for_each(self, _f: F) {}
}

impl<F: AsyncFunc<Head, Output = ()>, Head: Send, Tail: AsyncForEach<Poly<F>>> AsyncForEach<Poly<F>>
    for HCons<Head, Tail>
{
    async fn for_each(self, mut f: Poly<F>) {
        let HCons { head, tail } = self;
        f.0.call(head).await;
        tail.for_each(f).await
    }
}

pub trait AsyncParForEach<F>: Send {
    fn par_for_each(self, f: &F) -> impl Future<Output = ()> + Send;
}

impl<F: Sync> AsyncParForEach<F> for HNil {
    async fn par_for_each(self, _f: &F) {}
}

impl<F: AsyncParFunc<Head, Output = ()>, Head: Send, Tail: AsyncParForEach<Poly<F>>>
    AsyncParForEach<Poly<F>> for HCons<Head, Tail>
{
    async fn par_for_each(self, f: &Poly<F>) {
        let HCons { head, tail } = self;
        ((), ()) = join! {
            f.0.call(head),
            tail.par_for_each(f),
        };
    }
}
