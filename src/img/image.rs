use crate::scene::Display;

use super::Color;

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
