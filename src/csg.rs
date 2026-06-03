//! Constructive Solid Geometry module.

use crate::geometry::{Normal, Point, Vec2D};
use crate::hit_record::HitRecord;
use crate::materials::Material;
use crate::ray::Ray;
use crate::shapes::{Shape, Volumetric};

#[derive(Clone)]
pub enum OperationsCSGType {
    Intersection,
    Union,
    Difference,
}

#[derive(Clone)]
pub struct CSG {
    pub object1: Box<dyn Volumetric>,
    pub object2: Box<dyn Volumetric>,
    pub operation: OperationsCSGType,
}

impl CSG {
    pub fn new(
        object1: Box<dyn Volumetric>,
        object2: Box<dyn Volumetric>,
        operation: OperationsCSGType,
    ) -> Self {
        Self {
            object1,
            object2,
            operation,
        }
    }
}

impl Shape for CSG {
    fn ray_intersection(&self, ray: &Ray) -> Option<HitRecord<'_>> {
        todo!()
    }

    fn normal_at(&self, point: Point, ray: &Ray) -> Normal {
        todo!()
    }

    fn point_to_uv(&self, point: &Point) -> anyhow::Result<Vec2D> {
        todo!()
    }

    fn material(&self) -> &Material {
        todo!()
    }
}

impl Volumetric for CSG {
    fn entry_exit_t(&self, ray: &Ray) -> Option<(f32, f32)> {
        todo!()
    }
}

// =============================================================
// Tests
// =============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::IDENTITY_4X4;
    use crate::geometry::Vector;
    use crate::materials::Material;
    use crate::shapes::Sphere;
    use crate::transformations::{Transformation, Translation};
    #[test]
    fn test_csg_constructor() {
        let transformation = Transformation::new(IDENTITY_4X4);
        let material = Material::default();

        let object1: Box<dyn Volumetric> = Box::new(Sphere::new(transformation, material.clone()));
        let object2: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.1, 0.0, 0.0)),
            material,
        ));
        let csg = CSG::new(object1.clone(), object2, OperationsCSGType::Intersection);

        let _ = CSG::new(object1, Box::new(csg), OperationsCSGType::Union);
    }
}
