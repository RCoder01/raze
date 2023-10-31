// #![allow(unused)]
use std::{
    cmp::Ordering,
    fs::{remove_file, File},
    io::{self, BufWriter, Write},
    ops::Deref,
};

use crate::math::Vec3;

mod math;

fn main() {
    // const RESOLUTION: (i32, i32) = (10, 10);
    const RESOLUTION: (i32, i32) = (1280, 720);
    const CAM_XFOV: f32 = 50. / 2.;
    const CAM_YFOV: f32 = (RESOLUTION.1 as f32 / RESOLUTION.0 as f32) * CAM_XFOV;
    const CAM_POS: Vec3 = Vec3::new(-10., 10., -10.);
    let cam_fwd = Vec3::new(1., -1., 1.).normalize();
    let cam_up = Vec3::new(1., 1., 1.).normalize();

    let light_pos = Vec3::new(-5., 8., 10.);
    // let light_pos = Vec3::splat(10.);

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
    let sphere = Sphere::new(Vec3::splat(0.), 1.2);
    let mesh: Vec<&dyn Shape> = vec![&tri_mesh, &sphere];

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

    for y in (0..RESOLUTION.1).rev() {
        let y_percent = y as f32 / RESOLUTION.1 as f32 - 0.5;
        for x in (0..RESOLUTION.0).rev() {
            let x_percent = x as f32 / RESOLUTION.0 as f32 - 0.5;
            let ray = (cam_fwd + x_percent * cam_left_deflection + y_percent * cam_up_deflection)
                .normalize();
            let hit = mesh.ray_intersection(CAM_POS, ray);
            if let Some(collision) = hit {
                let attenuation = (collision.position - light_pos).squared_magnitude().recip();
                let reflected = ray.reflect_across(collision.normal);
                let brightness = reflected
                    .dot((light_pos - collision.position).normalize())
                    .max(0.); // Look here for specular-ness
                let color = Vec3::splat(brightness * attenuation * 100.);
                write_percent_vec(&mut file, color + Vec3::splat(0.1)).unwrap();
                // write_percent_vec_neg(&mut file, collision.normal).unwrap();
                // write_percent_vec(&mut file, Vec3::splat(((CAM_POS - collision.position).magnitude() - 15.) / 3.)).unwrap();
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

#[derive(Debug, PartialEq)]
struct Collision {
    position: Vec3,
    normal: Vec3,
    // color, scattering, ...
}

impl Collision {
    fn new(position: Vec3, normal: Vec3) -> Self {
        Self { position, normal }
    }

    // fn cmp(&self, other: &Self) -> Ordering {
    //     self.position
    //         .squared_magnitude()
    //         .total_cmp(&other.position.squared_magnitude())
    // }
}

trait Shape {
    fn ray_intersection(&self, ray_start: Vec3, ray_dir: Vec3) -> Option<Collision>;
}

impl<T> Shape for T
where
    T: Deref,
    T::Target: Shape,
{
    fn ray_intersection(&self, ray_start: Vec3, ray_dir: Vec3) -> Option<Collision> {
        (**self).ray_intersection(ray_start, ray_dir)
    }
}

impl<T> Shape for [T]
where
    T: Shape,
{
    fn ray_intersection(&self, ray_start: Vec3, ray_dir: Vec3) -> Option<Collision> {
        self.iter()
            .filter_map(|shape| shape.ray_intersection(ray_start, ray_dir))
            .min_by(|c1, c2| {
                (c1.position - ray_start)
                    .squared_magnitude()
                    .total_cmp(&(c2.position - ray_start).squared_magnitude())
            })
    }
}

#[derive(Debug, Clone)]
struct TriangleMesh {
    vertices: Vec<Vec3>,
    triangles: Vec<[u16; 3]>,
    normals: Vec<Vec3>,
}

impl TriangleMesh {
    fn new(vertices: Vec<Vec3>, triangles: Vec<[u16; 3]>) -> Self {
        let normals: Vec<_> = triangles
            .iter()
            .copied()
            .map(|[a, b, c]| {
                (vertices[b as usize] - vertices[a as usize])
                    .cross(vertices[c as usize] - vertices[b as usize])
                    .normalize()
            })
            .collect();
        Self {
            vertices,
            triangles,
            normals,
        }
    }
}

impl Shape for TriangleMesh {
    fn ray_intersection(&self, ray_start: Vec3, ray_dir: Vec3) -> Option<Collision> {
        let nearest_collision = self
            .triangles
            .iter()
            .copied()
            .zip(self.normals.iter().copied())
            .enumerate()
            .filter_map(|(i, ([a, b, c], normal))| {
                if ray_dir.dot(normal) > -1e-5 {
                    // ray and normal are roughly the same direction
                    return None;
                }

                let vertices = [
                    self.vertices[a as usize] - ray_start,
                    self.vertices[b as usize] - ray_start,
                    self.vertices[c as usize] - ray_start,
                ];
                if vertices.iter().all(|vertex| vertex.dot(ray_dir) < -1e-5) {
                    return None;
                }
                // println!("{}, {:?}", i, ray_dir);

                let normals = [
                    vertices[1].cross(vertices[0]),
                    vertices[2].cross(vertices[1]),
                    vertices[0].cross(vertices[2]),
                ];

                let intersects = normals.iter().all(|normal| normal.dot(ray_dir) > -1e-5);
                if !intersects {
                    return None;
                }
                let distance = (vertices[0]).project_onto(normal);
                let intersect_point =
                    (ray_dir.dot(distance) / distance.squared_magnitude()).recip() * ray_dir;
                Some((i, intersect_point))
            })
            .min_by(|(_, v1), (_, v2)| v1.squared_magnitude().total_cmp(&v2.squared_magnitude()));
        nearest_collision
            .map(|(i, intersect)| Collision::new(ray_start + intersect, self.normals[i]))
    }
}

#[derive(Debug, Clone)]
struct Sphere {
    center: Vec3,
    radius: f32,
}

impl Sphere {
    fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }
}

impl Shape for Sphere {
    fn ray_intersection(&self, ray_start: Vec3, ray_dir: Vec3) -> Option<Collision> {
        let relative_center = self.center - ray_start;
        let cx = relative_center.dot(ray_dir);
        let cc = relative_center.dot(relative_center);
        let xx = ray_dir.dot(ray_dir);
        let rr = self.radius * self.radius;
        let l = cx - (cx * cx - xx * (cc - rr)).sqrt();
        if l.is_nan() {
            None
        } else {
            let ray = ray_dir * l;
            let point = ray + ray_start;
            let normal = (relative_center - ray).normalize();
            Some(Collision::new(point, normal))
        }
    }
}
