use std::{
    cmp::Ordering,
    ops::{Deref, DerefMut},
};

use crate::{
    img::Color,
    material::{DiffuseColorMaterial, Material},
    math::{Mat3x3, Ray, Vec3},
    EPSILON,
};

#[derive(Debug, Clone)]
pub struct RayCollision<M: Material> {
    pub ray: Ray,
    pub collision: Collision<M>,
}

impl<M: Material> RayCollision<M> {
    pub fn new(ray: Ray, collision: Collision<M>) -> Self {
        Self { ray, collision }
    }

    pub fn collision_point(&self) -> Vec3 {
        self.ray.point_at(self.collision.distance)
    }

    pub fn reflection(&self) -> Ray {
        self.material
            .update_ray(self.ray.translate(self.collision.distance))
    }
}

impl<M: Material> Deref for RayCollision<M> {
    type Target = Collision<M>;

    fn deref(&self) -> &Self::Target {
        &self.collision
    }
}

impl<M: Material> DerefMut for RayCollision<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.collision
    }
}

#[derive(Debug, Clone)]
pub struct Collision<M: Material> {
    pub distance: f64,
    pub material: M,
}

impl<M: Material> Collision<M> {
    pub fn new(distance: f64, material: M) -> Self {
        Self { distance, material }
    }

    pub fn cmp<M2: Material>(&self, other: &Collision<M2>) -> Ordering {
        self.distance.total_cmp(&other.distance)
    }
}

impl<M: Material + Copy> Copy for Collision<M> {}

pub trait Shape {
    type Material: Material;
    // when ray_start is on some surface, only if include_start
    // and the ray is facing into the surface, it should return a collision
    fn ray_intersection(&self, ray: Ray, include_start: bool) -> Option<Collision<Self::Material>>;

    fn intersect_inclusive(&self, ray: Ray) -> Option<RayCollision<Self::Material>> {
        self.ray_intersection(ray.clone(), true)
            .map(|collision| RayCollision::new(ray, collision))
    }

    fn intersect_exclusive(&self, ray: Ray) -> Option<RayCollision<Self::Material>> {
        self.ray_intersection(ray.clone(), false)
            .map(|collision| RayCollision::new(ray, collision))
    }
}

impl<T> Shape for T
where
    T: Deref,
    T::Target: Shape,
{
    type Material = <T::Target as Shape>::Material;

    fn ray_intersection(&self, ray: Ray, include_start: bool) -> Option<Collision<Self::Material>> {
        (**self).ray_intersection(ray, include_start)
    }
}

impl<T> Shape for [T]
where
    T: Shape,
{
    type Material = T::Material;

    fn ray_intersection(&self, ray: Ray, include_start: bool) -> Option<Collision<Self::Material>> {
        self.iter()
            .filter_map(|shape| shape.ray_intersection(ray.clone(), include_start))
            .min_by(|c1, c2| c1.cmp(c2))
    }
}

#[derive(Debug, Clone)]
pub struct TriangleMesh {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<[u16; 3]>,
    pub tri_colors: Vec<u16>,
    pub triangle_projections: Vec<Mat3x3>,
    pub normals: Vec<Vec3>,
    pub colors: Vec<Color>,
}

pub type VertexIndex = u16;
pub type ColorIndex = u16;

impl TriangleMesh {
    pub fn new(
        vertices: Vec<Vec3>,
        colors: Vec<Color>,
        triangles: Vec<([VertexIndex; 3], ColorIndex)>,
    ) -> Self {
        let tri_colors = triangles.iter().map(|(_, c)| *c).collect();
        let normals: Vec<_> = triangles
            .iter()
            .copied()
            .map(|([a, b, c], _)| {
                (vertices[b as usize] - vertices[a as usize])
                    .cross(vertices[c as usize] - vertices[b as usize])
                    .normalize()
            })
            .collect();
        let triangle_projections = triangles
            .iter()
            .copied()
            .zip(normals.iter().cloned())
            .map(|(([a, b, c], _), normal)| {
                let v100 = vertices[b as usize] - vertices[a as usize];
                let v010 = vertices[c as usize] - vertices[a as usize];
                let v001 = vertices[a as usize].project_onto(normal);
                let fwd_change_of_basis = Mat3x3::from_col_vectors(v100, v010, v001);
                fwd_change_of_basis.inverse().unwrap()
            })
            .collect();
        let triangles = triangles
            .into_iter()
            .map(|([a, b, c], _)| [a, b, c])
            .collect();
        Self {
            vertices,
            triangles,
            tri_colors,
            triangle_projections,
            normals,
            colors,
        }
    }

    // pub fn with_uvs() {
    //     todo!()
    // }
}

impl Shape for TriangleMesh {
    type Material = DiffuseColorMaterial;

    fn ray_intersection(
        &self,
        ray: Ray,
        include_start: bool,
    ) -> Option<Collision<DiffuseColorMaterial>> {
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
        nearest_collision.map(|(i, intersect)| {
            Collision::new(
                intersect,
                DiffuseColorMaterial::new(
                    self.normals[i],
                    self.colors[self.tri_colors[i] as usize],
                ),
            )
        })
    }
}

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub color: Color,
}

impl Sphere {
    pub const fn new(center: Vec3, radius: f64, color: Color) -> Self {
        Self {
            center,
            radius,
            color,
        }
    }

    fn intersect_equation(&self, ray: Ray) -> (Vec3, f64, f64) {
        let relative_center = self.center - ray.start;
        let cx = relative_center.dot(ray.dir);
        let cc = relative_center.dot(relative_center);
        let xx = ray.dir.dot(ray.dir);
        let rr = self.radius * self.radius;
        let root = cx * cx - xx * (cc - rr);
        (relative_center, cx, root)
    }
}

impl Shape for Sphere {
    type Material = DiffuseColorMaterial;

    fn ray_intersection(&self, ray: Ray, include_start: bool) -> Option<Collision<Self::Material>> {
        let (relative_center, cx, root) = self.intersect_equation(ray.clone());
        if root.is_sign_negative() {
            return None;
        }
        let l = cx - root.sqrt();
        if !include_start && l < EPSILON {
            return None;
        }
        let normal = (ray.dir * l - relative_center) / self.radius;
        Some(Collision::new(
            l,
            DiffuseColorMaterial::new(normal, self.color),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct InvertedSphere(Sphere);

impl InvertedSphere {
    pub fn new(center: Vec3, radius: f64, color: Color) -> Self {
        Self(Sphere::new(center, radius, color))
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
    type Material = DiffuseColorMaterial;

    fn ray_intersection(&self, ray: Ray, include_start: bool) -> Option<Collision<Self::Material>> {
        let (relative_center, cx, root) = self.0.intersect_equation(ray.clone());
        if root.is_sign_negative() {
            return None;
        }
        let l = cx + root.sqrt();
        if !include_start && l < EPSILON {
            return None;
        }
        let normal = -(ray.dir * l - relative_center) / self.0.radius;
        Some(Collision::new(
            l,
            DiffuseColorMaterial::new(normal, self.0.color),
        ))
    }
}
