use std::fs::File;
use std::io::{Result, Write, BufWriter, stderr};
mod vec3;
const IMAGE_WIDTH: i32 = 256;
const IMAGE_HEIGHT: i32 = 256;
const OUTPUT_FILENAME: &str = "img.ppm";

fn main() -> Result<()>  {
    // make ppm file
    let mut writer = BufWriter::new(File::create(OUTPUT_FILENAME)?);
    let mut err = stderr();
    write!(&mut writer, "P3\n{} {}\n255\n", IMAGE_HEIGHT, IMAGE_HEIGHT)?;
    for j in 0..IMAGE_HEIGHT {
        write!(err, "\rScanlines remaining: {} ", IMAGE_HEIGHT-j)?;
        err.flush()?;
        for i in 0..IMAGE_WIDTH {
            let r: f32 = (i as f32) / (IMAGE_WIDTH-1) as f32;
            let g: f32 = (j as f32) / (IMAGE_HEIGHT-1) as f32;
            let b: f32 = 0.0;

            let ir: i32 = (255.999 * r) as i32;
            let ig: i32 = (255.999 * g) as i32;
            let ib: i32 = (255.999 * b) as i32;

            write!(&mut writer, "{} {} {}\n", ir, ig, ib)?;
        }
    }
    write!(err, "\rDone.                                  \n")?;

    Ok(())
}