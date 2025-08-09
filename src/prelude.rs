//! A prelude for the ray tracer, containing common imports.

// Re-export common types from our modules.
pub use crate::vec3::{Point3, Vec3, dot, unit_vector};
pub use crate::ray::Ray;
pub use crate::colors::{Color, write_color};

// Re-export common standard library items.
pub use std::rc::Rc;
pub use std::f64::INFINITY;
pub use std::f64::consts::PI;
pub use std::fs::File;
pub use std::io::{Result, Write, BufWriter, stderr};

#[inline]
pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}