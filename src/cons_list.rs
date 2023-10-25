use std::{marker::PhantomData, mem::ManuallyDrop};

#[repr(C)]
pub struct Cons<T, Tail>(ManuallyDrop<T>, Tail);
pub struct Nil<T>(PhantomData<T>);

impl<T, Tail> Drop for Cons<T, Tail> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.0);
        }
    }
}

pub trait ConsListT<T> {
    const LEN: usize;

    /// # Safety
    ///
    /// `i` must be less than the length of the list and not be used again.
    ///
    /// If this function is ever called, it must be called on every `i` exactly once and the list itself must not be
    /// dropped.
    unsafe fn take_unchecked(&mut self, i: usize) -> T;
}

impl<T> ConsListT<T> for Nil<T> {
    const LEN: usize = 0;

    unsafe fn take_unchecked(&mut self, _: usize) -> T {
        panic!("Index out of bounds")
    }
}

impl<T, Ts: ConsListT<T>> ConsListT<T> for Cons<T, Ts> {
    const LEN: usize = 1 + Ts::LEN;

    unsafe fn take_unchecked(&mut self, i: usize) -> T {
        ManuallyDrop::take(&mut *(&mut self.0 as *mut ManuallyDrop<T>).add(i))
    }
}

pub struct ConsList<T, Ts: ConsListT<T>> {
    list: Ts,
    marker: PhantomData<T>,
}

impl<T, Ts: ConsListT<T>> ConsList<T, Ts> {
    pub fn into_iter(self) -> impl Iterator<Item = T> + DoubleEndedIterator + ExactSizeIterator {
        let ConsList { list, .. } = self;
        let mut list = ManuallyDrop::new(list);
        FullyConsumeOnDrop((0..Ts::LEN).map(move |i| unsafe { list.take_unchecked(i) }))
    }
}

impl<T> ConsList<T, Nil<T>> {
    pub fn nil() -> ConsList<T, Nil<T>> {
        Self {
            list: Nil(PhantomData),
            marker: PhantomData,
        }
    }
}

impl<T, Ts: ConsListT<T>> ConsList<T, Cons<T, Ts>> {
    pub fn cons(head: T, tail: ConsList<T, Ts>) -> ConsList<T, Cons<T, Ts>> {
        let ConsList { list: tail, marker } = tail;
        ConsList {
            list: Cons(ManuallyDrop::new(head), tail),
            marker,
        }
    }
}

pub struct FullyConsumeOnDrop<I: Iterator>(I);

impl<I: Iterator> Iterator for FullyConsumeOnDrop<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<I: DoubleEndedIterator> DoubleEndedIterator for FullyConsumeOnDrop<I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<I: ExactSizeIterator> ExactSizeIterator for FullyConsumeOnDrop<I> {}

impl<I: Iterator> Drop for FullyConsumeOnDrop<I> {
    fn drop(&mut self) {
        for _ in &mut self.0 {}
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use super::ConsList;

    #[test]
    fn collect() {
        let list = ConsList::cons(1, ConsList::cons(2, ConsList::cons(3, ConsList::nil())));
        assert_eq!(list.into_iter().collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn partial_consume() {
        struct DropSignaller {
            value: i32,
            dropped: Rc<Cell<bool>>,
        }
        impl Drop for DropSignaller {
            fn drop(&mut self) {
                self.dropped.set(true)
            }
        }
        let drop_signals = vec![
            Rc::new(Cell::new(false)),
            Rc::new(Cell::new(false)),
            Rc::new(Cell::new(false)),
        ];

        let list = ConsList::cons(
            DropSignaller {
                value: 1,
                dropped: drop_signals[0].clone(),
            },
            ConsList::cons(
                DropSignaller {
                    value: 2,
                    dropped: drop_signals[1].clone(),
                },
                ConsList::cons(
                    DropSignaller {
                        value: 3,
                        dropped: drop_signals[2].clone(),
                    },
                    ConsList::nil(),
                ),
            ),
        );
        let mut iter = list.into_iter();
        assert_eq!(iter.next().unwrap().value, 1);
        assert!(drop_signals[0].get());
        assert!(!drop_signals[1].get());
        assert!(!drop_signals[2].get());
        drop(iter);
        assert!(drop_signals[1].get());
        assert!(drop_signals[2].get());
    }
}
