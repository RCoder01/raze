// #![allow(unused)]
use std::{
    fs::{remove_file, File},
    io::{BufWriter, Write, self},
};

use crate::math::Vec3;

mod math;

fn main() {
    const RESOLUTION: (i32, i32) = (1280, 720);
    const CAM_XFOV: f32 = 50. / 2.;
    const CAM_YFOV: f32 = (RESOLUTION.1 as f32 / RESOLUTION.0 as f32) * CAM_XFOV;
    // const CAM_POS: Vec3 = Vec3::new(0., 0., -10.);
    // let cam_fwd = Vec3::new(0., 0., 1.).normalize();
    // let cam_up = Vec3::new(0., 1., 0.).normalize();
    const CAM_POS: Vec3 = Vec3::new(-10., 10., -10.);
    let cam_fwd = Vec3::new(1., -1., 1.).normalize();
    let cam_up = Vec3::new(1., 1., 1.).normalize();

    let light_pos = Vec3::new(-5., 8., 10.) - CAM_POS;

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
    let cam_left = cam_up.cross(cam_fwd);
    let cam_left_deflection = max_left_deflection * cam_left;

    let (facing_top_y, facing_top_z) = CAM_YFOV.to_radians().sin_cos();
    let max_up_deflection = facing_top_z.recip() * facing_top_y;
    let cam_up_deflection = max_up_deflection * cam_up;

    let camera_space_vertices: Vec<_> = vertices.iter().map(|vertex| *vertex - CAM_POS).collect();

    for y in (0..RESOLUTION.1).rev() {
        let y_percent = y as f32 / RESOLUTION.1 as f32 - 0.5;
        for x in (0..RESOLUTION.0).rev() {
            let x_percent = x as f32 / RESOLUTION.0 as f32 - 0.5;
            let ray = (cam_fwd + x_percent * cam_left_deflection + y_percent * cam_up_deflection)
                .normalize();
            let hit = triangles
                .iter()
                .copied()
                .zip(normals.iter().copied())
                .enumerate()
                .filter_map(|(i, ([a, b, c], normal))| {
                    if ray.dot(normal) > -1e-5 {
                        // ray and normal are roughly the same direction
                        return None;
                    }

                    let vertices = [
                        camera_space_vertices[a as usize],
                        camera_space_vertices[b as usize],
                        camera_space_vertices[c as usize],
                    ];
                    let normals = [
                        vertices[1].cross(vertices[0]),
                        vertices[2].cross(vertices[1]),
                        vertices[0].cross(vertices[2]),
                    ];

                    let intersects = normals.iter().all(|normal| normal.dot(ray) > -1e-5);
                    if !intersects {
                        return None;
                    }
                    let distance = (vertices[0]).project_onto(normal);
                    let cam_space_intersect_point =
                        (ray.dot(distance) / distance.squared_magnitude()).recip() * ray;
                    Some((i, cam_space_intersect_point))
                })
                .min_by(|(_, v1), (_, v2)| {
                    v1.squared_magnitude().total_cmp(&v2.squared_magnitude())
                });
            if let Some((triangle_index, point)) = hit {
                let attenuation = (point - light_pos).squared_magnitude().recip();
                let reflected = ray.reflect_across(normals[triangle_index]);
                let brightness = reflected.dot((light_pos - point).normalize()).max(0.); // Look here for specular-ness
                let color = Vec3::splat(brightness * attenuation * 100.);
                // println!("{} {:.3}\t{:.3}\t{:.3}", triangle_index, color.x, attenuation, brightness);
                write_percent_vec(&mut file, color + Vec3::splat(0.1)).unwrap();
                // write_percent_vec_neg(&mut file, normals[triangle_index]).unwrap();
            } else {
                write_percent_vec(&mut file, Vec3::splat(0.)).unwrap();
                // write_percent_vec_neg(&mut file, Vec3::new(x_percent, 0., y_percent)).unwrap();
            }
        }
        file.write(b"\n").unwrap();
    }
}

fn write_percent_vec_neg(file: &mut BufWriter<File>, vec: Vec3) -> Result<usize, io::Error> {
    file.write(
        format!(
            "{} {} {} ",
            to_percent_byte_neg(vec.x),
            to_percent_byte_neg(vec.y),
            to_percent_byte_neg(vec.z),
        )
        .as_bytes(),
    )
}

fn write_percent_vec(file: &mut BufWriter<File>, vec: Vec3) -> Result<usize, io::Error> {
    file.write(
        format!(
            "{} {} {} ",
            to_percent_byte(vec.x),
            to_percent_byte(vec.y),
            to_percent_byte(vec.z),
        )
        .as_bytes(),
    )
}

fn to_percent_byte(x: f32) -> u8 {
    (x.clamp(0., 1.) * 255.).floor() as u8
}

fn to_percent_byte_neg(x: f32) -> u8 {
    ((x.clamp(-1., 1.) + 1.) * 127.).floor() as u8
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
