use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

macro_rules! impl_v2 {
    ($s:ty) => {
        impl Vec2<$s> {
            pub fn new(x: $s, y: $s) -> Self {
                Self { x, y }
            }

            pub fn dot(self, other: Self) -> $s {
                (self.x * other.x) +
                (self.y * other.y)
            }

            pub fn hadamard(self, other: Self) -> Self {
                Self {
                    x: (self.x * other.x),
                    y: (self.y * other.y),
                }
            }

            pub fn length_sq(self) -> $s {
                Self::dot(self, self)
            }

            pub fn length(self) -> $s {
                self.length_sq().sqrt()
            }

            pub fn normalize(self) -> Self {
                self * (1.0 / self.length())
            }

            pub fn clamp(self, min: $s, max: $s) -> Self {
                Self {
                    x: self.x.clamp(min, max),
                    y: self.y.clamp(min, max),
                }
            }

            pub fn clamp01(self) -> Self {
                self.clamp(0.0, 1.0)
            }
        }

        //
        // NEGATION
        //

        impl Neg for Vec2<$s> {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self {
                    x: -self.x,
                    y: -self.y,
                }
            }
        }

        //
        // ADDITION
        //

        impl Add for Vec2<$s> {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x + rhs.x,
                    y: self.y + rhs.y,
                }
            }
        }

        impl AddAssign for Vec2<$s> {
            fn add_assign(&mut self, rhs: Self) {
                *self = Self {
                    x: self.x + rhs.x,
                    y: self.y + rhs.y,
                }
            }
        }

        //
        // Subtraction
        //

        impl Sub for Vec2<$s> {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x - rhs.x,
                    y: self.y - rhs.y,
                }
            }
        }

        impl SubAssign for Vec2<$s> {
            fn sub_assign(&mut self, rhs: Self) {
                *self = Self {
                    x: self.x - rhs.x,
                    y: self.y - rhs.y,
                }
            }
        }

        //
        // Scalar Multiplication
        //

        impl Mul<$s> for Vec2<$s> {
            type Output = Self;

            fn mul(self, rhs: $s) -> Self::Output {
                Self {
                    x: self.x * rhs,
                    y: self.y * rhs,
                }
            }
        }

        impl Mul<Vec2<$s>> for $s {
            type Output = Vec2<$s>;

            fn mul(self, rhs: Vec2<$s>) -> Self::Output {
                Self::Output {
                    x: self * rhs.x,
                    y: self * rhs.y,
                }
            }
        }

        impl MulAssign<$s> for Vec2<$s> {
            fn mul_assign(&mut self, rhs: $s) {
                *self = Self {
                    x: self.x * rhs,
                    y: self.y * rhs,
                }
            }
        }
    };
}

macro_rules! impl_v3 {
    ($s:ty) => {
        impl Vec3<$s> {
            pub fn new(x: $s, y: $s, z: $s) -> Self {
                Self { x, y, z }
            }

            pub fn dot(self, other: Self) -> $s {
                (self.x * other.x) +
                (self.y * other.y) +
                (self.z * other.z)
            }

            pub fn cross(self, other: Self) -> Self {
                Self {
                    x: (self.y * other.z) - (self.z * other.y),
                    y: (self.z * other.x) - (self.x * other.z),
                    z: (self.x * other.y) - (self.y * other.x),
                }
            }

            pub fn hadamard(self, other: Self) -> Self {
                Self {
                    x: (self.x * other.x),
                    y: (self.y * other.y),
                    z: (self.z * other.z),
                }
            }

            pub fn length_sq(self) -> $s {
                Self::dot(self, self)
            }

            pub fn length(self) -> $s {
                self.length_sq().sqrt()
            }

            pub fn normalize(self) -> Self {
                self * (1.0 / self.length())
            }

            pub fn clamp(self, min: $s, max: $s) -> Self {
                Self {
                    x: self.x.clamp(min, max),
                    y: self.y.clamp(min, max),
                    z: self.z.clamp(min, max),
                }
            }

            pub fn clamp01(self) -> Self {
                self.clamp(0.0, 1.0)
            }
        }

        //
        // NEGATION
        //

        impl Neg for Vec3<$s> {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self {
                    x: -self.x,
                    y: -self.y,
                    z: -self.z,
                }
            }
        }

        //
        // ADDITION
        //

        impl Add for Vec3<$s> {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x + rhs.x,
                    y: self.y + rhs.y,
                    z: self.z + rhs.z,
                }
            }
        }

        impl AddAssign for Vec3<$s> {
            fn add_assign(&mut self, rhs: Self) {
                *self = Self {
                    x: self.x + rhs.x,
                    y: self.y + rhs.y,
                    z: self.z + rhs.z,
                }
            }
        }

        //
        // Subtraction
        //

        impl Sub for Vec3<$s> {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x - rhs.x,
                    y: self.y - rhs.y,
                    z: self.z - rhs.z,
                }
            }
        }

        impl SubAssign for Vec3<$s> {
            fn sub_assign(&mut self, rhs: Self) {
                *self = Self {
                    x: self.x - rhs.x,
                    y: self.y - rhs.y,
                    z: self.z - rhs.z,
                }
            }
        }

        //
        // Scalar Multiplication
        //

        impl Mul<$s> for Vec3<$s> {
            type Output = Self;

            fn mul(self, rhs: $s) -> Self::Output {
                Self {
                    x: self.x * rhs,
                    y: self.y * rhs,
                    z: self.z * rhs,
                }
            }
        }

        impl Mul<Vec3<$s>> for $s {
            type Output = Vec3<$s>;

            fn mul(self, rhs: Vec3<$s>) -> Self::Output {
                Self::Output {
                    x: self * rhs.x,
                    y: self * rhs.y,
                    z: self * rhs.z,
                }
            }
        }

        impl MulAssign<$s> for Vec3<$s> {
            fn mul_assign(&mut self, rhs: $s) {
                *self = Self {
                    x: self.x * rhs,
                    y: self.y * rhs,
                    z: self.z * rhs,
                }
            }
        }
    };
}

macro_rules! impl_v4 {
    ($s:ty) => {
        impl Vec4<$s> {
            pub fn new(x: $s, y: $s, z: $s, w: $s) -> Self {
                Self { x, y, z, w }
            }

            pub fn dot(self, other: Self) -> $s {
                (self.x * other.x) +
                (self.y * other.y) +
                (self.z * other.z) +
                (self.w * other.w)
            }

            pub fn hadamard(self, other: Self) -> Self {
                Self {
                    x: (self.x * other.x),
                    y: (self.y * other.y),
                    z: (self.z * other.z),
                    w: (self.w * other.w),
                }
            }

            pub fn length_sq(self) -> $s {
                Self::dot(self, self)
            }

            pub fn length(self) -> $s {
                self.length_sq().sqrt()
            }

            pub fn normalize(self) -> Self {
                self * (1.0 / self.length())
            }

            pub fn clamp(self, min: $s, max: $s) -> Self {
                Self {
                    x: self.x.clamp(min, max),
                    y: self.y.clamp(min, max),
                    z: self.z.clamp(min, max),
                    w: self.w.clamp(min, max),
                }
            }

            pub fn clamp01(self) -> Self {
                self.clamp(0.0, 1.0)
            }
        }

        //
        // NEGATION
        //

        impl Neg for Vec4<$s> {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self {
                    x: -self.x,
                    y: -self.y,
                    z: -self.z,
                    w: -self.w,
                }
            }
        }

        //
        // ADDITION
        //

        impl Add for Vec4<$s> {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x + rhs.x,
                    y: self.y + rhs.y,
                    z: self.z + rhs.z,
                    w: self.w + rhs.w,
                }
            }
        }

        impl AddAssign for Vec4<$s> {
            fn add_assign(&mut self, rhs: Self) {
                *self = Self {
                    x: self.x + rhs.x,
                    y: self.y + rhs.y,
                    z: self.z + rhs.z,
                    w: self.w + rhs.w,
                }
            }
        }

        //
        // Subtraction
        //

        impl Sub for Vec4<$s> {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x - rhs.x,
                    y: self.y - rhs.y,
                    z: self.z - rhs.z,
                    w: self.w - rhs.w,
                }
            }
        }

        impl SubAssign for Vec4<$s> {
            fn sub_assign(&mut self, rhs: Self) {
                *self = Self {
                    x: self.x - rhs.x,
                    y: self.y - rhs.y,
                    z: self.z - rhs.z,
                    w: self.w - rhs.w,
                }
            }
        }

        //
        // Scalar Multiplication
        //

        impl Mul<$s> for Vec4<$s> {
            type Output = Self;

            fn mul(self, rhs: $s) -> Self::Output {
                Self {
                    x: self.x * rhs,
                    y: self.y * rhs,
                    z: self.z * rhs,
                    w: self.w * rhs,
                }
            }
        }

        impl Mul<Vec4<$s>> for $s {
            type Output = Vec4<$s>;

            fn mul(self, rhs: Vec4<$s>) -> Self::Output {
                Self::Output {
                    x: self * rhs.x,
                    y: self * rhs.y,
                    z: self * rhs.z,
                    w: self * rhs.w,
                }
            }
        }

        impl MulAssign<$s> for Vec4<$s> {
            fn mul_assign(&mut self, rhs: $s) {
                *self = Self {
                    x: self.x * rhs,
                    y: self.y * rhs,
                    z: self.z * rhs,
                    w: self.w * rhs,
                }
            }
        }
    };
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vec2<T>
{
    pub x: T,
    pub y: T,
}

impl_v2!(f32);
impl_v2!(f64);

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl_v3!(f32);
impl_v3!(f64);

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vec4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl_v4!(f32);
impl_v4!(f64);
