use std::io::Write;

use super::Image;
pub use qoi::QOIWriter;

pub trait ImageWriter {
    fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()>;
    fn extension(&self) -> Option<String> {
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PPMWriter<'a>(&'a Image);

impl<'a> From<&'a Image> for PPMWriter<'a> {
    fn from(value: &'a Image) -> Self {
        Self(value)
    }
}

impl<'a> ImageWriter for PPMWriter<'a> {
    fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        write!(writer, "P3\n{} {}\n255\n", self.0.width(), self.0.height())?;
        for (i, datum) in self.0.data().iter().enumerate() {
            if i % self.0.width() == 0 {
                writeln!(writer)?;
            }
            let [r, g, b] = datum.to_rgb_bytes();
            write!(writer, "{r} {g} {b} ")?;
        }
        Ok(())
    }

    fn extension(&self) -> Option<String> {
        Some("ppm".into())
    }
}

mod qoi;
