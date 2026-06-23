// This file is licensed under the EUPL-1.2. See LICENSE.md.

//! Constructive Solid Geometry module.

use crate::functions::{Within, are_close};
use crate::geometry::{Dot, Normal, Point, Vec2D};
use crate::hit_record::HitRecord;
use crate::materials::Material;
use crate::ray::Ray;
use crate::shapes::{Shape, Volumetric};
use anyhow::anyhow;
use std::cmp::min;

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

    pub fn eet_intersection(&self, ray: &Ray) -> Option<(f32, f32)> {
        let (a1, b1): (f32, f32);
        let (a2, b2): (f32, f32);
        match self.object1.entry_exit_t(ray) {
            None => return None,
            Some((a, b)) => {
                (a1, b1) = (a, b)
            }
        }
        match self.object2.entry_exit_t(ray) {
            None => return None,
            Some((a, b)) => {
                (a2, b2) = (a, b)
            }
        }
        let entry = a1.max(a2);
        let exit = b1.min(b2);
        if entry < exit {
            // No borders
            Some((entry, exit))
        } else {
            None
        }
    }

    // Design choice: if there is no intersection or total intersection code returns error.
    pub fn eet_difference(&self, ray: &Ray) -> anyhow::Result<Option<(f32, f32)>> {
        match self.object1.entry_exit_t(ray) {
            None => Ok(None),
            Some((a, b)) => match self.object2.entry_exit_t(ray) {
                None => Ok(Some((a, b))),
                Some((c, d)) => {
                    if a < c && b.is_between_close(&c, &d) {
                        Ok(Some((a, c)))
                    } else if c < a && d.is_between_close(&a, &b) {
                        Ok(Some((d, b)))
                    } else {
                        Err(anyhow!(
                            "Unsupported CSG difference configuration:
object1 interval = ({a}, {b})
object2 interval = ({c}, {d})

The current implementation only supports a single partial overlap."
                        ))
                    }
                }
            },
        }
    }
}

impl Shape for CSG {
    fn ray_intersection(&self, ray: &Ray) -> Option<HitRecord<'_>> {
        match self.operation {
            OperationsCSGType::Intersection => {

                    let (t1_a, t2_a) = self.object1.entry_exit_t(ray)?;
                    let (t1_b, t2_b) = self.object2.entry_exit_t(ray)?;


                let entry = t1_a.max(t1_b);
                let exit  = t2_a.min(t2_b);
                if entry >= exit || entry < 0.0 { return None; }

                let (obj, t_hit) = if are_close(entry, t1_a){
                    (&self.object1, entry)
                } else {
                    (&self.object2, entry)
                };

                obj.ray_intersection(ray)
                  /* let world_point = ray.at(t_hit);
                let normal = obj.normal_at(world_point, ray);
                    Some(HitRecord {
                        world_point,
                        normal,
                        uv: obj.point_to_uv(&world_point).unwrap(),
                        t: t_hit,
                        ray: *ray,
                        material: obj.material(),
                    })*/
                }

            OperationsCSGType::Union => {
                let int1 = self.object1.ray_intersection(ray);
                let int2 = self.object2.ray_intersection(ray);
                match int1 {
                    Some(int1) => match int2 {
                        Some(int2) => {
                            if int1.t < int2.t {
                                return Some(int1);
                            }
                            Some(int2)
                        }
                        None => Some(int1),
                    },
                    None => int2,
                }
            }
            OperationsCSGType::Difference => {
                let ent_ex_1 = self.object1.entry_exit_t(ray);
                let ent_ex_2 = self.object2.entry_exit_t(ray);
                match ent_ex_1 {
                    Some((t1, t2)) => {
                        match ent_ex_2 {
                            Some((t1_diff, t2_diff)) => {
                                if t1_diff < t1 {
                                    if t2_diff > t2 { return None } //obj2 makes a hole in obj1
                                    if t2_diff > t1 {
                                        let world_point = transformed_ray.at(t2_diff);
                                        return Some(HitRecord {
                                            world_point,
                                            normal: -self.object2.transformation * self.object2.normal_at(world_point, ray),
                                            uv: self.object1.point_to_uv(&world_point).unwrap(),
                                            t: t2_diff,
                                            ray: *ray,
                                            material: self.object1.material(),
                                        })

                                    /*let world_point = ray.at(t2_diff)
                                    let mut hit = HitRecord{
                                        world_point,

                                    */
                                    }

                                }
                                self.object1.ray_intersection(ray)
                        },
                            None => self.object1.ray_intersection(ray)
                        }
                    },
                    None => None
                }
            }
        }
    }

    fn normal_at(&self, point: Point, ray: &Ray) -> Normal {
        todo!()
    }

    fn point_to_uv(&self, point: &Point) -> anyhow::Result<Vec2D> {
        todo!()
    }

    // Might be useful to change it in Shape definition
    // to feature the possibility of having more than one material
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
    use crate::functions::{IDENTITY_4X4, are_close};
    use crate::geometry::{Vector, Y_AXIS, Z_AXIS};
    use crate::materials::Material;
    use crate::shapes::{AABB, Sphere};
    use crate::transformations::{Scaling, Transformation, Translation};
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

    fn setup_csg_intersection() -> CSG {
        let material = Material::default();
        let transformation = Translation::new(Z_AXIS) * Scaling::new([0.5, 0.5, 0.5]);
        let object1: Box<dyn Volumetric> = Box::new(Sphere::new(transformation, material.clone()));
        let point1 = Point::new(0.0, 0.0, 0.0);
        let point2 = Point::new(1.0, 1.0, 1.0);
        let object2: Box<dyn Volumetric> =
            Box::new(AABB::new(point1, point2, material.clone()).unwrap());
        CSG::new(object1, object2, OperationsCSGType::Intersection)
    }

    #[test]
    fn test_csg_eet_intersection_totally_miss() {
        let intersection_csg = setup_csg_intersection();

        let origin = Point::new(10.0, 0.0, 0.0);
        let ray_miss = Ray::new(origin, Y_AXIS);

        let intersection = intersection_csg.eet_intersection(&ray_miss);
        assert!(intersection.is_none(), "{:?}", intersection);
    }

    #[test]
    fn test_csg_eet_intersection_miss_sphere() {
        let intersection_csg = setup_csg_intersection();

        let origin = Point::new(0.7, 0.7, 0.0);
        let ray_miss = Ray::new(origin, Z_AXIS);

        let intersection = intersection_csg.eet_intersection(&ray_miss);
        assert!(intersection.is_none(), "{:?}", intersection);
    }

    #[test]
    fn test_csg_eet_intersection_miss_cube() {
        let intersection_csg = setup_csg_intersection();

        let origin = Point::new(0.2, -0.1, 1.2);
        let ray_miss = Ray::new(origin, Y_AXIS);

        let intersection = intersection_csg.eet_intersection(&ray_miss);
        assert!(intersection.is_none(), "{:?}", intersection);
    }

    #[test]
    fn test_csg_eet_intersection_out() {
        let intersection_csg = setup_csg_intersection();

        let origin = Point::new(0.2, 0.2, -1.0);
        let ray_hit = Ray::new(origin, Z_AXIS);

        let (t1, t2): (f32, f32) = match intersection_csg.eet_intersection(&ray_hit) {
            None => panic!("Should hit!!"),
            Some((a, b)) => (a, b),
        };

        // x^2 + y^2 + (z-1)^2 = 0.5^2 => z = 1 - sqrt(0.5^2 - 2 *  0.2^2)
        let z = 1.0 - ((0.5 * 0.5 - 2.0 * 0.2 * 0.2) as f32).sqrt();
        assert!(are_close(t1, z + 1.0), "{}", t1);
        assert!(are_close(t2, 2.0), "{}", t2);
    }

    #[test]
    fn test_csg_eet_intersection_in() {
        let intersection_csg = setup_csg_intersection();
        let origin = Point::new(0.2, 0.2, 0.7);
        let ray_hit = Ray::new(origin, -Z_AXIS);

        let (t1, t2): (f32, f32) = match intersection_csg.eet_intersection(&ray_hit) {
            None => panic!("Should hit!!"),
            Some((a, b)) => (a, b),
        };

        // Once again
        // x^2 + y^2 + (z-1)^2 = 0.5^2 => z = 1 - sqrt(0.5^2 - 2 *  0.2^2)
        let z_intersection = 1.0 - ((0.5 * 0.5 - 2.0 * 0.2 * 0.2) as f32).sqrt();
        assert!(are_close(t1, -0.3), "{}", t1);
        assert!(are_close(t2, 0.7 - z_intersection), "{}", t2);
    }
}
