// #![allow(unused)]
use std::{
    fs::{remove_file, File},
    io::{self, BufWriter, Write},
    ops::{Mul, Range},
    time::Instant,
};

use crate::{
    math::{Mat3x3, Ray, Vec3},
    rand::Lcg,
    shapes::{Shape, Sphere, TriangleMesh},
};

mod math;
mod rand;
mod shapes;

// pub const EPSILON: f64 = f64::EPSILON;
pub const EPSILON: f64 = 1e-5;

fn main() {
    draw();
}

fn get_geometry() -> Vec<Box<dyn Shape>> {
    let vertices = vec![
        Vec3::new(1., 1., 1.),
        Vec3::new(1., 1., -1.),
        Vec3::new(1., -1., 1.),
        Vec3::new(1., -1., -1.),
        Vec3::new(-1., 1., 1.),
        Vec3::new(-1., 1., -1.),
        Vec3::new(-1., -1., 1.),
        Vec3::new(-1., -1., -1.),
    ];
    let triangles: Vec<[u16; 3]> = vec![
        // far
        [0b000, 0b100, 0b110],
        [0b000, 0b110, 0b010],
        // right
        [0b100, 0b101, 0b111],
        [0b100, 0b111, 0b110],
        // near
        [0b101, 0b001, 0b011],
        [0b101, 0b011, 0b111],
        // left
        [0b001, 0b000, 0b010],
        [0b001, 0b010, 0b011],
        // top
        [0b000, 0b001, 0b101],
        [0b000, 0b101, 0b100],
        // bottom
        [0b110, 0b111, 0b011],
        [0b110, 0b011, 0b010],
    ];
    let cube = Box::new(TriangleMesh::new(vertices.clone(), triangles.clone()));
    let big_cube = Box::new(TriangleMesh::new(
        vertices.into_iter().map(|v| v * 20.).collect(),
        triangles.into_iter().map(|[a, b, c]| [c, b, a]).collect(),
    ));
    let sphere = Box::new(Sphere::new(Vec3::new(0., 0., -0.8), 1.2));
    let sphere2 = Box::new(Sphere::new(Vec3::new(-0.8, 1.2, 0.), 0.3));
    vec![cube, big_cube, sphere, sphere2]
}

fn draw() {
    let display = Display::new(1280, 720);
    let cam_pos = Vec3::new(-7., 10., -10.);
    let camera = Camera::from_display(
        60. / 2.,
        display,
        cam_pos,
        -cam_pos.normalize(),
        (Vec3::Y - cam_pos.normalize()).normalize(),
    );

    let light_pos = Vec3::new(-5., 8., 10.);
    let mesh = get_geometry();

    let _ = remove_file("img.ppm");
    let mut file = BufWriter::new(File::create("img.ppm").unwrap());
    file.write_all(format!("P3\n{} {}\n255\n", display.x, display.y).as_bytes())
        .unwrap();

    let max_left_deflection = camera.max_left_deflection();
    let max_up_deflection = camera.max_up_deflection();

    let start_time = Instant::now();
    for (x, y) in display.into_iter().rev() {
        let x_percent = x as f64 / display.x as f64 - 0.5;
        let y_percent = y as f64 / display.y as f64 - 0.5;
        let ray_dir =
            (camera.forward + x_percent * max_left_deflection + y_percent * max_up_deflection)
                .normalize();
        let ray = Ray::new(camera.pos, ray_dir);
        let Some(collision) = mesh.ray_intersection(ray, false) else {
            write_percent_vec(&mut file, Vec3::splat(0.), 0., 1.).unwrap();
            continue;
        };
        let light_relative = light_pos - collision.position;
        let to_light_ray_dist = light_relative.normalize();
        let to_light_ray = Ray::new(collision.position, to_light_ray_dist);
        let has_line_of_sight =
            !mesh
                .ray_intersection(to_light_ray, true)
                .is_some_and(|light_collision| {
                    (light_collision.position - collision.position).squared_magnitude()
                        - light_relative.squared_magnitude()
                        < EPSILON
                });
        let attenuation = (collision.position - light_pos).squared_magnitude().recip();
        let reflected = ray_dir.reflect_across(collision.normal);
        let brightness = reflected.dot(to_light_ray_dist).max(0.); // Look here for specular-ness
        let color = Vec3::splat(brightness * attenuation * 100. * has_line_of_sight as i32 as f64);
        write_percent_vec(&mut file, color + Vec3::splat(0.1), 0., 1.).unwrap();
        // if has_line_of_sight {
        // } else {
        //     write_percent_vec(&mut file, Vec3::splat(1.), 0., 1.).unwrap();
        // }
        // write_percent_vec(&mut file, collision.normal, -1., 1.).unwrap(); // normals
        // write_percent_vec(&mut file, Vec3::splat(((CAM_POS - collision.position).magnitude() - 15.) / 3.), 0., 1.).unwrap(); // distance?
        file.write_all(b"\n").unwrap();
    }
    dbg!(Instant::now().duration_since(start_time));
}

#[derive(Debug, Clone, Copy)]
struct Display {
    x: u32,
    y: u32,
}

impl Display {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl IntoIterator for Display {
    type Item = (u32, u32);
    type IntoIter = DisplayIter;

    fn into_iter(self) -> Self::IntoIter {
        DisplayIter::new(self)
    }
}

#[derive(Debug, Clone, Copy)]
struct DisplayIter {
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
struct Camera {
    xfov: f64,
    yfov: f64,
    pos: Vec3,
    forward: Vec3,
    up: Vec3,
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

fn write_percent_vec(
    file: &mut BufWriter<File>,
    vec: Vec3,
    min: f64,
    max: f64,
) -> Result<usize, io::Error> {
    file.write(
        format!(
            "{} {} {} ",
            to_percent_byte(vec.x, min, max),
            to_percent_byte(vec.y, min, max),
            to_percent_byte(vec.z, min, max),
        )
        .as_bytes(),
    )
}

fn to_percent_byte(x: f64, min: f64, max: f64) -> u8 {
    (x.clamp(min, max) * 255.).floor() as u8
}

#[derive(Debug, Clone)]
struct Reflector {
    random: Lcg,
}

impl Reflector {
    pub fn new() -> Self {
        Self {
            random: Lcg::from_time(),
        }
    }

    fn random_unit_y(&mut self) -> Vec3 {
        let dir = self.random.pseudo_rand_f64() * std::f64::consts::TAU;
        let height = self.random.pseudo_rand_f64();
        let (sin, cos) = dir.sin_cos();
        let xz = Vec3::new(cos, 0., sin);
        (1. - height.powi(2)).sqrt() * xz + height * Vec3::Y
    }

    pub fn random_diffuse(&mut self, normal: Vec3) -> Vec3 {
        let orthogonal_1 = if normal.x.abs() + EPSILON >= 1. {
            Vec3::Y
        } else {
            Vec3::X
        }
        .cross(normal);
        let orthogonal_2 = normal.cross(orthogonal_1);
        // dbg!(normal, orthogonal_1, orthogonal_2);
        &Mat3x3::from_col_vectors(orthogonal_1, normal, orthogonal_2) * self.random_unit_y()
    }
}
