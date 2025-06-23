use std::future::Future;

pub trait AsyncLocalFunc<I> {
    type Output;

    fn call(&mut self, i: I) -> impl Future<Output = Self::Output>;
}

impl<F: AsyncLocalFunc<I>, I> AsyncLocalFunc<I> for &mut F {
    type Output = F::Output;

    fn call(&mut self, i: I) -> impl Future<Output = Self::Output> {
        (*self).call(i)
    }
}

pub trait AsyncFunc<I>: Send {
    type Output: Send;

    fn call(&mut self, i: I) -> impl Future<Output = Self::Output> + Send;
}

impl<F: AsyncFunc<I>, I> AsyncFunc<I> for &mut F {
    type Output = F::Output;

    fn call(&mut self, i: I) -> impl Future<Output = Self::Output> + Send {
        (*self).call(i)
    }
}

pub trait AsyncLocalParFunc<I> {
    type Output;

    fn call(&self, i: I) -> impl Future<Output = Self::Output>;
}

impl<F: AsyncLocalParFunc<I>, I> AsyncLocalParFunc<I> for &F {
    type Output = F::Output;

    fn call(&self, i: I) -> impl Future<Output = Self::Output> {
        F::call(self, i)
    }
}

pub trait AsyncParFunc<I>: Sync {
    type Output: Send;

    fn call(&self, i: I) -> impl Future<Output = Self::Output> + Send;
}

impl<F: AsyncParFunc<I> + Sync, I> AsyncParFunc<I> for &F {
    type Output = F::Output;

    fn call(&self, i: I) -> impl Future<Output = Self::Output> + Send {
        F::call(self, i)
    }
}
