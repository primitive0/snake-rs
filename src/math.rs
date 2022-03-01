use std::ops::{Add, Mul, Neg, Sub};

macro_rules! impl_commutative_multiplication {
    {
        fn mul($self: ident : $this: ty, $rhs: ident : $other: ty) -> $out: ty $body: block
    } => {
        impl std::ops::Mul<$other> for $this {
            type Output = $out;

            fn mul($self, $rhs: $other) -> Self::Output $body
        }

        impl std::ops::Mul<$this> for $other {
            type Output = $out;

            fn mul(self, rhs: $this) -> Self::Output {
                rhs.mul(self)
            }
        }
    };
}

pub struct Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3f {
    pub const fn new(x: f32, y: f32, z: f32) -> Vector3f {
        Vector3f { x, y, z }
    }

    pub const fn zeroed() -> Vector3f {
        Vector3f::new(0.0, 0.0, 0.0)
    }

    pub const fn into_array(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

impl Add for Vector3f {
    type Output = Vector3f;

    fn add(self, rhs: Self) -> Self::Output {
        Vector3f {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Neg for Vector3f {
    type Output = Vector3f;

    fn neg(self) -> Self::Output {
        Vector3f {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Sub for Vector3f {
    type Output = Vector3f;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Vector4f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4f {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Vector4f {
        Vector4f { x, y, z, w }
    }

    pub const fn zeroed() -> Vector4f {
        Vector4f::new(0.0, 0.0, 0.0, 0.0)
    }

    pub const fn into_array(self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }
}

impl Add for Vector4f {
    type Output = Vector4f;

    fn add(self, rhs: Self) -> Self::Output {
        Vector4f {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl Neg for Vector4f {
    type Output = Vector4f;

    fn neg(self) -> Self::Output {
        Vector4f {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

impl Sub for Vector4f {
    type Output = Vector4f;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

#[inline]
fn mul_vector4f_by_scalar(vector: &Vector4f, scalar: f32) -> Vector4f {
    Vector4f {
        x: vector.x * scalar,
        y: vector.y * scalar,
        z: vector.z * scalar,
        w: vector.w * scalar,
    }
}

impl_commutative_multiplication! {
    fn mul(self: Vector4f, rhs: f32) -> Vector4f {
        mul_vector4f_by_scalar(&self, rhs)
    }
}

impl_commutative_multiplication! {
    fn mul(self: &Vector4f, rhs: f32) -> Vector4f {
        mul_vector4f_by_scalar(self, rhs)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Matrix4f {
    pub i: Vector4f,
    pub j: Vector4f,
    pub k: Vector4f,
    pub l: Vector4f,
}

impl Matrix4f {
    pub const fn new(i: Vector4f, j: Vector4f, k: Vector4f, l: Vector4f) -> Matrix4f {
        Matrix4f { i, j, k, l }
    }

    pub const fn zeroed() -> Matrix4f {
        Matrix4f::new(Vector4f::zeroed(), Vector4f::zeroed(), Vector4f::zeroed(), Vector4f::zeroed())
    }

    pub const fn identity() -> Matrix4f {
        Matrix4f::new(
            Vector4f::new(1.0, 0.0, 0.0, 0.0),
            Vector4f::new(0.0, 1.0, 0.0, 0.0),
            Vector4f::new(0.0, 0.0, 1.0, 0.0),
            Vector4f::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    #[inline]
    pub fn translate(self, vec3: &Vector3f) -> Matrix4f {
        let mut translation_matrix = Matrix4f::identity();
        translation_matrix.l = Vector4f::new(vec3.x, vec3.y, vec3.z, 1.0);
        return self * translation_matrix;
    }
}

#[inline]
fn mul_matrix4f_by_vector4f(matrix: &Matrix4f, rhs: &Vector4f) -> Vector4f {
    (&matrix.i * rhs.x) + (&matrix.j * rhs.y) + (&matrix.k * rhs.z) + (&matrix.l * rhs.w)
}

impl_commutative_multiplication! {
    fn mul(self: Matrix4f, rhs: Vector4f) -> Vector4f {
        mul_matrix4f_by_vector4f(&self, &rhs)
    }
}

impl_commutative_multiplication! {
    fn mul(self: &Matrix4f, rhs: Vector4f) -> Vector4f {
        mul_matrix4f_by_vector4f(self, &rhs)
    }
}

impl_commutative_multiplication! {
    fn mul(self: Matrix4f, rhs: &Vector4f) -> Vector4f {
        mul_matrix4f_by_vector4f(&self, rhs)
    }
}

impl_commutative_multiplication! {
    fn mul(self: &Matrix4f, rhs: &Vector4f) -> Vector4f {
        mul_matrix4f_by_vector4f(self, rhs)
    }
}

#[inline]
fn mul_matrix4f_by_matrix4f(this: &Matrix4f, rhs: &Matrix4f) -> Matrix4f {
    Matrix4f {
        i: this * &rhs.i,
        j: this * &rhs.j,
        k: this * &rhs.k,
        l: this * &rhs.l,
    }
}

impl Mul for Matrix4f {
    type Output = Matrix4f;

    fn mul(self, rhs: Matrix4f) -> Matrix4f {
        mul_matrix4f_by_matrix4f(&self, &rhs)
    }
}

pub fn ortho2d(left: f32, right: f32, bottom: f32, top: f32) -> Matrix4f {
    let mut matrix = Matrix4f::zeroed();
    matrix.i.x = 2.0 / (right - left);
    matrix.j.y = 2.0 / (top - bottom);
    matrix.k.z = 1.0;
    matrix.l.x = -(right + left) / (right - left);
    matrix.l.y = -(top + bottom) / (top - bottom);
    matrix.l.w = 1.0;
    return matrix;
}
