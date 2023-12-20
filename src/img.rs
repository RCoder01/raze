use std::fmt::Formatter;

use crate::{scene::Display, math::Vec3};

#[derive(Debug, Clone, Copy, Default)]
pub struct Color(pub f64, pub f64, pub f64);

impl Color {
    pub const BLACK: Color = Color(0., 0., 0.);
    pub const WHITE: Color = Color(1., 1., 1.);
    pub const RED: Color = Color(1., 0., 0.);
    pub const GREEN: Color = Color(0., 1., 0.);
    pub const BLUE: Color = Color(0., 0., 1.);

    pub const fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self(r, g, b)
    }

    pub const fn gray(brightness: f64) -> Self {
        Self(brightness, brightness, brightness)
    }

    pub fn to_rgb_bytes(self) -> [u8; 3] {
        [
            to_percent_byte(self.0),
            to_percent_byte(self.1),
            to_percent_byte(self.2),
        ]
    }
}

impl From<Vec3> for Color {
    fn from(value: Vec3) -> Self {
        Self(value.x, value.y, value.z)
    }
}

#[derive(Debug, Clone)]
pub struct Image {
    width: usize,
    height: usize,
    data: Box<[Color]>,
}

impl Image {
    pub fn zeros(size: Display) -> Self {
        Self {
            width: size.x as usize,
            height: size.y as usize,
            data: vec![Color::BLACK; size.x as usize * size.y as usize].into(),
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn at(&self, x: usize, y: usize) -> Color {
        self.data[y * self.width + x]
    }

    pub fn at_mut(&mut self, x: usize, y: usize) -> &mut Color {
        &mut self.data[(self.height - y - 1) * self.width + x]
    }

    pub fn data(&self) -> &[Color] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [Color] {
        &mut self.data
    }
}

#[derive(Debug)]
pub struct PPMWriter<'a>(&'a Image);

impl<'a> From<&'a Image> for PPMWriter<'a> {
    fn from(value: &'a Image) -> Self {
        Self(value)
    }
}

impl<'a> std::fmt::Display for PPMWriter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "P3\n{} {}\n255\n", self.0.width, self.0.height)?;
        for (i, datum) in self.0.data.iter().enumerate() {
            if i % self.0.width == 0 {
                writeln!(f)?;
            }
            let [r, g, b] = datum.to_rgb_bytes();
            write!(f, "{r} {g} {b} ")?;
        }
        Ok(())
    }
}

fn to_percent_byte(x: f64) -> u8 {
    (x * 256.).clamp(0., 255.).floor() as u8
}
