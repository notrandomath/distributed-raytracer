use crate::vec3::{Vec3, Point3};

pub struct Ray<'a>{
    origin: &'a Point3,
    direction: &'a Vec3
}

impl<'a> Ray<'a> {
    pub fn new(origin: &'a Point3, direction: &'a Vec3) -> Self {
        Ray{origin, direction}
    }

    pub fn origin(&self) -> &'a Point3 { self.origin }
    pub fn direction(&self) -> &'a Vec3 { self.direction }

    pub fn at(&self, t: f64) -> Point3 { 
        *self.origin + t * *self.direction
    }
}