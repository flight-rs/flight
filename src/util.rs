use std::mem::{size_of, align_of};

fn assert_layout<A, B>() {
    debug_assert_eq!(size_of::<A>(), size_of::<B>());
    debug_assert_eq!(align_of::<A>(), align_of::<B>());
}

/// Can convert between complex wrappers and their primitive forms
pub trait NativeRepr<T: Sized + Copy>: Sized + Copy {
    fn advanced(nat: T) -> Self { *Self::advanced_ref(&nat) }
    fn advanced_ref(nat: &T) -> &Self {
        assert_layout::<T, Self>();
        unsafe { &*(nat as *const T as *const Self) }
    }
    fn advanced_mut(nat: &mut T) -> &mut Self {
        assert_layout::<T, Self>();
        unsafe { &mut *(nat as *mut T as *mut Self) }
    }
    fn native(self) -> T { *self.native_ref() }
    fn native_ref(&self) -> &T {
        assert_layout::<T, Self>();
        unsafe { &*(self as *const Self as *const T) }
    }
    fn native_mut(&mut self) -> &mut T {
        assert_layout::<T, Self>();
        unsafe { &mut *(self as *mut Self as *mut T) }
    }
}

use nalgebra::*;

impl<N: Scalar> NativeRepr<N> for Vector1<N> { }
impl<N: Scalar> NativeRepr<[N; 1]> for Vector1<N> { }
impl<N: Scalar> NativeRepr<[N; 2]> for Vector2<N> { }
impl<N: Scalar> NativeRepr<[N; 3]> for Vector3<N> { }
impl<N: Scalar> NativeRepr<[N; 4]> for Vector4<N> { }
impl<N: Scalar> NativeRepr<[N; 5]> for Vector5<N> { }
impl<N: Scalar> NativeRepr<[N; 6]> for Vector6<N> { }

impl<N: Scalar> NativeRepr<N> for Point1<N> { }
impl<N: Scalar> NativeRepr<[N; 1]> for Point1<N> { }
impl<N: Scalar> NativeRepr<[N; 2]> for Point2<N> { }
impl<N: Scalar> NativeRepr<[N; 3]> for Point3<N> { }
impl<N: Scalar> NativeRepr<[N; 4]> for Point4<N> { }
impl<N: Scalar> NativeRepr<[N; 5]> for Point5<N> { }
impl<N: Scalar> NativeRepr<[N; 6]> for Point6<N> { }

impl<N: Scalar> NativeRepr<[N; 2]> for Translation2<N> { }
impl<N: Scalar> NativeRepr<[N; 3]> for Translation3<N> { }
impl<N: Real> NativeRepr<[N; 4]> for Quaternion<N> { }

impl<N: Scalar> NativeRepr<[N; 4]> for Matrix2<N> { }
impl<N: Scalar> NativeRepr<[[N; 2]; 2]> for Matrix2<N> { }
impl<N: Scalar> NativeRepr<[N; 9]> for Matrix3<N> { }
impl<N: Scalar> NativeRepr<[[N; 3]; 3]> for Matrix3<N> { }
impl<N: Scalar> NativeRepr<[N; 16]> for Matrix4<N> { }
impl<N: Scalar> NativeRepr<[[N; 4]; 4]> for Matrix4<N> { }
impl<N: Scalar> NativeRepr<[N; 25]> for Matrix5<N> { }
impl<N: Scalar> NativeRepr<[[N; 5]; 5]> for Matrix5<N> { }
impl<N: Scalar> NativeRepr<[N; 36]> for Matrix6<N> { }
impl<N: Scalar> NativeRepr<[[N; 6]; 6]> for Matrix6<N> { }

impl<N: Real> NativeRepr<[N; 9]> for Transform2<N> { }
impl<N: Real> NativeRepr<[[N; 3]; 3]> for Transform2<N> { }
impl<N: Real> NativeRepr<[N; 16]> for Transform3<N> { }
impl<N: Real> NativeRepr<[[N; 4]; 4]> for Transform3<N> { }