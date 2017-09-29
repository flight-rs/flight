use std::mem::{size_of, align_of};

/// Zero-cost (I looked at the assembly) check that two types can share a pointer 
fn assert_layout<A, B>() {
    assert_eq!(size_of::<A>(), size_of::<B>());
    assert_eq!(align_of::<A>(), align_of::<B>());
}

/// Zero-cost conversion between a complex wrapper and its native form.
/// For example, between `&mut Vector2<f32>` and `&mut [f32; 2]`.
pub trait NativeRepr<T: Sized + Copy>: Sized + Copy {
    /// Upgrade some native data into a high level wrapper
    fn upgrade(nat: T) -> Self { *Self::upgrade_ref(&nat) }
    /// Borrow some native data as a high level wrapper
    fn upgrade_ref(nat: &T) -> &Self {
        assert_layout::<T, Self>();
        unsafe { &*(nat as *const T as *const Self) }
    }
    /// Mutably borrow some native data as a high level wrapper
    fn upgrade_mut(nat: &mut T) -> &mut Self {
        assert_layout::<T, Self>();
        unsafe { &mut *(nat as *mut T as *mut Self) }
    }
    /// Downgrade this high level wrapper into some native data
    fn downgrade(self) -> T { *self.downgrade_ref() }
    /// Borrow this high level wrapper as some native data
    fn downgrade_ref(&self) -> &T {
        assert_layout::<T, Self>();
        unsafe { &*(self as *const Self as *const T) }
    }
    /// Mutably borrow this high level wrapper as some native data
    fn downgrade_mut(&mut self) -> &mut T {
        assert_layout::<T, Self>();
        unsafe { &mut *(self as *mut Self as *mut T) }
    }
}

use nalgebra::*;

macro_rules! transmute_repr {
    ({$(<$g:ident: $t:path> $a:ty = $b:ty;)*}, $test:ident<$tg:ident>) => {
        $(impl<$g: $t> NativeRepr<$b> for $a { })*
        
        #[test]
        fn $test() {
            fn test<$tg: Real>() {
                $(assert_layout::<$a, $b>();)*
            }
            test::<f32>();
            test::<f64>();
        }
    }
}

transmute_repr!({
    <N: Scalar> Vector1<N> = N;
    <N: Scalar> Vector1<N> = [N; 1];
    <N: Scalar> Vector2<N> = [N; 2];
    <N: Scalar> Vector3<N> = [N; 3];
    <N: Scalar> Vector4<N> = [N; 4];
    <N: Scalar> Vector5<N> = [N; 5];
    <N: Scalar> Vector6<N> = [N; 6];
}, vector_layouts<N>);

transmute_repr!({
    <N: Scalar> Point1<N> = N;
    <N: Scalar> Point1<N> = [N; 1];
    <N: Scalar> Point2<N> = [N; 2];
    <N: Scalar> Point3<N> = [N; 3];
    <N: Scalar> Point4<N> = [N; 4];
    <N: Scalar> Point5<N> = [N; 5];
    <N: Scalar> Point6<N> = [N; 6];
}, point_layouts<N>);

transmute_repr!({
    <N: Real> Transform2<N> = [N; 9];
    <N: Real> Transform2<N> = [[N; 3]; 3];
    <N: Real> Transform3<N> = [N; 16];
    <N: Real> Transform3<N> = [[N; 4]; 4];

    <N: Scalar> Translation2<N> = [N; 2];
    <N: Scalar> Translation3<N> = [N; 3];

    <N: Real> Quaternion<N> = [N; 4];
}, transform_layouts<N>);

transmute_repr!({
    <N: Scalar> Matrix2<N> = [N; 4];
    <N: Scalar> Matrix2<N> = [[N; 2]; 2];
    <N: Scalar> Matrix3<N> = [N; 9];
    <N: Scalar> Matrix3<N> = [[N; 3]; 3];
    <N: Scalar> Matrix4<N> = [N; 16];
    <N: Scalar> Matrix4<N> = [[N; 4]; 4];
    <N: Scalar> Matrix5<N> = [N; 25];
    <N: Scalar> Matrix5<N> = [[N; 5]; 5];
    <N: Scalar> Matrix6<N> = [N; 36];
    <N: Scalar> Matrix6<N> = [[N; 6]; 6];
}, matrix_layouts<N>);