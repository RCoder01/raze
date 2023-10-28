// #![allow(unused)]
use std::{
    fs::{remove_file, File},
    io::{BufWriter, Write},
};

use crate::math::Vec3;

mod math;

#[derive(Debug, Clone, Copy)]
enum Color {
    Red = 0,
    Green = 1,
    Blue = 2,
}

const COLORS: [Color; 3] = [Color::Red, Color::Green, Color::Blue];

fn main() {
    println!("Hello, world!");

    const RESOLUTION: (i32, i32) = (192, 108);
    const CAM_XFOV: f32 = 80. / 2.;
    const CAM_YFOV: f32 = (RESOLUTION.1 as f32 / RESOLUTION.0 as f32) * CAM_XFOV;
    const CAM_POS: Vec3 = Vec3::new(0., 0., -10.);
    const CAM_FWD: Vec3 = Vec3::new(0., 0., 1.);
    const CAM_UP: Vec3 = Vec3::new(0., 1., 0.);

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
        [0b000, 0b100, 0b110],
        [0b000, 0b110, 0b010],
        [0b100, 0b101, 0b111],
        [0b100, 0b111, 0b110],
        [0b101, 0b001, 0b011],
        [0b101, 0b011, 0b111],
        [0b001, 0b000, 0b010],
        [0b001, 0b010, 0b011],
        [0b000, 0b001, 0b101],
        [0b000, 0b101, 0b100],
        [0b110, 0b111, 0b011],
        [0b110, 0b011, 0b010],
    ];
    let normals: Vec<_> = triangles
        .iter()
        .copied()
        .map(|[a, b, c]| {
            (vertices[b as usize] - vertices[a as usize])
                .cross(vertices[c as usize] - vertices[b as usize])
                .normalize()
        })
        .collect();

    let _ = remove_file("img.ppm");
    let mut file = BufWriter::new(File::create("img.ppm").unwrap());
    file.write(b"P3\n").unwrap();
    file.write(format!("{} {}\n", RESOLUTION.0, RESOLUTION.1).as_bytes())
        .unwrap();
    file.write(b"255\n").unwrap();

    let (facing_left_x, facing_left_z) = CAM_XFOV.to_radians().sin_cos();
    let max_left_deflection = facing_left_z.recip() * facing_left_x;
    let cam_left = CAM_UP.cross(CAM_FWD);
    let cam_left_deflection = max_left_deflection * cam_left;

    let (facing_top_y, facing_top_z) = CAM_YFOV.to_radians().sin_cos();
    let max_up_deflection = facing_top_z.recip() * facing_top_y;
    let cam_up_deflection = max_up_deflection * CAM_UP;
    dbg!(cam_left_deflection, cam_up_deflection);

    for y in (0..RESOLUTION.1).rev() {
        let y_percent = y as f32 / RESOLUTION.1 as f32 - 0.5;
        for x in (0..RESOLUTION.0).rev() {
            let x_percent = x as f32 / RESOLUTION.0 as f32 - 0.5;
            let ray = (CAM_FWD + x_percent * cam_left_deflection + y_percent * cam_up_deflection).normalize();
            for ([a, b, c], normal) in triangles.iter().copied().zip(normals.iter().copied()) {
                let distance = (vertices[a as usize] - CAM_POS).project_onto(normal);
                let point = (ray.dot(distance) / distance.squared_magnitude()).recip() * ray + CAM_POS;
            }
            file.write(format!("{} ", (x_percent * 255.).floor() as i32).as_bytes())
                .unwrap();
            file.write(b"0 ").unwrap();
            file.write(format!("{} ", (y_percent * 255.).floor() as i32).as_bytes())
                .unwrap();
        }
        file.write(b"\n").unwrap();
    }
}

fn triangle_contains_point(
    vertices: &Vec<Vec3>,
    triangle: &[u16; 3],
    normal: Vec3,
    point: Vec3,
) -> bool {
    let min_dot = (0..3)
        .into_iter()
        .map(|b| {
            let a = (b + 1) % 3;
            let edge = vertices[triangle[a] as usize] - vertices[triangle[b] as usize];
            let cross = normal.cross(edge);
            cross.dot(point)
        })
        .reduce(|a, b| a.min(b))
        .unwrap();
    !min_dot.is_sign_negative()
}

fn triangle_area(vertices: [Vec3; 3]) -> f32 {
    let edges = [
        vertices[1] - vertices[0],
        vertices[2] - vertices[1],
        vertices[0] - vertices[2],
    ];
    dbg!(edges);

    let sides = [
        edges[0].squared_magnitude(),
        edges[1].squared_magnitude(),
        edges[2].squared_magnitude(),
    ];

    (2. * (sides[0] * sides[1] + sides[1] * sides[2] + sides[2] * sides[0])
        - (sides[0].powi(2) + sides[1].powi(2) + sides[2].powi(2)))
        .sqrt()
        / 4.
}
