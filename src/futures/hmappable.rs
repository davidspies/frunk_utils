use std::future::Future;

use frunk::{HCons, HNil};
use futures::join;

use crate::Poly;

use super::funcs::{AsyncFunc, AsyncLocalFunc, AsyncLocalParFunc, AsyncParFunc};

pub trait AsyncLocalHMappable<Mapper> {
    type Output;

    fn map_local(self, f: Mapper) -> impl Future<Output = Self::Output>;
}

impl<Mapper> AsyncLocalHMappable<Mapper> for HNil {
    type Output = HNil;

    async fn map_local(self, _f: Mapper) -> Self::Output {
        HNil
    }
}

impl<F: AsyncLocalFunc<Head>, Head, Tail: AsyncLocalHMappable<Poly<F>>> AsyncLocalHMappable<Poly<F>>
    for HCons<Head, Tail>
{
    type Output = HCons<F::Output, Tail::Output>;

    async fn map_local(self, mut f: Poly<F>) -> Self::Output {
        let HCons { head, tail } = self;
        HCons {
            head: f.0.call(head).await,
            tail: tail.map_local(f).await,
        }
    }
}

pub trait AsyncLocalParHMappable<Mapper> {
    type Output;

    fn par_map_local(self, f: &Mapper) -> impl Future<Output = Self::Output>;
}

impl<Mapper> AsyncLocalParHMappable<Mapper> for HNil {
    type Output = HNil;

    async fn par_map_local(self, _f: &Mapper) -> Self::Output {
        HNil
    }
}

impl<F: AsyncLocalParFunc<Head>, Head, Tail: AsyncLocalParHMappable<Poly<F>>>
    AsyncLocalParHMappable<Poly<F>> for HCons<Head, Tail>
{
    type Output = HCons<F::Output, Tail::Output>;

    async fn par_map_local(self, f: &Poly<F>) -> Self::Output {
        let HCons { head, tail } = self;
        let (head, tail) = join! {
            f.0.call(head),
            tail.par_map_local(f),
        };
        HCons { head, tail }
    }
}

pub trait AsyncHMappable<Mapper>: Send {
    type Output;

    fn map(self, f: Mapper) -> impl Future<Output = Self::Output> + Send;
}

impl<Mapper: Send> AsyncHMappable<Mapper> for HNil {
    type Output = HNil;

    async fn map(self, _f: Mapper) -> Self::Output {
        HNil
    }
}

impl<F: AsyncFunc<Head>, Head: Send, Tail: AsyncHMappable<Poly<F>>> AsyncHMappable<Poly<F>>
    for HCons<Head, Tail>
{
    type Output = HCons<F::Output, Tail::Output>;

    async fn map(self, mut f: Poly<F>) -> Self::Output {
        let HCons { head, tail } = self;
        HCons {
            head: f.0.call(head).await,
            tail: tail.map(f).await,
        }
    }
}

pub trait AsyncParHMappable<Mapper>: Send {
    type Output: Send;

    fn par_map(self, f: &Mapper) -> impl Future<Output = Self::Output> + Send;
}

impl<Mapper: Sync> AsyncParHMappable<Mapper> for HNil {
    type Output = HNil;

    async fn par_map(self, _f: &Mapper) -> Self::Output {
        HNil
    }
}

impl<F: AsyncParFunc<Head>, Head: Send, Tail: AsyncParHMappable<Poly<F>>> AsyncParHMappable<Poly<F>>
    for HCons<Head, Tail>
{
    type Output = HCons<F::Output, Tail::Output>;

    async fn par_map(self, f: &Poly<F>) -> Self::Output {
        let HCons { head, tail } = self;
        let (head, tail) = join! {
            f.0.call(head),
            tail.par_map(f),
        };
        HCons { head, tail }
    }
}
