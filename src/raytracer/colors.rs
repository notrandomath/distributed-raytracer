use minifb::Window;

use crate::raytracer::prelude::*;

pub type Color = Vec3;

const INTENSITY: Interval = Interval { min: 0.000, max: 0.999 };

pub fn color_to_rgb(pixel_color: &Color) -> (u32, u32, u32) {
    let mut r = pixel_color.x();
    let mut g = pixel_color.y();
    let mut b = pixel_color.z();

    // Apply a linear to gamma transform for gamma 2
    r = linear_to_gamma(r);
    g = linear_to_gamma(g);
    b = linear_to_gamma(b);

    let rbyte = (255.999 * INTENSITY.clamp(r)) as u32;
    let gbyte = (255.999 * INTENSITY.clamp(g)) as u32;
    let bbyte = (255.999 * INTENSITY.clamp(b)) as u32;

    (rbyte, gbyte, bbyte)
}

pub fn write_color(
    i: i32,
    j: i32,
    width: usize,
    height: usize,
    pixel_color: &Color,
    window: &mut Window, 
    color_buffer: &mut Vec<u32>,
    raw_buffer: &mut Vec<Vec3>,
    count_buffer: &mut Vec<i32>
) -> Result<()> {
    let index = j as usize * width + i as usize;
    raw_buffer[index] += *pixel_color;
    count_buffer[index] += 1;
    let denom = if count_buffer[index] != 0 {count_buffer[index] as f64} else {1.};
    let (rbyte, gbyte, bbyte) = color_to_rgb(&(raw_buffer[index] / denom));
    let color: u32 = (255 << 24) | (rbyte << 16) | (gbyte << 8) | bbyte;
    color_buffer[index] = color;
    window.update_with_buffer(&color_buffer, width, height);
    Ok(())
}

pub fn linear_to_gamma(linear_component: f64) -> f64
{
    if linear_component > 0. {
        return linear_component.sqrt();
    }

    return 0.;
}