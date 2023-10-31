// #![allow(unused)]
use std::{
    fs::{remove_file, File},
    io::{self, BufWriter, Write},
};

use crate::{
    math::Vec3,
    shapes::{Shape, Sphere, TriangleMesh},
};

mod math;
mod shapes;

type Float = f32;
// pub const EPSILON: Float = Float::EPSILON;
pub const EPSILON: Float = 1e-5;

fn main() {
    draw();
}

fn draw() {
    // const RESOLUTION: (i32, i32) = (10, 10);
    const RESOLUTION: (i32, i32) = (1280, 720);
    const CAM_XFOV: Float = 50. / 2.;
    const CAM_YFOV: Float = (RESOLUTION.1 as Float / RESOLUTION.0 as Float) * CAM_XFOV;
    const CAM_POS: Vec3 = Vec3::new(-10., 10., -10.);
    let cam_fwd = Vec3::new(1., -1., 1.).normalize();
    let cam_up = Vec3::new(1., 1., 1.).normalize();

    let light_pos = Vec3::new(-5., 8., 10.);

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
    let tri_mesh = TriangleMesh::new(vertices, triangles);
    let sphere = Sphere::new(Vec3::new(0., 0., -0.8), 1.2);
    let mesh: Vec<&dyn Shape> = vec![&tri_mesh, &sphere];

    let _ = remove_file("img.ppm");
    let mut file = BufWriter::new(File::create("img.ppm").unwrap());
    file.write(format!("P3\n{} {}\n255\n", RESOLUTION.0, RESOLUTION.1).as_bytes())
        .unwrap();

    let (facing_left_x, facing_left_z) = CAM_XFOV.to_radians().sin_cos();
    let max_left_deflection = facing_left_z.recip() * facing_left_x;
    let cam_left = cam_up.cross(cam_fwd);
    let cam_left_deflection = max_left_deflection * cam_left;

    let (facing_top_y, facing_top_z) = CAM_YFOV.to_radians().sin_cos();
    let max_up_deflection = facing_top_z.recip() * facing_top_y;
    let cam_up_deflection = max_up_deflection * cam_up;

    let mut has_sight = 0;
    let mut no_sight = 0;
    for y in (0..RESOLUTION.1).rev() {
        let y_percent = y as Float / RESOLUTION.1 as Float - 0.5;
        for x in (0..RESOLUTION.0).rev() {
            let x_percent = x as Float / RESOLUTION.0 as Float - 0.5;
            let ray = (cam_fwd + x_percent * cam_left_deflection + y_percent * cam_up_deflection)
                .normalize();
            let hit = mesh.ray_intersection(CAM_POS, ray, false);
            if let Some(collision) = hit {
                let light_relative = light_pos - collision.position;
                let to_light_ray = light_relative.normalize();
                let has_line_of_sight = !mesh
                    .ray_intersection(collision.position, to_light_ray, true)
                    .is_some_and(|light_collision| {
                        (light_collision.position - collision.position).squared_magnitude()
                            - light_relative.squared_magnitude()
                            < EPSILON
                    });
                if has_line_of_sight {
                    has_sight += 1;
                } else {
                    no_sight += 1;
                }
                let attenuation = (collision.position - light_pos).squared_magnitude().recip();
                let reflected = ray.reflect_across(collision.normal);
                let brightness = reflected.dot(to_light_ray).max(0.); // Look here for specular-ness
                let color = Vec3::splat(
                    brightness * attenuation * 100. * has_line_of_sight as i32 as Float,
                );
                write_percent_vec(&mut file, color + Vec3::splat(0.1), 0., 1.).unwrap();
                // write_percent_vec(&mut file, collision.normal, -1., 1.).unwrap(); // normals
                // write_percent_vec(&mut file, Vec3::splat(((CAM_POS - collision.position).magnitude() - 15.) / 3.), 0., 1.).unwrap(); // distance?
            } else {
                write_percent_vec(&mut file, Vec3::splat(0.), 0., 1.).unwrap();
            }
        }
        file.write(b"\n").unwrap();
    }
    dbg!(has_sight, no_sight);
}

fn write_percent_vec(
    file: &mut BufWriter<File>,
    vec: Vec3,
    min: Float,
    max: Float,
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

fn to_percent_byte(x: Float, min: Float, max: Float) -> u8 {
    (x.clamp(min, max) * 255.).floor() as u8
}
