#![allow(dead_code)]
use material::{ColorMaterial, UniformDiffuse};
use shapes::{ColorIndex, VertexIndex};
use std::{
    fs::{remove_file, File},
    io::{BufWriter, Write},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Instant,
};

use crate::{
    img::{
        writer::{ImageWriter, QOIWriter},
        Color, Image, PPMWriter,
    },
    material::Lambertian,
    math::Vec3,
    rand::thread_lcg,
    scene::{Camera, Display, Scene},
    shapes::{InvertedSphere, Shape, Sphere, TriangleMesh},
    utils::{CartesianProduct, RangeChunks},
};

mod img;
mod material;
mod math;
mod rand;
mod scene;
mod shapes;
pub mod utils;

pub const EPSILON: f64 = 1e-5;

fn main() {
    draw();
}

fn my_world() -> impl Shape + Send + Sync {
    type Reflector = Lambertian;
    const REFLECTOR: Reflector = Lambertian;
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
    let triangles: Vec<([VertexIndex; 3], ColorIndex)> = vec![
        // far
        ([0b000, 0b100, 0b110], 0),
        ([0b000, 0b110, 0b010], 0),
        // right
        ([0b100, 0b101, 0b111], 0),
        ([0b100, 0b111, 0b110], 0),
        // near
        ([0b101, 0b001, 0b011], 0),
        ([0b101, 0b011, 0b111], 0),
        // left
        ([0b001, 0b000, 0b010], 0),
        ([0b001, 0b010, 0b011], 0),
        // top
        ([0b000, 0b001, 0b101], 0),
        ([0b000, 0b101, 0b100], 0),
        // bottom
        ([0b110, 0b111, 0b011], 0),
        ([0b110, 0b011, 0b010], 0),
    ];
    let cube = Box::new(TriangleMesh::new(
        vertices.clone(),
        vec![Color::from_rgb(0.6, 0.4, 0.3)],
        triangles.clone(),
        REFLECTOR,
    ));
    let outer = Box::new(TriangleMesh::new(
        vertices.into_iter().map(|v| v * 20.).collect(),
        vec![Color::WHITE],
        triangles
            .into_iter()
            .map(|([a, b, c], i)| ([c, b, a], i))
            .collect(),
        REFLECTOR,
    ));
    let _outer = Box::new(InvertedSphere::new(
        Vec3::ZERO,
        15.,
        Color::WHITE,
        REFLECTOR,
    ));
    let sphere = Box::new(Sphere::new(
        Vec3::new(0., 0., -0.8),
        1.2,
        Color::from_rgb(0.1, 0.1, 1.),
        REFLECTOR,
    ));
    let sphere2 = Box::new(Sphere::new(
        Vec3::new(-0.8, 1.2, 0.),
        0.3,
        Color::from_rgb(0.1, 1., 0.1),
        REFLECTOR,
    ));
    let v: Vec<Box<dyn Shape<Material = ColorMaterial<Reflector>> + Send + Sync>> =
        vec![cube, outer, sphere, sphere2];
    v
}

fn my_scene(display: Display) -> Scene<impl Shape + Send + Sync> {
    let cam_pos = Vec3::new(-7., 10., -10.);
    Scene {
        display,
        camera: Camera::from_display(
            60. / 2.,
            display,
            cam_pos,
            -cam_pos.normalize(),
            (Vec3::Y - cam_pos.normalize()).normalize(),
        ),
        light_pos: Vec3::new(-5., 8., 10.),
        world: my_world(),
        background_color: Color::from_rgb(0.0, 0.75, 1.),
    }
}

fn weekend_scene(display: Display) -> Scene<impl Shape + Send + Sync> {
    type Reflector = UniformDiffuse;
    const REFLECTOR: Reflector = UniformDiffuse;
    const WEEKEND_WORLD: [Sphere<Reflector>; 2] = [
        Sphere::new(Vec3::new(0., 0., -1.), 0.5, Color::gray(0.5), REFLECTOR),
        Sphere::new(
            Vec3::new(0., -100.5, -1.),
            100.,
            Color::gray(0.5),
            REFLECTOR,
        ),
    ];
    Scene {
        display,
        camera: Camera::new(
            74.29136217098426,
            63.43494882292201,
            Vec3::default(),
            Vec3::NEG_Z,
            Vec3::Y,
        ),
        light_pos: Vec3::new(0., 0., 0.),
        world: &WEEKEND_WORLD as &'static [Sphere<Reflector>],
        background_color: Color::from_rgb(0.5, 0.7, 1.),
    }
}

fn draw() {
    let display = Display { x: 1280, y: 720 };
    let scene = Arc::new(my_scene(display));
    let start_time = Instant::now();
    const SAMPLES: usize = 100;
    const BOUNCES: u16 = 50;
    const THREADS: usize = 16;
    let mut handles = Vec::with_capacity(THREADS);
    let mut main_img = Image::zeros(display);
    let x_chunk_iter = RangeChunks::new(0..display.x(), (display.x() + 1) / THREADS + 1);
    let y_chunk_iter = RangeChunks::new(0..display.y(), (display.y() + 1) / THREADS + 1);
    let chunks_iter =
        CartesianProduct::new(x_chunk_iter, y_chunk_iter).map(|(x, y)| CartesianProduct::new(x, y));
    let (_, len) = chunks_iter.size_hint();
    let chunks_iter = Arc::new(Mutex::new(chunks_iter));
    let progress = Arc::new(AtomicUsize::new(0));
    for _thread_idx in 0..THREADS {
        let scene = Arc::clone(&scene);
        let chunks_iter = Arc::clone(&chunks_iter);
        let progress = Arc::clone(&progress);
        handles.push(thread::spawn(move || {
            let mut img = Image::zeros(display);
            while let Ok(Some(chunk)) = chunks_iter.lock().map(|mut c| c.next()) {
                for (x, y) in chunk {
                    let color = (0..SAMPLES)
                        .map(|_| {
                            let x_offset = thread_lcg::<f64>();
                            let y_offset = thread_lcg::<f64>();
                            let ray = scene.pixel_ray(x as f64 + x_offset, y as f64 + y_offset);
                            scene.cast_ray(ray.clone(), BOUNCES)
                        })
                        .fold(Vec3::default(), |s, v| s + v.0);
                    let pixel = img.at_mut(x as usize, y as usize);
                    *pixel = (color / SAMPLES as f64).into();
                }
                progress.fetch_add(1, Ordering::Release);
            }
            img
        }));
    }
    while !handles.iter().all(JoinHandle::is_finished) {
        thread::sleep(std::time::Duration::from_millis(10));
        print_progress(len, progress.load(Ordering::Acquire), start_time);
    }
    for handle in handles {
        let img = handle.join().expect("Threads should not panic");
        for (main, thread) in main_img.data_mut().iter_mut().zip(img.data()) {
            if main.0 == Color::BLACK.0 {
                *main = *thread;
            }
        }
    }
    let brightest = main_img.data().iter().fold((0., 0., 0.), |c1, c2| {
        (c2.r().max(c1.0), c2.g().max(c1.1), c2.b().max(c1.2))
    });
    let brightest = brightest.0.max(brightest.1).max(brightest.2);
    for color in main_img.data_mut() {
        *color = color_correction((color.0 / brightest).into())
    }
    println!(
        "\nFinished rendering in {:.3?}",
        Instant::now().duration_since(start_time)
    );
    const FILE_STEM: &'static str = "img";
    let writer = QOIWriter::from(&main_img);
    // let writer = PPMWriter::from(&main_img);
    let file_name = format!("{}.{}", FILE_STEM, writer.extension().unwrap());
    let _ = remove_file(&file_name);
    let mut file = BufWriter::new(File::create(&file_name).unwrap());
    writer
        .write_to(&mut file)
        .expect("Expected writing to succeed");
}

fn color_correction(input: Color) -> Color {
    Vec3::new(input.0.x.sqrt(), input.0.y.sqrt(), input.0.z.sqrt()).into()
}

fn print_progress(len: Option<usize>, progress: usize, start_time: Instant) {
    if let Some(len) = len {
        print!(
            "Progress in {:.2?}: {}/{} ({:.2}%)            \r",
            Instant::now().duration_since(start_time),
            progress,
            len,
            progress as f64 / len as f64 * 100.
        );
    } else {
        print!(
            "Progress in {:.2?}: {}              \r",
            Instant::now().duration_since(start_time),
            progress
        );
    }
    let _ = std::io::stdout().flush();
}
