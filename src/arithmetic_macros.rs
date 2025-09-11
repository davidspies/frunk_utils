use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};

pub use frunk::ToMut;

use crate::Func;

pub struct Negator;
impl<S: std::ops::Neg> Func<S> for Negator {
    type Output = S::Output;

    fn call(&mut self, s: S) -> Self::Output {
        -s
    }
}

#[macro_export]
macro_rules! impl_neg {
    ($(@($($params:tt)+):)? $l:ty => $o:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? std::ops::Neg for $l
        $(where $($bounds)+)?
        {
            type Output = $o;

            fn neg(self) -> Self::Output {
                use $crate::WithGeneric;
                self.hmap($crate::arithmetic_macros::Negator)
            }
        }
    };
}

pub struct Adder;
impl<S: Add<T>, T> Func<(S, T)> for Adder {
    type Output = S::Output;

    fn call(&mut self, (l, r): (S, T)) -> Self::Output {
        l + r
    }
}

#[macro_export]
macro_rules! impl_add {
    ($(@($($params:tt)+):)? $l:ty , $r:ty => $o:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? std::ops::Add<$r> for $l
        $(where $($bounds)+)?
        {
            type Output = $o;

            fn add(self, rhs: $r) -> Self::Output {
                use $crate::WithGeneric;
                self.hzip_with(rhs, $crate::arithmetic_macros::Adder)
            }
        }
    };
}

pub struct AddAssigner;
impl<S: AddAssign<T>, T> Func<(&mut S, T)> for AddAssigner {
    type Output = ();

    fn call(&mut self, (l, r): (&mut S, T)) -> Self::Output {
        *l += r
    }
}

#[macro_export]
macro_rules! impl_add_assign {
    ($(@($($params:tt)+):)? $l:ty , $r:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? std::ops::AddAssign<$r> for $l
        $(where $($bounds)+)?
        {
            fn add_assign(&mut self, rhs: $r) {
                use $crate::WithGeneric;
                use $crate::arithmetic_macros::ToMut;
                self.to_mut().hzipped_for_each(rhs, $crate::arithmetic_macros::AddAssigner)
            }
        }
    };
}

pub struct AddOffset<T>(T);
impl<S: Add<T>, T: Clone> Func<S> for AddOffset<T> {
    type Output = S::Output;

    fn call(&mut self, s: S) -> Self::Output {
        s + self.0.clone()
    }
}

#[macro_export]
macro_rules! impl_add_offset {
    ($(@($($params:tt)+):)? $l:ty , $r:ty => $o:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? Add<$r> for $l
        $(where $($bounds)+)?
        {
            type Output = $o;

            fn add(self, rhs: $r) -> Self::Output {
                use $crate::WithGeneric;
                self.hmap($crate::arithmetic_macros::AddOffset(rhs))
            }
        }
    };
}

pub struct AddAssignOffset<T>(T);
impl<S: AddAssign<T>, T: Clone> Func<&mut S> for AddAssignOffset<T> {
    type Output = ();

    fn call(&mut self, s: &mut S) -> Self::Output {
        *s += self.0.clone()
    }
}

#[macro_export]
macro_rules! impl_add_assign_offset {
    ($(@($($params:tt)+):)? $l:ty , $r:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? AddAssign<$r> for $l
        $(where $($bounds)+)?
        {
            fn add_assign(&mut self, rhs: $r) {
                use $crate::WithGeneric;
                use $crate::arithmetic_macros::ToMut;
                self.to_mut().for_each($crate::arithmetic_macros::AddAssignOffset(rhs));
            }
        }
    };
}

pub struct Subber;
impl<S: Sub<T>, T> Func<(S, T)> for Subber {
    type Output = S::Output;

    fn call(&mut self, (l, r): (S, T)) -> Self::Output {
        l - r
    }
}

#[macro_export]
macro_rules! impl_sub {
    ($(@($($params:tt)+):)? $l:ty , $r:ty => $o:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? std::ops::Sub<$r> for $l
        $(where $($bounds)+)?
        {
            type Output = $o;

            fn sub(self, rhs: $r) -> Self::Output {
                use $crate::WithGeneric;
                self.hzip_with(rhs, $crate::arithmetic_macros::Subber)
            }
        }
    };
}

pub struct SubAssigner;
impl<S: SubAssign<T>, T> Func<(&mut S, T)> for SubAssigner {
    type Output = ();

    fn call(&mut self, (l, r): (&mut S, T)) -> Self::Output {
        *l -= r
    }
}

#[macro_export]
macro_rules! impl_sub_assign {
    ($(@($($params:tt)+):)? $l:ty , $r:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? std::ops::SubAssign<$r> for $l
        $(where $($bounds)+)?
        {
            fn sub_assign(&mut self, rhs: $r) {
                use $crate::WithGeneric;
                use $crate::arithmetic_macros::ToMut;
                self.to_mut().hzipped_for_each(rhs, $crate::arithmetic_macros::SubAssigner)
            }
        }
    };
}

pub struct SubOffset<T>(T);
impl<S: Sub<T>, T: Clone> Func<S> for SubOffset<T> {
    type Output = S::Output;

    fn call(&mut self, s: S) -> Self::Output {
        s - self.0.clone()
    }
}

#[macro_export]
macro_rules! impl_sub_offset {
    ($(@($($params:tt)+):)? $l:ty , $r:ty => $o:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? Sub<$r> for $l
        $(where $($bounds)+)?
        {
            type Output = $o;

            fn sub(self, rhs: $r) -> Self::Output {
                use $crate::WithGeneric;
                self.hmap($crate::arithmetic_macros::SubOffset(rhs))
            }
        }
    };
}

pub struct SubAssignOffset<T>(T);
impl<S: SubAssign<T>, T: Clone> Func<&mut S> for SubAssignOffset<T> {
    type Output = ();

    fn call(&mut self, s: &mut S) -> Self::Output {
        *s -= self.0.clone()
    }
}

#[macro_export]
macro_rules! impl_sub_assign_offset {
    ($(@($($params:tt)+):)? $l:ty , $r:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? SubAssign<$r> for $l
        $(where $($bounds)+)?
        {
            fn sub_assign(&mut self, rhs: $r) {
                use $crate::WithGeneric;
                use $crate::arithmetic_macros::ToMut;
                self.to_mut().for_each($crate::arithmetic_macros::SubAssignOffset(rhs));
            }
        }
    };
}

pub struct Multer;
impl<S: Mul<T>, T> Func<(S, T)> for Multer {
    type Output = S::Output;

    fn call(&mut self, (l, r): (S, T)) -> Self::Output {
        l * r
    }
}

#[macro_export]
macro_rules! impl_mul {
    ($(@($($params:tt)+):)? $l:ty , $r:ty => $o:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? std::ops::Mul<$r> for $l
        $(where $($bounds)+)?
        {
            type Output = $o;

            fn mul(self, rhs: $r) -> Self::Output {
                use $crate::WithGeneric;
                self.hzip_with(rhs, $crate::arithmetic_macros::Multer)
            }
        }
    };
}

pub struct MulAssigner;
impl<S: MulAssign<T>, T> Func<(&mut S, T)> for MulAssigner {
    type Output = ();

    fn call(&mut self, (l, r): (&mut S, T)) -> Self::Output {
        *l *= r
    }
}

#[macro_export]
macro_rules! impl_mul_assign {
    ($(@($($params:tt)+):)? $l:ty , $r:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? std::ops::MulAssign<$r> for $l
        $(where $($bounds)+)?
        {
            fn mul_assign(&mut self, rhs: $r) {
                use $crate::WithGeneric;
                use $crate::arithmetic_macros::ToMut;
                self.to_mut().hzipped_for_each(rhs, $crate::arithmetic_macros::MulAssigner)
            }
        }
    };
}

pub struct MulScalar<T>(T);
impl<S: Mul<T>, T: Clone> Func<S> for MulScalar<T> {
    type Output = S::Output;

    fn call(&mut self, s: S) -> Self::Output {
        s * self.0.clone()
    }
}

#[macro_export]
macro_rules! impl_mul_scalar {
    ($(@($($params:tt)+):)? $l:ty , $r:ty => $o:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? Mul<$r> for $l
        $(where $($bounds)+)?
        {
            type Output = $o;

            fn mul(self, rhs: $r) -> Self::Output {
                use $crate::WithGeneric;
                self.hmap($crate::arithmetic_macros::MulScalar(rhs))
            }
        }
    };
}

pub struct MulAssignScalar<T>(T);
impl<S: MulAssign<T>, T: Clone> Func<&mut S> for MulAssignScalar<T> {
    type Output = ();

    fn call(&mut self, s: &mut S) -> Self::Output {
        *s *= self.0.clone()
    }
}

#[macro_export]
macro_rules! impl_mul_assign_scalar {
    ($(@($($params:tt)+):)? $l:ty , $r:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? MulAssign<$r> for $l
        $(where $($bounds)+)?
        {
            fn mul_assign(&mut self, rhs: $r) {
                use $crate::WithGeneric;
                use $crate::arithmetic_macros::ToMut;
                self.to_mut().for_each($crate::arithmetic_macros::MulAssignScalar(rhs));
            }
        }
    };
}

pub struct Divider;
impl<S: Div<T>, T> Func<(S, T)> for Divider {
    type Output = S::Output;

    fn call(&mut self, (l, r): (S, T)) -> Self::Output {
        l / r
    }
}

#[macro_export]
macro_rules! impl_div {
    ($(@($($params:tt)+):)? $l:ty , $r:ty => $o:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? std::ops::Div<$r> for $l
        $(where $($bounds)+)?
        {
            type Output = $o;

            fn div(self, rhs: $r) -> Self::Output {
                use $crate::WithGeneric;
                self.hzip_with(rhs, $crate::arithmetic_macros::Divider)
            }
        }
    };
}

pub struct DivAssigner;
impl<S: DivAssign<T>, T> Func<(&mut S, T)> for DivAssigner {
    type Output = ();

    fn call(&mut self, (l, r): (&mut S, T)) -> Self::Output {
        *l /= r
    }
}

#[macro_export]
macro_rules! impl_div_assign {
    ($(@($($params:tt)+):)? $l:ty , $r:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? std::ops::DivAssign<$r> for $l
        $(where $($bounds)+)?
        {
            fn div_assign(&mut self, rhs: $r) {
                use $crate::WithGeneric;
                use $crate::arithmetic_macros::ToMut;
                self.to_mut().hzipped_for_each(rhs, $crate::arithmetic_macros::DivAssigner)
            }
        }
    };
}

pub struct DivScalar<T>(T);
impl<S: Div<T>, T: Clone> Func<S> for DivScalar<T> {
    type Output = S::Output;

    fn call(&mut self, s: S) -> Self::Output {
        s / self.0.clone()
    }
}

#[macro_export]
macro_rules! impl_div_scalar {
    ($(@($($params:tt)+):)? $l:ty , $r:ty => $o:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? Div<$r> for $l
        $(where $($bounds)+)?
        {
            type Output = $o;

            fn div(self, rhs: $r) -> Self::Output {
                use $crate::WithGeneric;
                self.hmap($crate::arithmetic_macros::DivScalar(rhs))
            }
        }
    };
}

pub struct DivAssignScalar<T>(T);
impl<S: DivAssign<T>, T: Clone> Func<&mut S> for DivAssignScalar<T> {
    type Output = ();

    fn call(&mut self, s: &mut S) -> Self::Output {
        *s /= self.0.clone()
    }
}

#[macro_export]
macro_rules! impl_div_assign_scalar {
    ($(@($($params:tt)+):)? $l:ty , $r:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? DivAssign<$r> for $l
        $(where $($bounds)+)?
        {
            fn div_assign(&mut self, rhs: $r) {
                use $crate::WithGeneric;
                use $crate::arithmetic_macros::ToMut;
                self.to_mut().for_each($crate::arithmetic_macros::DivAssignScalar(rhs));
            }
        }
    };
}

pub struct Remainder;
impl<S: Rem<T>, T> Func<(S, T)> for Remainder {
    type Output = S::Output;

    fn call(&mut self, (l, r): (S, T)) -> Self::Output {
        l % r
    }
}

#[macro_export]
macro_rules! impl_rem {
    ($(@($($params:tt)+):)? $l:ty , $r:ty => $o:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? std::ops::Rem<$r> for $l
        $(where $($bounds)+)?
        {
            type Output = $o;

            fn rem(self, rhs: $r) -> Self::Output {
                use $crate::WithGeneric;
                self.hzip_with(rhs, $crate::arithmetic_macros::Remainder)
            }
        }
    };
}

pub struct RemAssigner;
impl<S: RemAssign<T>, T> Func<(&mut S, T)> for RemAssigner {
    type Output = ();

    fn call(&mut self, (l, r): (&mut S, T)) -> Self::Output {
        *l %= r
    }
}

#[macro_export]
macro_rules! impl_rem_assign {
    ($(@($($params:tt)+):)? $l:ty , $r:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? std::ops::RemAssign<$r> for $l
        $(where $($bounds)+)?
        {
            fn rem_assign(&mut self, rhs: $r) {
                use $crate::WithGeneric;
                use $crate::arithmetic_macros::ToMut;
                self.to_mut().hzipped_for_each(rhs, $crate::arithmetic_macros::RemAssigner)
            }
        }
    };
}

pub struct RemScalar<T>(T);
impl<S: Rem<T>, T: Clone> Func<S> for RemScalar<T> {
    type Output = S::Output;

    fn call(&mut self, s: S) -> Self::Output {
        s % self.0.clone()
    }
}

#[macro_export]
macro_rules! impl_rem_scalar {
    ($(@($($params:tt)+):)? $l:ty , $r:ty => $o:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? Rem<$r> for $l
        $(where $($bounds)+)?
        {
            type Output = $o;

            fn rem(self, rhs: $r) -> Self::Output {
                use $crate::WithGeneric;
                self.hmap($crate::arithmetic_macros::RemScalar(rhs))
            }
        }
    };
}

pub struct RemAssignScalar<T>(T);
impl<S: RemAssign<T>, T: Clone> Func<&mut S> for RemAssignScalar<T> {
    type Output = ();

    fn call(&mut self, s: &mut S) -> Self::Output {
        *s %= self.0.clone()
    }
}

#[macro_export]
macro_rules! impl_rem_assign_scalar {
    ($(@($($params:tt)+):)? $l:ty , $r:ty $(where $($bounds:tt)+)?) => {
        impl $(<$($params)+>)? RemAssign<$r> for $l
        $(where $($bounds)+)?
        {
            fn rem_assign(&mut self, rhs: $r) {
                use $crate::WithGeneric;
                use $crate::arithmetic_macros::ToMut;
                self.to_mut().for_each($crate::arithmetic_macros::RemAssignScalar(rhs));
            }
        }
    };
}
