#![allow(dead_code)]
use std::{
    fs::{remove_file, File},
    io::{BufWriter, Write},
    time::{Instant, Duration}, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc, Mutex}, thread::{self, yield_now},
};

use crate::{
    img::{Image, PPMWriter, Color},
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

fn get_geometry() -> impl Shape + Send + Sync {
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
    let v: Vec<Box<dyn Shape + Send + Sync>> = vec![cube, outer, sphere, sphere2];
    v
}

fn draw() {
    let display = Display { x: 1280, y: 720 };
    let cam_pos = Vec3::new(-7., 10., -10.);
    let scene = Arc::new(Scene {
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
    });
    let start_time = Instant::now();
    const SAMPLES: usize = 200;
    const NUM_THREADS: usize = 16;
    let len = display.size();
    let mut it = display.into_iter();
    let per_thread = (len + 1) / NUM_THREADS;
    let progress_changed = Arc::new(AtomicBool::new(true));
    let progress = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::with_capacity(NUM_THREADS);
    let mut main_img = Image::zeros(display);
    for _i in 0..NUM_THREADS {
        let thread_it = it.clone().take(per_thread);
        let _ = it.nth(per_thread);
        let progress = Arc::clone(&progress);
        let progress_changed = Arc::clone(&progress_changed);
        let scene = Arc::clone(&scene);
        handles.push(thread::spawn(move || {
            let mut rand = Reflector::new();
            let mut img = Image::zeros(display);
            for (i, (x, y)) in thread_it.enumerate() {
                if i > 0 && i % 10000 == 0 {
                    progress.fetch_add(10000, Ordering::Release);
                    progress_changed.store(true, Ordering::Release);
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
            progress_changed.store(true, Ordering::Release);
            img
        }));
    }
    for handle in handles {
        while !handle.is_finished() {
            if progress_changed.load(Ordering::Acquire) {
                progress_changed.store(false, Ordering::Release);
                let progress = progress.load(Ordering::Acquire);
                print!("Progress: {}/{} ({:.2}%)\r", progress, len, progress as f64 / len as f64 * 100.);
                let _ = std::io::stdout().flush();
                thread::sleep(Duration::from_millis(100));
            }
        }
        let img = handle.join().expect("Threads should not panic");
        for (main, thread) in main_img.data_mut().iter_mut().zip(img.data()) {
            if main.0 == Color::BLACK.0 {
                *main = *thread;
            }
        }
    }
    // let mut img = main_img.into_inner().expect("All threads should have finished");
    let brightest = main_img.data().into_iter().fold((0., 0., 0.), |c1, c2| {
        (c2.r().max(c1.0), c2.g().max(c1.1), c2.b().max(c1.2))
    });
    let brightest = brightest.0.max(brightest.1).max(brightest.2);
    for color in main_img.data_mut() {
        *color = (color.0 / brightest).into()
    }
    dbg!(Instant::now().duration_since(start_time));
    let _ = remove_file("img.ppm");
    let mut file = BufWriter::new(File::create("img.ppm").unwrap());
    write!(&mut file, "{}", PPMWriter::from(&main_img)).expect("Expected writing to succeed");
}
