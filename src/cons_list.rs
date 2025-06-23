use std::{iter::FusedIterator, marker::PhantomData, mem::ManuallyDrop, ops::Range, ptr};

#[repr(C)]
pub struct Cons<T, Tail>(T, Tail);

#[repr(C)]
pub struct Nil;

/// Recursively expands to nested `ConsList::cons(…)` calls.
///
/// ```ignore
/// let list = cons_list![1, 2, 3];
/// // expands to:
/// // ConsList::cons(1, ConsList::cons(2, ConsList::cons(3, ConsList::nil())))
/// ```
#[macro_export]
macro_rules! cons_list {
    () => { ConsList::nil() };
    ($head:expr $(, $tail:expr)* $(,)?) => {
        ConsList::cons($head, cons_list![$($tail),*])
    };
}

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

unsafe impl<T> ConsListT<T> for Nil {
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

impl<T> ConsList<T, Nil> {
    pub fn nil() -> Self {
        Self {
            list: Nil,
            marker: PhantomData,
        }
    }
}

impl<T, Ts: ConsListT<T>> ConsList<T, Cons<T, Ts>> {
    pub fn cons(head: T, tail: ConsList<T, Ts>) -> Self {
        let ConsList { list: tail, marker } = tail;
        ConsList {
            list: Cons(head, tail),
            marker,
        }
    }
}

impl<T, Ts: ConsListT<T>> ConsList<T, Ts> {
    pub fn as_slice(&self) -> &[T] {
        self.list.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.list.as_mut_slice()
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
        let list = cons_list![1, 2, 3];
        assert_eq!(list.into_iter().collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn partial_consume() {
        let list = cons_list![1, 2, 3];
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
            let bombs = cons_list![Bomb::default(), Bomb::default(), Bomb::default()];
            let mut bombs = bombs.into_iter().collect::<Vec<_>>();
            for bomb in &mut bombs {
                bomb.disarm();
            }
        }
        assert_eq!(NUM_ALLOC.load(Ordering::SeqCst), 0);

        {
            let list = cons_list![Bomb::default(), Bomb::default(), Bomb::default()];
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
            let list = cons_list![Bomb::default(), Bomb::inert(), Bomb::inert()];
            assert_eq!(NUM_ALLOC.load(Ordering::SeqCst), 3);
            let mut bombs = list.into_iter();
            bombs.next().unwrap().disarm();
            assert_eq!(NUM_ALLOC.load(Ordering::SeqCst), 2);
        }
        assert_eq!(NUM_ALLOC.load(Ordering::SeqCst), 0);

        {
            let _list = cons_list![Bomb::inert(), Bomb::inert(), Bomb::inert()];
            assert_eq!(NUM_ALLOC.load(Ordering::SeqCst), 3);
        }
        assert_eq!(NUM_ALLOC.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn as_slice_returns_correct_view() {
        // [1, 2, 3]
        let list = cons_list![1u8, 2, 3];

        let slice = list.as_slice();
        assert_eq!(slice, &[1, 2, 3], "slice contents mismatch");
        assert_eq!(slice.len(), 3, "slice length mismatch");

        // sanity-check contiguity with raw pointers
        unsafe {
            let base = slice.as_ptr();
            assert_eq!(*base.add(0), 1);
            assert_eq!(*base.add(1), 2);
            assert_eq!(*base.add(2), 3);
        }

        // The list can still be consumed afterwards
        let collected: Vec<_> = list.into_iter().collect();
        assert_eq!(collected, vec![1, 2, 3], "iterator view after as_slice");
    }

    #[test]
    fn as_mut_slice_allows_in_place_mutation() {
        // [10, 20]
        let mut list = cons_list![10i32, 20];

        {
            let slice = list.as_mut_slice();
            assert_eq!(slice, &[10, 20], "initial slice contents");

            // mutate through the slice
            slice[0] = 11;
            slice[1] = 22;
        } // mutable borrow ends here

        let collected: Vec<_> = list.into_iter().collect();
        assert_eq!(collected, vec![11, 22], "mutation via as_mut_slice lost");
    }

    #[test]
    fn round_trip_medium_list() {
        // 16-element list: 0 … 15
        let mut list = cons_list![0u16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let original: Vec<u16> = (0..16).collect();

        // immutable slice view
        let slice = list.as_slice();
        assert_eq!(slice, &original[..], "as_slice view incorrect");

        {
            let slice_mut = list.as_mut_slice();
            for (idx, elem) in slice_mut.iter_mut().enumerate() {
                if idx % 2 == 0 {
                    *elem += 1;
                } else {
                    *elem += 2;
                }
            }
        }

        // verify mutations propagated
        let expected: Vec<u16> = original
            .iter()
            .enumerate()
            .map(|(i, &v)| if i % 2 == 0 { v + 1 } else { v + 2 })
            .collect();

        let collected: Vec<_> = list.into_iter().collect();
        assert_eq!(collected, expected, "round-trip medium list mismatch");
    }

    #[test]
    fn empty_list_slice_is_empty() {
        let list: ConsList<u8, Nil> = ConsList::nil();
        let slice = list.as_slice();
        assert!(slice.is_empty(), "Nil must yield an empty slice");
    }
}
