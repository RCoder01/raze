use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign, Div, Index};

// #[derive(Debug)]
// pub struct Vector<D, const N: usize>([D; N]);

// impl<D, const N: usize> Vector<D, N> {
//     pub const fn new(data: [D; N]) -> Self {
//         Self(data)
//     }
// }

// impl<D, const N: usize> Default for Vector<D, N>
// where
//     D: Default
// {
//     fn default() -> Self {
//         Self((0..N).map(|_| D::default()).collect::<Vec<_>>().try_into().ok().unwrap())
//     }
// }

// impl<D, const N: usize> Add for Vector<D, N>
// where
//     D: Add,
// {
//     type Output = Vector<<D as Add>::Output, N>;

//     fn add(self, rhs: Self) -> Self::Output {
//         let result: Vec<_> = self
//             .0
//             .into_iter()
//             .zip(rhs.0.into_iter())
//             .map(|(a, b)| a + b)
//             .collect();
//         Vector(result.try_into().ok().unwrap())
//     }
// }

// impl<D, const N: usize> Vector<D, N> {}

// pub type Matrix<D, const N: usize, const M: usize> = Vector<Vector<D, N>, M>;

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    pub const fn splat(v: f32) -> Self {
        Self::new(v, v, v)
    }

    pub fn scale(self, c: f32) -> Self {
        Self {
            x: self.x * c,
            y: self.y * c,
            z: self.z * c,
        }
    }

    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn squared_magnitude(self) -> f32 {
        self.dot(self)
    }

    pub fn magnitude(self) -> f32 {
        self.squared_magnitude().sqrt()
    }

    pub fn normalize(self) -> Self {
        self.magnitude().recip() * self
    }

    pub fn project_onto(self, dir: Self) -> Self {
        self.dot(dir) / dir.dot(dir) * dir
    }

    pub fn reflect_across(self, normal: Self) -> Self {
        self - 2. * self.project_onto(normal)
    }
}

// impl Mul for Vec3 {
//     type Output = f32;

//     fn mul(self, rhs: Self) -> Self::Output {
//         self.dot(rhs)
//     }
// }

impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output {
        self.scale(rhs)
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs.scale(self)
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        rhs.recip() * self
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        -1. * self
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self += -rhs;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl Index<usize> for Vec3 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Index out of bounds for Vec3")
        }
    }
}