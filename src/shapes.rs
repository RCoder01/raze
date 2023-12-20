use std::{ops::{Deref, DerefMut}, cmp::Ordering};

use crate::{
    math::{Mat3x3, Ray, Vec3},
    EPSILON,
};

#[derive(Debug, Clone)]
pub struct RayCollision {
    pub ray: Ray,
    pub collision: Collision,
}

impl RayCollision {
    pub fn new(ray: Ray, collision: Collision) -> Self {
        Self { ray, collision }
    }

    pub fn position(&self) -> Vec3 {
        self.ray.translate(self.collision.distance)
    }

    pub fn reflection(&self) -> Ray {
        Ray::new(
            self.position(),
            self.ray.dir.reflect_across(self.collision.normal),
        )
    }
}

impl Deref for RayCollision {
    type Target = Collision;

    fn deref(&self) -> &Self::Target {
        &self.collision
    }
}

impl DerefMut for RayCollision {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.collision
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Collision {
    pub distance: f64,
    pub normal: Vec3,
    // color, scattering, ...
}

impl PartialOrd for Collision {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

impl Collision {
    pub fn new(distance: f64, normal: Vec3) -> Self {
        Self { distance, normal }
    }
}

pub trait Shape {
    // when ray_start is on some surface, only if include_start
    // and the ray is facing into the surface, it should return a collision
    fn ray_intersection(&self, ray: Ray, include_start: bool) -> Option<Collision>;

    fn intersect_inclusive(&self, ray: Ray) -> Option<RayCollision> {
        self.ray_intersection(ray.clone(), true).map(|collision| RayCollision::new(ray, collision))
    }

    fn intersect_exclusive(&self, ray: Ray) -> Option<RayCollision> {
        self.ray_intersection(ray.clone(), false).map(|collision| RayCollision::new(ray, collision))
    }
}

impl<T> Shape for T
where
    T: Deref,
    T::Target: Shape,
{
    fn ray_intersection(&self, ray: Ray, include_start: bool) -> Option<Collision> {
        (**self).ray_intersection(ray, include_start)
    }
}

impl<T> Shape for [T]
where
    T: Shape,
{
    fn ray_intersection(&self, ray: Ray, include_start: bool) -> Option<Collision> {
        self.iter()
            .filter_map(|shape| shape.ray_intersection(ray.clone(), include_start))
            .min_by(|c1, c2| {
                c1.partial_cmp(c2)
                    .expect("Collision distance should not be NaN")
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
                fwd_change_of_basis.inverse().unwrap()
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
    fn ray_intersection(&self, ray: Ray, include_start: bool) -> Option<Collision> {
        let nearest_collision = self
            .triangles
            .iter()
            .copied()
            .zip(self.triangle_projections.iter())
            .enumerate()
            .filter_map(|(i, ([a, _b, _c], projection))| {
                let start_in_triangle_space = projection * ray.start;
                if ray.dir.dot(self.normals[i]) > -EPSILON
                    || start_in_triangle_space.z < -1. - EPSILON
                    || (!include_start && start_in_triangle_space.z < -1. + EPSILON)
                {
                    return None;
                }
                let ray_in_triangle_space = projection * ray.dir;
                let w = self.vertices[a as usize];
                let mut corner_in_triangle_space = projection * w;
                corner_in_triangle_space.z = 0.;
                let ray_scale = (1. - start_in_triangle_space.z) / ray_in_triangle_space.z;
                let uvw = ray_in_triangle_space * ray_scale + start_in_triangle_space
                    - corner_in_triangle_space;
                if ray_in_triangle_space.z.abs() <= EPSILON
                    || !ray_scale.is_finite()
                    || ray_scale <= -EPSILON
                    || (!include_start && ray_scale <= EPSILON)
                    || uvw.x + uvw.y > 1. + EPSILON
                    || uvw.x < -EPSILON
                    || uvw.y < -EPSILON
                    || !uvw.x.is_finite()
                    || !uvw.y.is_finite()
                    || !uvw.z.is_finite()
                {
                    return None;
                }
                Some((i, ray_scale))
            })
            .min_by(|(_, d1), (_, d2)| d1.total_cmp(d2));
        nearest_collision.map(|(i, intersect)| Collision::new(intersect, self.normals[i]))
    }
}

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f64) -> Self {
        Self { center, radius }
    }
}

impl Shape for Sphere {
    fn ray_intersection(&self, ray: Ray, include_start: bool) -> Option<Collision> {
        let relative_center = self.center - ray.start;
        let cx = relative_center.dot(ray.dir);
        let cc = relative_center.dot(relative_center);
        let xx = ray.dir.dot(ray.dir);
        let rr = self.radius * self.radius;
        let root = cx * cx - xx * (cc - rr);
        if root.is_sign_negative() {
            return None;
        }
        let l = cx - root.sqrt();
        if !include_start && l < EPSILON {
            return None;
        }
        let normal = (ray.dir * l - relative_center) / self.radius;
        Some(Collision::new(l, normal))
    }
}

#[derive(Debug, Clone)]
pub struct InvertedSphere(Sphere);

impl InvertedSphere {
    pub fn new(center: Vec3, radius: f64) -> Self {
        Self(Sphere::new(center, radius))
    }
}

impl From<Sphere> for InvertedSphere {
    fn from(value: Sphere) -> Self {
        Self(value)
    }
}

impl From<InvertedSphere> for Sphere {
    fn from(value: InvertedSphere) -> Self {
        value.0
    }
}

impl Shape for InvertedSphere {
    fn ray_intersection(&self, ray: Ray, include_start: bool) -> Option<Collision> {
        let relative_center = self.0.center - ray.start;
        let cx = relative_center.dot(ray.dir);
        let cc = relative_center.dot(relative_center);
        let xx = ray.dir.dot(ray.dir);
        let rr = self.0.radius * self.0.radius;
        let l = cx + (cx * cx - xx * (cc - rr)).sqrt();
        if l.is_nan() || l < -EPSILON * 1e2 || (!include_start && l < EPSILON * 1e2) {
            None
        } else {
            let normal = (relative_center - ray.dir * l) / self.0.radius;
            Some(Collision::new(l, normal))
        }
    }
}
