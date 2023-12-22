#![allow(dead_code)]
use std::{
    fs::{remove_file, File},
    io::{BufWriter, Write},
    time::Instant, sync::atomic::AtomicBool,
};

use crate::{
    img::{Image, PPMWriter},
    math::Vec3,
    rand::Reflector,
    scene::{Camera, Display, Scene},
    shapes::{InvertedSphere, Shape, Sphere, TriangleMesh},
};

mod img;
mod math;
mod rand;
mod scene;
mod shapes;

pub const EPSILON: f64 = 1e-5;

fn main() {
    draw();
}

fn get_geometry() -> impl Shape {
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
    let outer = Box::new(TriangleMesh::new(
        vertices.into_iter().map(|v| v * 20.).collect(),
        triangles.into_iter().map(|[a, b, c]| [c, b, a]).collect(),
    ));
    let _outer = Box::new(InvertedSphere::new(Vec3::ZERO, 15.));
    let sphere = Box::new(Sphere::new(Vec3::new(0., 0., -0.8), 1.2));
    let sphere2 = Box::new(Sphere::new(Vec3::new(-0.8, 1.2, 0.), 0.3));
    let v: Vec<Box<dyn Shape>> = vec![cube, outer, sphere, sphere2];
    v
}

fn draw() {
    let display = Display { x: 12, y: 7 };
    let cam_pos = Vec3::new(-7., 10., -10.);
    let scene = Scene {
        display,
        camera: Camera::from_display(
            60. / 2.,
            display,
            cam_pos,
            -cam_pos.normalize(),
            (Vec3::Y - cam_pos.normalize()).normalize(),
        ),
        light_pos: Vec3::new(-5., 8., 10.),
        world: get_geometry(),
    };
    let mut img = Image::zeros(display);

    let start_time = Instant::now();
    let mut rand = Reflector::new();
    const SAMPLES: usize = 50;
    for (i, (x, y)) in display.into_iter().rev().enumerate() {
        if i % 10000 == 0 {
            let total_pixels = display.x * display.y;
            print!(
                "{}/{} ({:.1}%)\r",
                i,
                total_pixels,
                i as f64 * 100. / total_pixels as f64
            );
            let _ = std::io::stdout().flush();
        }
        let x_offset = rand.random.pseudo_rand_f64();
        let y_offset = rand.random.pseudo_rand_f64();
        let ray = scene.pixel_ray(x as f64 + x_offset, y as f64 + y_offset);
        let color = (0..SAMPLES)
            .map(|_| scene.cast_ray(&mut rand, ray.clone(), 2))
            .fold(Vec3::default(), |s, v| s + v.0);
        let pixel = img.at_mut(img.width() - x as usize - 1, y as usize);
        *pixel = (color / SAMPLES as f64).into();
    }
    let brightest = img.data().into_iter().fold((0., 0., 0.), |c1, c2| {
        (c2.r().max(c1.0), c2.g().max(c1.1), c2.b().max(c1.2))
    });
    let brightest = brightest.0.max(brightest.1).max(brightest.2);
    for color in img.data_mut() {
        *color = (color.0 / brightest).into()
    }
    dbg!(Instant::now().duration_since(start_time));
    let _ = remove_file("img.ppm");
    let mut file = BufWriter::new(File::create("img.ppm").unwrap());
    write!(&mut file, "{}", PPMWriter::from(&img)).expect("Expected writing to succeed");
}
