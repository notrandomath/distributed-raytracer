use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec3 {
    coords: [f32; 3]
}

impl Vec3 {
    pub fn new(coords: [f32; 3]) -> Self {
        Vec3 { coords }
    }
    pub fn new_xyz(x: f32, y: f32, z: f32) -> Self {
        Vec3 { coords: [x, y, z] }
    }

    pub fn x(&self) -> f32 { self.coords[0] }
    pub fn y(&self) -> f32 { self.coords[1] }
    pub fn z(&self) -> f32 { self.coords[2] }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> f32 {
        self.x().powi(2) + self.y().powi(2)+ self.z().powi(2)
    }

    pub fn dot(&self, _other: Self) -> f32 {
        self.x() * _other.x() + self.y() * _other.y() + self.z() * _other.z()
    }

    pub fn cross(&self, _other: Self) -> Self {
        Vec3::new_xyz(
            self.y() * _other.z() - self.z() * _other.y(),
            self.z() * _other.x() - self.x() * _other.z(),
            self.x() * _other.y() - self.y() * _other.x()
        )
    }

    pub fn unit_vector(self) -> Self {
        self / self.length()
    }
}   

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vec3::new_xyz(-self.coords[0], -self.coords[1], -self.coords[2])
    }
}

impl Index<usize> for Vec3 {
    type Output = f32;

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

impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, _rhs: f32) -> Self::Output {
        Vec3::new_xyz(
            self.x() * _rhs,
            self.y() * _rhs,
            self.z() * _rhs
        )
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, _rhs: f32) {
        self.coords[0] *= _rhs;
        self.coords[1] *= _rhs;
        self.coords[2] *= _rhs;
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;
    fn div(self, _rhs: f32) -> Self::Output {
        Vec3::new_xyz(
            self.x() / _rhs,
            self.y() / _rhs,
            self.z() / _rhs
        )
    }
}

impl DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, _rhs: f32) {
        self.coords[0] /= _rhs;
        self.coords[1] /= _rhs;
        self.coords[2] /= _rhs;
    }
}

// implement unittests for vec3
#[cfg(test)]
mod tests {
    use super::Vec3;

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
    fn test_mul_scalar() {
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
        assert_eq!(v.length(), 14.0_f32.sqrt());
    }

    #[test]
    fn test_dot_product() {
        let v1 = Vec3::new_xyz(1.0, 2.0, 3.0);
        let v2 = Vec3::new_xyz(4.0, 5.0, 6.0);
        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        assert_eq!(v1.dot(v2), 32.0);
    }

    #[test]
    fn test_cross_product() {
        let v1 = Vec3::new_xyz(1.0, 2.0, 3.0);
        let v2 = Vec3::new_xyz(4.0, 5.0, 6.0);
        // (2*6 - 3*5, 3*4 - 1*6, 1*5 - 2*4) = (-3, 6, -3)
        let expected = Vec3::new_xyz(-3.0, 6.0, -3.0);
        assert_eq!(v1.cross(v2), expected);
    }

    #[test]
    fn test_unit_vector() {
        // Vector with length 5
        let v = Vec3::new_xyz(0.0, 3.0, 4.0);
        let expected = Vec3::new_xyz(0.0, 0.6, 0.8);
        assert_eq!(v.unit_vector(), expected);
    }
}