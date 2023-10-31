use std::ops::Deref;

use super::Float;
use crate::{
    math::{Mat3x3, Vec3},
    EPSILON,
};

#[derive(Debug, PartialEq)]
pub struct Collision {
    pub position: Vec3,
    pub normal: Vec3,
    // color, scattering, ...
}

impl Collision {
    pub fn new(position: Vec3, normal: Vec3) -> Self {
        Self { position, normal }
    }

    // fn cmp(&self, other: &Self) -> Ordering {
    //     self.position
    //         .squared_magnitude()
    //         .total_cmp(&other.position.squared_magnitude())
    // }
}

pub trait Shape {
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
pub struct TriangleMesh {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<[u16; 3]>,
    pub triangle_projections: Vec<Mat3x3>,
    pub normals: Vec<Vec3>,
}

impl TriangleMesh {
    pub fn new(vertices: Vec<Vec3>, triangles: Vec<[u16; 3]>) -> Self {
        let normals: Vec<_> = triangles
            .iter()
            .copied()
            .map(|[a, b, c]| {
                (vertices[b as usize] - vertices[a as usize])
                    .cross(vertices[c as usize] - vertices[b as usize])
                    .normalize()
            })
            .collect();
        let triangle_projections = triangles
            .iter()
            .copied()
            .zip(normals.iter().cloned())
            .map(|([a, b, c], normal)| {
                let v100 = vertices[b as usize] - vertices[a as usize];
                let v010 = vertices[c as usize] - vertices[a as usize];
                let v001 = vertices[a as usize].project_onto(normal);
                let fwd_change_of_basis = Mat3x3::from_col_vectors(v100, v010, v001);
                let inv = fwd_change_of_basis.inverse().unwrap();
                inv
            })
            .collect();
        Self {
            vertices,
            triangles,
            triangle_projections,
            normals,
        }
    }

    // pub fn with_uvs() {
    //     todo!()
    // }
}

impl Shape for TriangleMesh {
    fn ray_intersection(&self, ray_start: Vec3, ray_dir: Vec3) -> Option<Collision> {
        let nearest_collision = self
            .triangles
            .iter()
            .copied()
            .zip(self.triangle_projections.iter())
            .enumerate()
            .filter_map(|(i, ([a, b, c], projection))| {
                let start_in_triangle_space = projection * ray_start;
                // dbg!(
                //     start_in_triangle_space,
                //     self.vertices[a as usize],
                //     self.vertices[b as usize],
                //     self.vertices[c as usize],
                //     projection
                // );
                if start_in_triangle_space.z <= -1. {
                    return None;
                }
                let ray_in_triangle_space = projection * ray_dir;
                let u = self.vertices[b as usize] - self.vertices[a as usize];
                let v = self.vertices[c as usize] - self.vertices[a as usize];
                let w = self.vertices[a as usize];
                let mut corner_in_triangle_space = projection * w;
                // dbg!(corner_in_triangle_space, ray_in_triangle_space);
                corner_in_triangle_space.z = 0.;
                let ray_scale = (1. - start_in_triangle_space.z) / ray_in_triangle_space.z;
                // dbg!(ray_scale);
                let uvw = ray_in_triangle_space * ray_scale + start_in_triangle_space
                    - corner_in_triangle_space;
                // dbg!(uvw);
                if uvw.x + uvw.y > 1. + EPSILON
                    || uvw.x < -EPSILON
                    || uvw.y < -EPSILON
                    || ray_in_triangle_space.z.abs() <= EPSILON
                    || ray_scale.abs() <= EPSILON
                {
                    return None;
                }

                // Some((i, dbg!(w + uvw.x * u + uvw.y * v), ray_scale))
                Some((i, w + uvw.x * u + uvw.y * v, ray_scale))
            })
            .min_by(|(_, _, d1), (_, _, d2)| d1.total_cmp(&d2));
        nearest_collision
            .map(|(i, intersect, _)| Collision::new(intersect, self.normals[i]))
    }
}

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: Float,
}

impl Sphere {
    pub fn new(center: Vec3, radius: Float) -> Self {
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
