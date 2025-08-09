use crate::prelude::*;

pub type Color = Vec3;

const INTENSITY: Interval = Interval { min: 0.000, max: 0.999 };

pub fn write_color(writer: &mut impl Write, pixel_color: &Color) -> Result<()> {
    let r = pixel_color.x();
    let g = pixel_color.y();
    let b = pixel_color.z();

    let rbyte = (255.999 * INTENSITY.clamp(r)) as i32;
    let gbyte = (255.999 * INTENSITY.clamp(g)) as i32;
    let bbyte = (255.999 * INTENSITY.clamp(b)) as i32;

    // Write out the pixel color components.
    write!(writer, "{} {} {}\n", rbyte, gbyte, bbyte)?;

    Ok(())
}