use std::fmt::{Display, Formatter, Result};
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Vec3 {
    coords: [f64; 3]
}

pub type Point3 = Vec3;

impl Vec3 {
    pub fn new(coords: [f64; 3]) -> Self {
        Vec3 { coords }
    }
    pub fn new_xyz(x: f64, y: f64, z: f64) -> Self {
        Vec3 { coords: [x, y, z] }
    }

    pub fn x(&self) -> f64 { self.coords[0] }
    pub fn y(&self) -> f64 { self.coords[1] }
    pub fn z(&self) -> f64 { self.coords[2] }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> f64 {
        dot(self, self)
    }

    pub fn random() -> Vec3 {
        return Self::new_xyz(random_f64(), random_f64(), random_f64());
    }

    pub fn random_range(min: f64, max: f64) -> Vec3 {
        return Self::new_xyz(random_f64_range(min,max), random_f64_range(min,max), random_f64_range(min,max));
    }

    pub fn near_zero(&self) -> bool {
        let s = 1e-8;
        self.x().abs() < s && self.y().abs() < s && self.z().abs() < s
    }
}

pub fn dot(u: &Vec3, v: &Vec3) -> f64 {
    u.x() * v.x() + u.y() * v.y() + u.z() * v.z()
}

pub fn cross(u: &Vec3, v: &Vec3) -> Vec3 {
    Vec3::new_xyz(
        u.y() * v.z() - u.z() * v.y(),
        u.z() * v.x() - u.x() * v.z(),
        u.x() * v.y() - u.y() * v.x(),
    )
}

pub fn random_unit_vector() -> Vec3 {
    loop {
        let p = Vec3::random_range(-1.,1.);
        let lensq = p.length_squared();
        if 1e-160 <= lensq && lensq <= 1. {
            return p / lensq.sqrt();
        }
    }
}

pub fn random_on_hemisphere(normal: &Vec3) -> Vec3 {
    let on_unit_sphere: Vec3 = random_unit_vector();
    if dot(&on_unit_sphere, normal) > 0.0 { 
        // In the same hemisphere as the normal
        return on_unit_sphere;
    } else {
        // In the opposite hemisphere, so turn negative
        return -on_unit_sphere;
    }
}

pub fn reflect(v: &Vec3, n: &Vec3) -> Vec3 {
    // dot(v,n)*(*n) is scaling n by projection of v onto n (no need to divide since n is unit)
    let b =  dot(v,n)*(*n);
    // v goes into the surface, so subtracting 2 of scaled n makes it reflect in opposite direction
    *v - 2.0*b
}

pub fn unit_vector(v: &Vec3) -> Vec3 {
    *v / v.length()
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        // Write the vector components separated by spaces.
        write!(f, "{} {} {}", self.x(), self.y(), self.z())
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vec3::new_xyz(-self.coords[0], -self.coords[1], -self.coords[2])
    }
}

impl Index<usize> for Vec3 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.coords[index]
    }
}

impl IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, _index: usize) -> &mut Self::Output {
        &mut self.coords[_index]
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, _rhs: Self) -> Self::Output {
        Vec3::new_xyz(
            self.x() + _rhs.x(),
            self.y() + _rhs.y(),
            self.z() + _rhs.z()
        )
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, _rhs: Self) {
        self.coords[0] += _rhs.coords[0];
        self.coords[1] += _rhs.coords[1];
        self.coords[2] += _rhs.coords[2];
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, _rhs: Self) -> Self::Output {
        Vec3::new_xyz(
            self.x() - _rhs.x(),
            self.y() - _rhs.y(),
            self.z() - _rhs.z()
        )
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, _rhs: Self) {
        self.coords[0] -= _rhs.coords[0];
        self.coords[1] -= _rhs.coords[1];
        self.coords[2] -= _rhs.coords[2];
    }
}

impl Mul for Vec3 {
    type Output = Self;
    fn mul(self, _rhs: Self) -> Self::Output {
        Vec3::new_xyz(
            self.x() * _rhs.x(),
            self.y() * _rhs.y(),
            self.z() * _rhs.z()
        )
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, _rhs: f64) -> Self::Output {
        Vec3::new_xyz(
            self.x() * _rhs,
            self.y() * _rhs,
            self.z() * _rhs
        )
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, _rhs: Vec3) -> Self::Output {
        _rhs * self
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, _rhs: f64) {
        self.coords[0] *= _rhs;
        self.coords[1] *= _rhs;
        self.coords[2] *= _rhs;
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, _rhs: f64) -> Self::Output {
        Vec3::new_xyz(
            self.x() / _rhs,
            self.y() / _rhs,
            self.z() / _rhs
        )
    }
}

impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, _rhs: f64) {
        self.coords[0] /= _rhs;
        self.coords[1] /= _rhs;
        self.coords[2] /= _rhs;
    }
}

// implement unittests for vec3
#[cfg(test)]
mod tests {
    use super::{cross, dot, unit_vector, Vec3};

    #[test]
    fn test_new() {
        let v = Vec3::new([1.0, 2.0, 3.0]);
        assert_eq!(v.x(), 1.0);
        assert_eq!(v.y(), 2.0);
        assert_eq!(v.z(), 3.0);
    }

    #[test]
    fn test_new_xyz() {
        let v = Vec3::new_xyz(1.0, 2.0, 3.0);
        assert_eq!(v.x(), 1.0);
        assert_eq!(v.y(), 2.0);
        assert_eq!(v.z(), 3.0);
    }

    #[test]
    fn test_neg() {
        let v = Vec3::new_xyz(1.0, -2.0, 3.0);
        let expected = Vec3::new_xyz(-1.0, 2.0, -3.0);
        assert_eq!(-v, expected);
    }

    #[test]
    fn test_index() {
        let v = Vec3::new_xyz(4.0, 5.0, 6.0);
        assert_eq!(v[0], 4.0);
        assert_eq!(v[1], 5.0);
        assert_eq!(v[2], 6.0);
    }

    #[test]
    fn test_index_mut() {
        let mut v = Vec3::new_xyz(1.0, 2.0, 3.0);
        v[1] = 10.0;
        let expected = Vec3::new_xyz(1.0, 10.0, 3.0);
        assert_eq!(v, expected);
    }

    #[test]
    fn test_add() {
        let v1 = Vec3::new_xyz(1.0, 2.0, 3.0);
        let v2 = Vec3::new_xyz(4.0, 5.0, 6.0);
        let expected = Vec3::new_xyz(5.0, 7.0, 9.0);
        assert_eq!(v1 + v2, expected);
    }

    #[test]
    fn test_add_assign() {
        let mut v1 = Vec3::new_xyz(1.0, 2.0, 3.0);
        let v2 = Vec3::new_xyz(4.0, 5.0, 6.0);
        let expected = Vec3::new_xyz(5.0, 7.0, 9.0);
        v1 += v2;
        assert_eq!(v1, expected);
    }

    #[test]
    fn test_sub() {
        let v1 = Vec3::new_xyz(4.0, 5.0, 6.0);
        let v2 = Vec3::new_xyz(1.0, 2.0, 3.0);
        let expected = Vec3::new_xyz(3.0, 3.0, 3.0);
        assert_eq!(v1 - v2, expected);
    }

    #[test]
    fn test_sub_assign() {
        let mut v1 = Vec3::new_xyz(4.0, 5.0, 6.0);
        let v2 = Vec3::new_xyz(1.0, 2.0, 3.0);
        let expected = Vec3::new_xyz(3.0, 3.0, 3.0);
        v1 -= v2;
        assert_eq!(v1, expected);
    }

    #[test]
    fn test_mul_vec() {
        let v1 = Vec3::new_xyz(1.0, 2.0, 3.0);
        let v2 = Vec3::new_xyz(4.0, 5.0, 6.0);
        let expected = Vec3::new_xyz(4.0, 10.0, 18.0);
        assert_eq!(v1 * v2, expected);
    }

    #[test]
    fn test_mul_scalar_before() {
        let v = Vec3::new_xyz(1.0, 2.0, 3.0);
        let scalar = 3.0;
        let expected = Vec3::new_xyz(3.0, 6.0, 9.0);
        assert_eq!(scalar * v, expected);
    }

    #[test]
    fn test_mul_scalar_after() {
        let v = Vec3::new_xyz(1.0, 2.0, 3.0);
        let scalar = 3.0;
        let expected = Vec3::new_xyz(3.0, 6.0, 9.0);
        assert_eq!(v * scalar, expected);
    }

    #[test]
    fn test_mul_assign_scalar() {
        let mut v = Vec3::new_xyz(1.0, 2.0, 3.0);
        v *= 3.0;
        assert_eq!(v, Vec3::new_xyz(3.0, 6.0, 9.0));
    }

    #[test]
    fn test_div_scalar() {
        let v = Vec3::new_xyz(3.0, 6.0, 9.0);
        let scalar = 3.0;
        let expected = Vec3::new_xyz(1.0, 2.0, 3.0);
        assert_eq!(v / scalar, expected);
    }

    #[test]
    fn test_div_assign_scalar() {
        let mut v = Vec3::new_xyz(3.0, 6.0, 9.0);
        v /= 3.0;
        assert_eq!(v, Vec3::new_xyz(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_length_squared() {
        let v = Vec3::new_xyz(1.0, 2.0, 3.0);
        // 1*1 + 2*2 + 3*3 = 1 + 4 + 9 = 14
        assert_eq!(v.length_squared(), 14.0);
    }

    #[test]
    fn test_length() {
        let v = Vec3::new_xyz(1.0, 2.0, 3.0);
        assert_eq!(v.length(), 14.0_f64.sqrt());
    }

    #[test]
    fn test_dot_product() {
        let v1 = Vec3::new_xyz(1.0, 2.0, 3.0);
        let v2 = Vec3::new_xyz(4.0, 5.0, 6.0);
        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        assert_eq!(dot(&v1, &v2), 32.0);
    }

    #[test]
    fn test_cross_product() {
        let v1 = Vec3::new_xyz(1.0, 2.0, 3.0);
        let v2 = Vec3::new_xyz(4.0, 5.0, 6.0);
        // (2*6 - 3*5, 3*4 - 1*6, 1*5 - 2*4) = (-3, 6, -3)
        let expected = Vec3::new_xyz(-3.0, 6.0, -3.0);
        assert_eq!(cross(&v1, &v2), expected);
    }

    #[test]
    fn test_unit_vector() {
        // Vector with length 5
        let v = Vec3::new_xyz(0.0, 3.0, 4.0);
        let expected = Vec3::new_xyz(0.0, 0.6, 0.8);
        assert_eq!(unit_vector(&v), expected);
    }

    #[test]
    fn test_display() {
        let v = Vec3::new_xyz(1.1, 2.2, 3.3);
        assert_eq!(format!("{}", v), "1.1 2.2 3.3");
    }
}