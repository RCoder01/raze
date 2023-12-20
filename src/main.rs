// #![allow(unused)]
use std::{
    fs::{remove_file, File},
    io::{BufWriter, Write},
    time::Instant,
};

use crate::{
    img::{Color, Image, PPMWriter},
    math::{Ray, Vec3},
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
    let display = Display { x: 1280, y: 720 };
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
        let ray = scene.pixel_ray(x, y);
        let Some(collision) = scene.world.intersect_exclusive(ray.clone()) else {
            continue;
        };
        let mut brightness = 0.;
        let curr_dist = collision.distance;
        for _ in 0..SAMPLES {
            let new_ray = Ray::new(collision.position(), rand.random_diffuse(collision.normal));
            brightness += if let Some(collision) = scene.world.intersect_exclusive(new_ray.clone())
            {
                let bounce_has_los = scene.sees_light(collision.position());
                scene.brightness(collision.reflection())
                    * bounce_has_los as i32 as f64
                    * scene.sees_light(collision.position()) as i32 as f64
                    * (curr_dist
                        + (new_ray.start - collision.position()).magnitude()
                        + (collision.position() - scene.light_pos).magnitude())
                    .powi(-2)
                    * 700.
            } else {
                0.
            };
        }
        brightness /= SAMPLES as f64;
        // let brightness = scene.brightness(collision.reflection())
        //     * scene.sees_light(collision.position()) as i32 as f64
        //     * 100.;
        let color = Color::gray(brightness);
        let pixel = img.at_mut(img.width() - x as usize - 1, y as usize);
        *pixel = color;
        // if has_line_of_sight {
        // } else {
        //     write_percent_vec(&mut file, Vec3::splat(1.), 0., 1.).unwrap();
        // }
        // *pixel = (collision.normal + Vec3::splat(1.) * 0.5).into();
    }
    dbg!(Instant::now().duration_since(start_time));
    let _ = remove_file("img.ppm");
    let mut file = BufWriter::new(File::create("img.ppm").unwrap());
    write!(&mut file, "{}", PPMWriter::from(&img)).expect("Expected writing to succeed");
}
