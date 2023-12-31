use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    pub const X: Vec3 = Vec3 {
        x: 1.,
        y: 0.,
        z: 0.,
    };
    pub const Y: Vec3 = Vec3 {
        x: 0.,
        y: 1.,
        z: 0.,
    };
    pub const Z: Vec3 = Vec3 {
        x: 0.,
        y: 0.,
        z: 1.,
    };
    pub const NEG_X: Vec3 = Vec3 {
        x: -1.,
        y: 0.,
        z: 0.,
    };
    pub const NEG_Y: Vec3 = Vec3 {
        x: 0.,
        y: -1.,
        z: 0.,
    };
    pub const NEG_Z: Vec3 = Vec3 {
        x: 0.,
        y: 0.,
        z: -1.,
    };

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    pub const fn splat(v: f64) -> Self {
        Self::new(v, v, v)
    }

    pub fn scale(self, c: f64) -> Self {
        Self {
            x: self.x * c,
            y: self.y * c,
            z: self.z * c,
        }
    }

    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn squared_magnitude(self) -> f64 {
        self.dot(self)
    }

    #[doc(alias = "l2_norm")]
    #[doc(alias = "length")]
    pub fn magnitude(self) -> f64 {
        self.squared_magnitude().sqrt()
    }

    pub fn normalize(self) -> Self {
        self.magnitude().recip() * self
    }

    pub fn normalize_or_zero(self) -> Self {
        let mag = self.magnitude();
        if mag == 0. {
            Self::ZERO
        } else {
            mag.recip() * self
        }
    }

    pub fn project_onto(self, dir: Self) -> Self {
        self.dot(dir) / dir.dot(dir) * dir
    }

    pub fn reject_wrt(self, dir: Self) -> Self {
        self - self.project_onto(dir)
    }

    pub fn reflect_across(self, normal: Self) -> Self {
        self - 2. * self.project_onto(normal)
    }

    pub fn reflect(self, vec: Self) -> Self {
        vec.reflect_across(self)
    }

    pub fn l1_norm(self) -> f64 {
        self.x.abs() + self.y.abs() + self.z.abs()
    }
}

impl From<Vec3> for [f64; 3] {
    fn from(value: Vec3) -> Self {
        [value.x, value.y, value.z]
    }
}

impl From<[f64; 3]> for Vec3 {
    fn from(value: [f64; 3]) -> Self {
        Self::new(value[0], value[1], value[2])
    }
}

impl IntoIterator for Vec3 {
    type Item = f64;

    type IntoIter = <[f64; 3] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <Self as Into<[f64; 3]>>::into(self).into_iter()
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        self.scale(rhs)
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs.scale(self)
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        #![allow(clippy::suspicious_arithmetic_impl)]
        rhs.recip() * self
    }
}

impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs;
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        -1. * self
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self += -rhs;
    }
}

impl Index<usize> for Vec3 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            i => panic!("Index {i} out of bounds for Vec3"),
        }
    }
}

impl IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            i => panic!("Index {i} out of bounds for Vec3"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mat3x3 {
    pub row1: Vec3,
    pub row2: Vec3,
    pub row3: Vec3,
}

impl Mat3x3 {
    pub const fn from_col_vectors(col1: Vec3, col2: Vec3, col3: Vec3) -> Self {
        Mat3x3 {
            row1: Vec3::new(col1.x, col2.x, col3.x),
            row2: Vec3::new(col1.y, col2.y, col3.y),
            row3: Vec3::new(col1.z, col2.z, col3.z),
        }
    }

    pub const fn from_row_vectors(row1: Vec3, row2: Vec3, row3: Vec3) -> Self {
        Self { row1, row2, row3 }
    }

    pub const fn identity() -> Self {
        Mat3x3::from_col_vectors(
            Vec3::new(1., 0., 0.),
            Vec3::new(0., 1., 0.),
            Vec3::new(0., 0., 1.),
        )
    }

    pub const fn transpose(&self) -> Self {
        Mat3x3::from_col_vectors(self.row1, self.row2, self.row3)
    }

    pub fn scale(mut self, c: f64) -> Self {
        for i in 0..3 {
            self[i] *= c;
        }
        self
    }

    pub fn inverse(mut self) -> Option<Self> {
        let mut inverse = Self::identity();
        if self[0][0] == 0. {
            if self[1][0] != 0. {
                std::mem::swap(&mut self.row1, &mut self.row2);
                std::mem::swap(&mut inverse.row1, &mut inverse.row2);
            } else if self[2][0] != 0. {
                std::mem::swap(&mut self.row1, &mut self.row3);
                std::mem::swap(&mut inverse.row1, &mut inverse.row3);
            } else {
                return None;
            }
        }
        for i in 0..3 {
            if self[i][i] == 0. {
                for j in (i + 1)..3 {
                    if self[j][i] == 0. {
                        continue;
                    }
                    let mut tmp = self[i];
                    self[i] = self[j];
                    self[j] = tmp;
                    tmp = inverse[i];
                    inverse[i] = inverse[j];
                    inverse[j] = tmp;
                    break;
                }
                if self[i][i] == 0. {
                    return None;
                }
            }
            let row_mul_factor = self[i][i].recip();
            inverse[i] *= row_mul_factor;
            self[i] *= row_mul_factor;
            for j in 0..3 {
                if i == j {
                    continue;
                }
                let sub_row_scale_factor = self[j][i];
                let scaled_row_i = sub_row_scale_factor * self[i];
                self[j] -= scaled_row_i;
                let scaled_inverse_i = sub_row_scale_factor * inverse[i];
                inverse[j] -= scaled_inverse_i;
            }
        }
        Some(inverse)
    }
}

impl From<Mat3x3> for [Vec3; 3] {
    fn from(val: Mat3x3) -> Self {
        [val.row1, val.row2, val.row3]
    }
}

impl IntoIterator for Mat3x3 {
    type Item = Vec3;

    type IntoIter = <[Vec3; 3] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <Self as Into<[Vec3; 3]>>::into(self).into_iter()
    }
}

impl Mul<f64> for Mat3x3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        self.scale(rhs)
    }
}

impl Mul<Mat3x3> for f64 {
    type Output = Mat3x3;

    fn mul(self, rhs: Mat3x3) -> Self::Output {
        rhs.scale(self)
    }
}

impl Mul<&Mat3x3> for &Mat3x3 {
    type Output = Mat3x3;

    fn mul(self, rhs: &Mat3x3) -> Self::Output {
        let rhs = rhs.transpose();
        Mat3x3::from_col_vectors(
            Vec3::new(
                self.row1.dot(rhs.row1),
                self.row1.dot(rhs.row2),
                self.row1.dot(rhs.row3),
            ),
            Vec3::new(
                self.row2.dot(rhs.row1),
                self.row2.dot(rhs.row2),
                self.row2.dot(rhs.row3),
            ),
            Vec3::new(
                self.row3.dot(rhs.row1),
                self.row3.dot(rhs.row2),
                self.row3.dot(rhs.row3),
            ),
        )
    }
}

impl MulAssign<f64> for Mat3x3 {
    fn mul_assign(&mut self, rhs: f64) {
        for i in 0..3 {
            self[i] *= rhs;
        }
    }
}

impl MulAssign<&Mat3x3> for Mat3x3 {
    fn mul_assign(&mut self, rhs: &Mat3x3) {
        let rhs = rhs.transpose();
        *self = Self::from_row_vectors(
            Vec3::new(
                self.row1.dot(rhs.row1),
                self.row1.dot(rhs.row2),
                self.row1.dot(rhs.row3),
            ),
            Vec3::new(
                self.row2.dot(rhs.row1),
                self.row2.dot(rhs.row2),
                self.row2.dot(rhs.row3),
            ),
            Vec3::new(
                self.row3.dot(rhs.row1),
                self.row3.dot(rhs.row2),
                self.row3.dot(rhs.row3),
            ),
        );
    }
}

impl Mul<Vec3> for &Mat3x3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(self.row1.dot(rhs), self.row2.dot(rhs), self.row3.dot(rhs))
    }
}

impl Mul<&Mat3x3> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: &Mat3x3) -> Self::Output {
        // self * rhs = output
        // equivalent to (self * rhs)^T = output^T
        // = rhs^T * self^T = output^T
        // = rhs^T * self = output because vec3 is a row and col vector
        &rhs.transpose() * self
    }
}

impl Div<f64> for Mat3x3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        #![allow(clippy::suspicious_arithmetic_impl)]
        rhs.recip() * self
    }
}

impl DivAssign<f64> for Mat3x3 {
    fn div_assign(&mut self, rhs: f64) {
        #![allow(clippy::suspicious_op_assign_impl)]
        *self *= rhs.recip();
    }
}

impl Add<&Mat3x3> for Mat3x3 {
    type Output = Self;

    fn add(mut self, rhs: &Mat3x3) -> Self::Output {
        for i in 0..3 {
            self[i] += rhs[i];
        }
        self
    }
}

impl AddAssign<&Mat3x3> for Mat3x3 {
    fn add_assign(&mut self, rhs: &Mat3x3) {
        for i in 0..3 {
            self[i] += rhs[i];
        }
    }
}

impl Neg for Mat3x3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        -1. * self
    }
}

impl Sub<&Mat3x3> for Mat3x3 {
    type Output = Self;

    fn sub(mut self, rhs: &Mat3x3) -> Self::Output {
        for i in 0..3 {
            self[i] -= rhs[i];
        }
        self
    }
}

impl SubAssign for Mat3x3 {
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..3 {
            self[i] -= rhs[i]
        }
    }
}

impl Index<usize> for Mat3x3 {
    type Output = Vec3;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.row1,
            1 => &self.row2,
            2 => &self.row3,
            i => panic!("Index {i} out of bounds for Vec3"),
        }
    }
}

impl IndexMut<usize> for Mat3x3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.row1,
            1 => &mut self.row2,
            2 => &mut self.row3,
            i => panic!("Index {i} out of bounds for Vec3"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ray {
    pub start: Vec3,
    pub dir: Vec3,
}

impl Ray {
    pub const fn new(start: Vec3, dir: Vec3) -> Self {
        Self { start, dir }
    }

    pub fn new_unit(start: Vec3, dir: Vec3) -> Self {
        Self::new(start, dir.normalize())
    }

    pub fn translate(&self, scale: f64) -> Ray {
        Ray::new(self.point_at(scale), self.dir)
    }

    pub fn point_at(&self, scale: f64) -> Vec3 {
        self.start + self.dir * scale
    }
}
