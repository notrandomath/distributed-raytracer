use crate::prelude::*;

pub type Color = Vec3;

const INTENSITY: Interval = Interval { min: 0.000, max: 0.999 };

pub fn write_color(writer: &mut impl Write, pixel_color: &Color) -> Result<()> {
    let mut r = pixel_color.x();
    let mut g = pixel_color.y();
    let mut b = pixel_color.z();

    // Apply a linear to gamma transform for gamma 2
    r = linear_to_gamma(r);
    g = linear_to_gamma(g);
    b = linear_to_gamma(b);

    let rbyte = (255.999 * INTENSITY.clamp(r)) as i32;
    let gbyte = (255.999 * INTENSITY.clamp(g)) as i32;
    let bbyte = (255.999 * INTENSITY.clamp(b)) as i32;

    // Write out the pixel color components.
    write!(writer, "{} {} {}\n", rbyte, gbyte, bbyte)?;

    Ok(())
}

pub fn linear_to_gamma(linear_component: f64) -> f64
{
    if linear_component > 0. {
        return linear_component.sqrt();
    }

    return 0.;
}