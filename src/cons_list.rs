use std::{iter::FusedIterator, marker::PhantomData, mem::ManuallyDrop, ops::Range, ptr};

#[repr(C)]
pub struct Cons<T, Tail>(T, Tail);

#[repr(C)]
pub struct Nil<T>(PhantomData<T>);

/// # Safety
/// The memory layout must be compatible with the memory layout of a slice of `T`.
pub unsafe trait ConsListT<T> {
    const LEN: usize;

    /// # Safety
    ///
    /// `i` must be less than the length of the list and not be used again.
    ///
    /// If this function is ever called, it must be called on every `i` exactly once and the list itself must not be
    /// dropped.
    unsafe fn take_unchecked(&mut self, i: usize) -> T;

    fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(ptr::from_ref(self).cast::<T>(), Self::LEN) }
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(ptr::from_mut(self).cast::<T>(), Self::LEN) }
    }
}

unsafe impl<T> ConsListT<T> for Nil<T> {
    const LEN: usize = 0;

    unsafe fn take_unchecked(&mut self, _: usize) -> T {
        panic!("Index out of bounds")
    }
}

unsafe impl<T, Ts: ConsListT<T>> ConsListT<T> for Cons<T, Ts> {
    const LEN: usize = 1 + Ts::LEN;

    unsafe fn take_unchecked(&mut self, i: usize) -> T {
        debug_assert!(i < Self::LEN, "Index out of bounds");
        let head = ptr::from_mut(self);
        let head = head.cast::<T>();
        let elem = head.add(i);
        std::ptr::read(elem)
    }
}

pub struct ConsList<T, Ts: ConsListT<T>> {
    list: Ts,
    marker: PhantomData<T>,
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
            list: Cons(head, tail),
            marker,
        }
    }
}

impl<T, Ts: ConsListT<T>> IntoIterator for ConsList<T, Ts> {
    type Item = T;
    type IntoIter = Iter<T, Ts>;

    fn into_iter(self) -> Self::IntoIter {
        let ConsList { list, .. } = self;
        Iter::new(list)
    }
}

pub struct Iter<T, Ts: ConsListT<T>> {
    list: ManuallyDrop<Ts>,
    alive: Range<usize>,
    marker: PhantomData<T>,
}

impl<T, Ts: ConsListT<T>> Iter<T, Ts> {
    fn new(list: Ts) -> Self {
        Iter {
            list: ManuallyDrop::new(list),
            alive: 0..Ts::LEN,
            marker: PhantomData,
        }
    }
}

impl<T, Ts: ConsListT<T>> Iterator for Iter<T, Ts> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.alive
            .next()
            .map(|idx| unsafe { self.list.take_unchecked(idx) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.len()
    }
}

impl<T, Ts: ConsListT<T>> DoubleEndedIterator for Iter<T, Ts> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.alive
            .next_back()
            .map(|idx| unsafe { self.list.take_unchecked(idx) })
    }
}

impl<T, Ts: ConsListT<T>> ExactSizeIterator for Iter<T, Ts> {
    fn len(&self) -> usize {
        self.alive.len()
    }
}

impl<T, Ts: ConsListT<T>> FusedIterator for Iter<T, Ts> {}

impl<T, Ts: ConsListT<T>> Drop for Iter<T, Ts> {
    fn drop(&mut self) {
        for _ in self {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::sync::atomic::{AtomicIsize, Ordering};

    #[test]
    fn full_consume() {
        let list = ConsList::cons(1, ConsList::cons(2, ConsList::cons(3, ConsList::nil())));
        assert_eq!(list.into_iter().collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn partial_consume() {
        let list = ConsList::cons(1, ConsList::cons(2, ConsList::cons(3, ConsList::nil())));
        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(1));
        drop(iter);
    }

    #[test]
    fn drop_behavior() {
        static NUM_ALLOC: AtomicIsize = AtomicIsize::new(0);

        #[derive(Clone)]
        struct Bomb(bool);

        impl Bomb {
            fn inert() -> Self {
                let mut bomb = Self::default();
                bomb.disarm();
                bomb
            }

            fn disarm(&mut self) {
                self.0 = false;
            }
        }

        impl Default for Bomb {
            fn default() -> Self {
                NUM_ALLOC.fetch_add(1, Ordering::SeqCst);
                Bomb(true)
            }
        }

        impl Drop for Bomb {
            fn drop(&mut self) {
                if self.0 {
                    panic!("failed to disarm");
                }
                NUM_ALLOC.fetch_sub(1, Ordering::SeqCst);
            }
        }

        {
            let bombs = ConsList::cons(
                Bomb::default(),
                ConsList::cons(
                    Bomb::default(),
                    ConsList::cons(Bomb::default(), ConsList::nil()),
                ),
            );
            let mut bombs = bombs.into_iter().collect::<Vec<_>>();
            for bomb in &mut bombs {
                bomb.disarm();
            }
        }
        assert_eq!(NUM_ALLOC.load(Ordering::SeqCst), 0);

        {
            let list = ConsList::cons(
                Bomb::default(),
                ConsList::cons(
                    Bomb::default(),
                    ConsList::cons(Bomb::default(), ConsList::nil()),
                ),
            );
            assert_eq!(NUM_ALLOC.load(Ordering::SeqCst), 3);
            let mut bombs = list.into_iter();
            bombs.next().unwrap().disarm();
            assert_eq!(NUM_ALLOC.load(Ordering::SeqCst), 2);
            bombs.next().unwrap().disarm();
            assert_eq!(NUM_ALLOC.load(Ordering::SeqCst), 1);
            bombs.next().unwrap().disarm();
        }
        assert_eq!(NUM_ALLOC.load(Ordering::SeqCst), 0);

        {
            let list = ConsList::cons(
                Bomb::default(),
                ConsList::cons(
                    Bomb::inert(),
                    ConsList::cons(Bomb::inert(), ConsList::nil()),
                ),
            );
            assert_eq!(NUM_ALLOC.load(Ordering::SeqCst), 3);
            let mut bombs = list.into_iter();
            bombs.next().unwrap().disarm();
            assert_eq!(NUM_ALLOC.load(Ordering::SeqCst), 2);
        }
        assert_eq!(NUM_ALLOC.load(Ordering::SeqCst), 0);

        {
            let _list = ConsList::cons(
                Bomb::inert(),
                ConsList::cons(
                    Bomb::inert(),
                    ConsList::cons(Bomb::inert(), ConsList::nil()),
                ),
            );
            assert_eq!(NUM_ALLOC.load(Ordering::SeqCst), 3);
        }
        assert_eq!(NUM_ALLOC.load(Ordering::SeqCst), 0);
    }
}
