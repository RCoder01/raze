use crate::{
    math::{Ray, Vec3},
    shapes::Shape,
    EPSILON,
};

#[derive(Debug, Clone, Copy)]
pub struct Display {
    pub x: u32,
    pub y: u32,
}

impl IntoIterator for Display {
    type Item = (u32, u32);
    type IntoIter = DisplayIter;

    fn into_iter(self) -> Self::IntoIter {
        DisplayIter::new(self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DisplayIter {
    display: Display,
    x_start: u32,
    x_end: u32,
    y_start: u32,
    y_end: u32,
}

impl DisplayIter {
    pub fn new(display: Display) -> Self {
        Self {
            display,
            x_start: 0,
            x_end: display.x - 1,
            y_start: 0,
            y_end: display.y - 1,
        }
    }
}

impl Iterator for DisplayIter {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y_start == self.y_end && self.x_start > self.x_end {
            return None;
        }
        if self.x_start >= self.display.x {
            self.x_start = 0;
            self.y_start += 1;
        }
        if self.y_start > self.y_end {
            return None;
        }
        self.x_start += 1;
        Some((self.x_start - 1, self.y_start))
    }
}

impl DoubleEndedIterator for DisplayIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.y_start == self.y_end && self.x_start > self.x_end {
            return None;
        }
        if self.x_end == 0 {
            self.x_end = self.display.x - 1;
            if self.y_end == 0 {
                self.x_end = 0;
                self.x_start = 1;
                return Some((0, 0));
            }
            self.y_end -= 1;
            return Some((0, self.y_end + 1));
        }
        if self.y_end < self.y_start {
            return None;
        }
        self.x_end -= 1;
        Some((self.x_end + 1, self.y_end))
    }
}

#[derive(Debug)]
pub struct Camera {
    pub xfov: f64,
    pub yfov: f64,
    pub pos: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
}

impl Camera {
    pub fn new(xfov: f64, yfov: f64, pos: Vec3, forward: Vec3, up: Vec3) -> Self {
        Self {
            xfov,
            yfov,
            pos,
            forward: forward.normalize(),
            up: up.normalize(),
        }
    }

    pub fn from_display(xfov: f64, display: Display, pos: Vec3, forward: Vec3, up: Vec3) -> Self {
        Self::new(
            xfov,
            (display.y as f64 / display.x as f64) * xfov,
            pos,
            forward,
            up,
        )
    }

    pub fn left(&self) -> Vec3 {
        self.up.cross(self.forward)
    }

    pub fn max_left_deflection(&self) -> Vec3 {
        let (facing_left_x, facing_left_z) = self.xfov.to_radians().sin_cos();
        let max_deflection = facing_left_z.recip() * facing_left_x;
        max_deflection * self.left()
    }

    pub fn max_up_deflection(&self) -> Vec3 {
        let (facing_top_y, facing_top_z) = self.yfov.to_radians().sin_cos();
        let up_deflection = facing_top_z.recip() * facing_top_y;
        up_deflection * self.up
    }
}

#[derive(Debug)]
pub struct Scene<S: Shape> {
    pub display: Display,
    pub camera: Camera,
    pub light_pos: Vec3,
    pub world: S,
}

impl<S: Shape> Scene<S> {
    pub fn brightness(&self, ray: Ray) -> f64 {
        let light_relative = self.light_pos - ray.start;
        let to_light_ray_dist = light_relative.normalize();
        ray.dir.dot(to_light_ray_dist).max(0.)
    }

    pub fn pixel_ray(&self, x: u32, y: u32) -> Ray {
        let x_percent = x as f64 / self.display.x as f64 - 0.5;
        let y_percent = y as f64 / self.display.y as f64 - 0.5;
        let dir = (self.camera.forward
            + x_percent * self.camera.max_left_deflection()
            + y_percent * self.camera.max_up_deflection())
        .normalize();
        Ray::new(self.camera.pos, dir)
    }

    pub fn sees_light(&self, pos: Vec3) -> bool {
        let light_relative = self.light_pos - pos;
        let to_light_ray = Ray::new_unit(pos, light_relative);
        !self
            .world
            .intersect_inclusive(to_light_ray)
            .is_some_and(|collision| {
                collision.distance.powi(2) - light_relative.squared_magnitude()
                    < EPSILON
            })
    }
}