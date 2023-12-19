// #![allow(unused)]
use std::{
    fs::{remove_file, File},
    io::{self, BufWriter, Write},
    time::Instant,
};

use crate::{
    math::{Ray, Vec3},
    rand::Reflector,
    scene::{Camera, Display, Scene},
    shapes::{InvertedSphere, Shape, Sphere, TriangleMesh},
};

mod math;
mod rand;
mod scene;
mod shapes;

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
    let outer = Box::new(TriangleMesh::new(
        vertices.into_iter().map(|v| v * 20.).collect(),
        triangles.into_iter().map(|[a, b, c]| [c, b, a]).collect(),
    ));
    // let outer = Box::new(InvertedSphere::new(Vec3::ZERO, 15.));
    let sphere = Box::new(Sphere::new(Vec3::new(0., 0., -0.8), 1.2));
    let sphere2 = Box::new(Sphere::new(Vec3::new(-0.8, 1.2, 0.), 0.3));
    vec![cube, outer, sphere, sphere2]
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

    let _ = remove_file("img.ppm");
    let mut file = BufWriter::new(File::create("img.ppm").unwrap());
    file.write_all(format!("P3\n{} {}\n255\n", display.x, display.y).as_bytes())
        .unwrap();

    let start_time = Instant::now();
    let mut rand = Reflector::new();
    const SAMPLES: usize = 50;
    for (i, (x, y)) in display.into_iter().rev().enumerate() {
        if i % 10000 == 0 {
            let total_pixels = display.x * display.y;
            print!(
                "\r{}/{} ({:.1}%)",
                i,
                total_pixels,
                i as f64 * 100. / total_pixels as f64
            );
            let _ = std::io::stdout().flush();
        }
        let ray = scene.pixel_ray(x, y);
        let Some(collision) = scene.world.intersect_exclusive(ray.clone()) else {
            write_percent_vec(&mut file, Vec3::splat(0.), 0., 1.).unwrap();
            continue;
        };
        let mut brightness = 0.;
        let curr_dist = (ray.start - collision.position).magnitude();
        for _ in 0..SAMPLES {
            let new_ray = Ray::new(collision.position, rand.random_diffuse(collision.normal));
            brightness += if let Some(collision) = scene.world.intersect_exclusive(new_ray.clone())
            {
                let bounce_has_los = scene.sees_light(collision.position);
                scene.brightness(collision.outgoing_ray(new_ray.dir))
                    * bounce_has_los as i32 as f64
                    * scene.sees_light(collision.position) as i32 as f64
                    * (curr_dist
                        + (new_ray.start - collision.position).magnitude()
                        + (collision.position - scene.light_pos).magnitude())
                    .powi(-2)
                    * 700.
            } else {
                0.
            };
        }
        brightness /= SAMPLES as f64;
        // let brightness = scene.brightness(collision.outgoing_ray(ray.dir))
        //     * scene.sees_light(collision.position) as i32 as f64
        //     * 100.;
        write_percent_vec(&mut file, Vec3::splat(brightness), 0., 1.).unwrap();
        // if has_line_of_sight {
        // } else {
        //     write_percent_vec(&mut file, Vec3::splat(1.), 0., 1.).unwrap();
        // }
        // dbg!(collision.normal);
        // write_percent_vec(&mut file, collision.normal, -1., 1.).unwrap(); // normals
        // write_percent_vec(&mut file, Vec3::splat(((CAM_POS - collision.position).magnitude() - 15.) / 3.), 0., 1.).unwrap(); // distance?
        if x == 0 {
            file.write_all(b"\n").unwrap();
        }
    }
    dbg!(Instant::now().duration_since(start_time));
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
    (((x.clamp(min, max) - min) / (max - min)) * 255.).floor() as u8
}
