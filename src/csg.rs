// This file is licensed under the EUPL-1.2. See LICENSE.md.

//! Constructive Solid Geometry module.

use crate::functions::{Within, are_close};
use crate::geometry::{Dot, Normal, Point, Vec2D};
use crate::hit_record::HitRecord;
use crate::materials::Material;
use crate::ray::Ray;
use crate::shapes::{Shape, Volumetric};
use crate::transformations::Transformation;

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventKind {
    EnterA,
    ExitA,
    EnterB,
    ExitB,
}

#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub(crate) t: f32,
    pub(crate) kind: EventKind,
}

pub fn events_for_object(entry: f32, exit: f32, is_a: bool) -> Vec<Event> {
    if is_a {
        vec![
            Event {
                t: entry,
                kind: EventKind::EnterA,
            },
            Event {
                t: exit,
                kind: EventKind::ExitA,
            },
        ]
    } else {
        vec![
            Event {
                t: entry,
                kind: EventKind::EnterB,
            },
            Event {
                t: exit,
                kind: EventKind::ExitB,
            },
        ]
    }
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
        match self.operation {
            OperationsCSGType::Intersection => {
                let mut events = Vec::new();

                self.fill_intersection_vector(ray, &mut events, false);

                events.sort_by(|e1, e2| e1.t.partial_cmp(&e2.t).unwrap());

                let mut is_in_a = false;
                let mut is_in_b = false;

                for event in events {
                    match event.kind {
                        EventKind::EnterA => is_in_a = true,
                        EventKind::ExitA => is_in_a = false,
                        EventKind::EnterB => is_in_b = true,
                        EventKind::ExitB => is_in_b = false,
                    }

                    if is_in_a && is_in_b && event.t > 0.0 && !are_close(event.t, 0.0) {
                        let mut hit = match event.kind {
                            EventKind::EnterA | EventKind::ExitA => {
                                self.object1.hit_from_t(ray, event.t)?
                            }
                            EventKind::ExitB | EventKind::EnterB => {
                                self.object2.hit_from_t(ray, event.t)?
                            }
                        };

                        hit.material = self.object1.material();
                        return Some(hit);
                    }
                }
                None
            }

            OperationsCSGType::Union => {
                let mut events = Vec::new();

                self.fill_intersection_vector(ray, &mut events, false);

                events.sort_by(|e1, e2| e1.t.partial_cmp(&e2.t).unwrap());

                let mut is_in_a = false;
                let mut is_in_b = false;

                for event in events {
                    match event.kind {
                        EventKind::EnterA => is_in_a = true,
                        EventKind::ExitA => is_in_a = false,
                        EventKind::EnterB => is_in_b = true,
                        EventKind::ExitB => is_in_b = false,
                    }

                    if (is_in_a || is_in_b) && event.t > 0.0 && !are_close(event.t, 0.0) {
                        let mut hit = match event.kind {
                            EventKind::EnterA | EventKind::ExitA => {
                                self.object1.hit_from_t(ray, event.t)?
                            }
                            EventKind::ExitB | EventKind::EnterB => {
                                self.object2.hit_from_t(ray, event.t)?
                            }
                        };

                        hit.material = self.object1.material();
                        return Some(hit);
                    }
                }
                None
            }

            OperationsCSGType::Difference => {
                /* let mut events = Vec::new();

                if let Some((a1, a2)) = self.object1.entry_exit_t(ray) {
                    // se l'intervallo è dietro al raggio, ignoriamo
                    if a2 > 0.0 || are_close(a2, 0.0) {
                        events.extend(events_for_object(a1, a2, true));
                    }
                } else {
                    return None;
                }

                if let Some((b1, b2)) = self.object2.entry_exit_t(ray) {
                    if b2 > 0.0 || are_close(b2, 0.0)  {
                        events.extend(events_for_object(b1, b2, false));
                    }
                }


                if events.is_empty() {
                    return None;
                }

                events.extend(events_for_object(1.0, 3.0, false));

                // ordina gli eventi per t
                events.sort_by(|e1, e2| e1.t.partial_cmp(&e2.t).unwrap());

                let mut inside_a = false;
                let mut inside_b = false;

                for event in events {
                    match event.kind {
                        EventKind::EnterA => inside_a = true,
                        EventKind::ExitA => {},
                        EventKind::EnterB => {},
                        EventKind::ExitB => inside_b = false,
                    }

                    // Difference: A - B → visibile quando dentro A e fuori B
                    if inside_a && !inside_b && event.t >= 0.0 {
                        let world_point = ray.at(event.t);

                        // Se l'evento è EnterA o ExitA → normale di A
                        // Se è EnterB o ExitB → normale di B
                        let mut hit = match event.kind {
                            EventKind::EnterA | EventKind::ExitA => self.object1.hit_from_t(ray, event.t)?,
                            EventKind::ExitB | EventKind::EnterB => self.object2.hit_from_t(ray, event.t)?,
                        };

                        hit.material = self.object1.material();
                        return Some(hit);
                    }
                    match event.kind {
                        EventKind::EnterA => {},
                        EventKind::ExitA => inside_a = false,
                        EventKind::EnterB => inside_b = true,
                        EventKind::ExitB => {},
                    }
                }

                None*/

                let mut events = Vec::new();

                self.fill_intersection_vector(ray, &mut events, false);

                events.sort_by(|e1, e2| e1.t.partial_cmp(&e2.t).unwrap());

                let mut is_in_a = false;
                let mut is_in_b = false;

                for event in events {
                    match event.kind {
                        EventKind::EnterA => is_in_a = true,
                        /*EventKind::ExitA => is_in_a = false,
                        EventKind::EnterB => is_in_b = true,*/
                        EventKind::ExitB => is_in_b = false,
                        _ => {}
                    }

                    if is_in_a && !is_in_b && event.t > 0.0 && !are_close(event.t, 0.0) {
                        let mut hit = match event.kind {
                            EventKind::EnterA | EventKind::ExitA => {
                                self.object1.hit_from_t(ray, event.t)?
                            }
                            EventKind::ExitB | EventKind::EnterB => {
                                self.object2.hit_from_t(ray, event.t)?
                            }
                        };

                        hit.material = self.object1.material();
                        return Some(hit);
                    }
                    match event.kind {
                        /*EventKind::EnterA => is_in_a = true,
                        EventKind::ExitB => is_in_b = false,*/
                        EventKind::ExitA => is_in_a = false,
                        EventKind::EnterB => is_in_b = true,
                        _ => {}
                    }
                }

                None
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
        &self.object1.material()
    }
}

impl Volumetric for CSG {
    fn entry_exit_t(&self, ray: &Ray) -> Option<(f32, f32)> {
        todo!()
    }

    fn hit_from_t(&self, ray: &Ray, t: f32) -> Option<HitRecord> {
        todo!()
    }

    fn fill_intersection_vector(&self, ray: &Ray, vec: &mut Vec<Event>, is_subtracted: bool) {
        self.object1
            .fill_intersection_vector(ray, vec, true == is_subtracted);
        self.object2
            .fill_intersection_vector(ray, vec, false == is_subtracted);
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
    use crate::transformations::{Scaling, Transformation, Translation};
    use std::num::FpCategory::Normal;
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

    #[test]
    fn test_intersection_no_overlap() {
        let material = Material::default();

        let object1: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(10.0, 0.0, 0.0)),
            material.clone(),
        ));
        let object2: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, 0.0)),
            material,
        ));
        let csg = CSG::new(object1, object2, OperationsCSGType::Intersection);
        let origin = Point::new(-3.0, 0.0, 0.0);
        let dir = Vector::new(1.0, 0.0, 0.0);
        let ray = Ray::new(origin, dir);
        let int = csg.ray_intersection(&ray);
        match int {
            None => {}
            Some(int) => {
                panic!("there should me no intersections! {}", int.t)
            }
        }
    }

    #[test]
    fn test_intersection_partial_overlap() {
        let material = Material::default();

        let object1: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(10.0, 0.0, 0.0)),
            material.clone(),
        ));
        let object2: Box<dyn Volumetric> = Box::new(Sphere::new(
            /*Scaling::new([0.2, 0.2, 0.2]) **/
            Translation::new(Vector::new(11.0, 0.0, 0.0)),
            material,
        ));
        let csg = CSG::new(object1, object2, OperationsCSGType::Intersection);
        let origin = Point::new(-3.0, 0.0, 0.0);
        let dir = Vector::new(1.0, 0.0, 0.0);
        let ray = Ray::new(origin, dir);
        let int = csg.ray_intersection(&ray);
        match int {
            None => {
                panic!("should have found an intersection!")
            }
            Some(int) => {
                assert_eq!(int.t, 13.0);
            }
        }
    }

    #[test]
    fn test_union_1() {
        let transformation = Transformation::new(IDENTITY_4X4);
        let material = Material::default();

        let object1: Box<dyn Volumetric> = Box::new(Sphere::new(transformation, material.clone()));
        let object2: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, 0.0)),
            material,
        ));
        let csg = CSG::new(object1, object2, OperationsCSGType::Union);
        let origin = Point::new(-3.0, 0.0, 0.0);
        let dir = Vector::new(1.0, 0.0, 0.0);
        let ray = Ray::new(origin, dir);
        let int = csg.ray_intersection(&ray);
        match int {
            None => {
                panic!("ray_intersection should have found an intersection")
            }
            Some(int) => {
                assert_eq!(int.t, 2.0);
                assert_eq!(int.world_point, Point::new(-1.0, 0.0, 0.0));
            }
        }
    }

    #[test]
    fn test_union_complete_overlap() {
        let material = Material::default();

        let object1: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, 0.0)),
            material.clone(),
        ));
        let object2: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, 0.0)),
            material,
        ));
        let csg = CSG::new(object1, object2, OperationsCSGType::Union);
        let origin = Point::new(-3.0, 0.0, 0.0);
        let dir = Vector::new(1.0, 0.0, 0.0);
        let ray = Ray::new(origin, dir);
        let int = csg.ray_intersection(&ray);
        match int {
            None => {
                panic!("ray_intersection should have found an intersection")
            }
            Some(int) => {
                assert_eq!(int.t, 2.0);
                assert_eq!(int.world_point, Point::new(-1.0, 0.0, 0.0));
            }
        }
    }

    #[test]
    fn test_union_no_overlap() {
        let material = Material::default();

        let object1: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(10.0, 0.0, 0.0)),
            material.clone(),
        ));
        let object2: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, 0.0)),
            material,
        ));
        let csg = CSG::new(object1, object2, OperationsCSGType::Union);
        let origin = Point::new(-3.0, 0.0, 0.0);
        let dir = Vector::new(1.0, 0.0, 0.0);
        let ray = Ray::new(origin, dir);
        let int = csg.ray_intersection(&ray);
        match int {
            None => {
                panic!("ray_intersection should have found an intersection")
            }
            Some(int) => {
                assert_eq!(int.t, 2.0);
                assert_eq!(int.world_point, Point::new(-1.0, 0.0, 0.0));
            }
        }
    }

    #[test]
    fn test_union_from_inside() {
        let material = Material::default();

        let object1: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(10.0, 0.0, 0.0)),
            material.clone(),
        ));
        let object2: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, 0.0)),
            material,
        ));
        let csg = CSG::new(object1, object2, OperationsCSGType::Union);
        let origin = Point::new(-0.0, 0.0, 0.0);
        let dir = Vector::new(1.0, 0.0, 0.0);
        let ray = Ray::new(origin, dir);
        let int = csg.ray_intersection(&ray);
        match int {
            None => {
                panic!("ray_intersection should have found an intersection")
            }
            Some(int) => {
                assert_eq!(int.t, 1.0);
                assert_eq!(int.world_point, Point::new(1.0, 0.0, 0.0));
            }
        }
    }

    #[test]
    fn test_difference_from_inside_z() {
        let material = Material::default();

        let object1: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, 0.0)),
            material.clone(),
        ));
        let object2: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, -1.0)),
            material,
        ));
        let csg = CSG::new(object1, object2, OperationsCSGType::Difference);
        let origin = Point::new(0.0, 0.0, -0.5);
        let dir = Vector::new(0.0, 0.0, -1.0);
        let ray = Ray::new(origin, dir);
        let int = csg.ray_intersection(&ray);
        match int {
            Some(_int) => {
                panic!(
                    "error using difference for csg: ray with origin in the subtracted part of the sphere\
                is intersecting the subtracted side"
                );
            }
            None => {}
        }
    }

    #[test]
    fn test_difference_from_inside_y() {
        let material = Material::default();

        let object1: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, 0.0)),
            material.clone(),
        ));
        let object2: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, -1.0, 0.0)),
            material,
        ));
        let csg = CSG::new(object1, object2, OperationsCSGType::Difference);
        let origin = Point::new(0.0, -0.5, 0.0);
        let dir = Vector::new(0.0, -1.0, 0.0);
        let ray = Ray::new(origin, dir);
        let int = csg.ray_intersection(&ray);
        match int {
            Some(_int) => {
                panic!(
                    "error using difference for csg: ray with origin in the subtracted part of the sphere\
                is intersecting the subtracted side"
                );
            }
            None => {}
        }
    }

    #[test]
    fn test_difference_from_inside_z_2() {
        let material = Material::default();

        let object1: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, 0.0)),
            material.clone(),
        ));
        let object2: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, -1.0)),
            material,
        ));
        let csg = CSG::new(object1, object2, OperationsCSGType::Difference);
        let origin = Point::new(0.0, 0.0, -0.5);
        let dir = Vector::new(0.0, 0.0, 1.0);
        let ray = Ray::new(origin, dir);
        let int = csg.ray_intersection(&ray);
        match int {
            Some(int) => {
                let point = Point::new(0.0, 0.0, 0.0);
                assert_eq!(point, int.world_point);
            }
            None => {
                panic!(
                    "error using difference for csg: ray with origin in the subtracted part of the sphere\
                is not intersecting"
                );
            }
        }
    }

    #[test]
    fn test_difference_from_inside_y_2() {
        let material = Material::default();

        let object1: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, 0.0)),
            material.clone(),
        ));
        let object2: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, -1.0, 0.0)),
            material,
        ));
        let csg = CSG::new(object1, object2, OperationsCSGType::Difference);
        let origin = Point::new(0.0, -0.5, 0.0);
        let dir = Vector::new(0.0, 1.0, 0.0);
        let ray = Ray::new(origin, dir);
        let int = csg.ray_intersection(&ray);
        match int {
            Some(int) => {
                let point = Point::new(0.0, 0.0, 0.0);
                assert_eq!(point, int.world_point);
            }
            None => {
                panic!(
                    "error using difference for csg: ray with origin in the subtracted part of the sphere\
                is not intersecting"
                );
            }
        }
    }

    #[test]
    fn test_difference_from_inside_t2_sub_behind_ray_origin_z() {
        let material = Material::default();

        let object1: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, 0.0)),
            material.clone(),
        ));
        let object2: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, -1.0)),
            material,
        ));
        let csg = CSG::new(object1, object2, OperationsCSGType::Difference);
        let origin = Point::new(0.0, 0.0, 0.5);
        let dir = Vector::new(0.0, 0.0, 1.0);
        let ray = Ray::new(origin, dir);
        let int = csg.ray_intersection(&ray);
        match int {
            Some(int) => {
                let point = Point::new(0.0, 0.0, 1.0);
                assert_eq!(point, int.world_point)
            }
            None => {
                panic!(
                    "should intersect object 1 in t2 (exit): {:?}",
                    int.unwrap().world_point
                )
            }
        }
    }
    #[test]
    fn test_difference_from_inside_t2_sub_behind_ray_origin_y() {
        let material = Material::default();

        let object1: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, 0.0)),
            material.clone(),
        ));
        let object2: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, -1.0, 0.0)),
            material,
        ));
        let csg = CSG::new(object1, object2, OperationsCSGType::Difference);
        let origin = Point::new(0.0, 0.5, 0.0);
        let dir = Vector::new(0.0, 1.0, 0.0);
        let ray = Ray::new(origin, dir);
        let int = csg.ray_intersection(&ray);
        match int {
            Some(int) => {
                let point = Point::new(0.0, 1.0, 0.0);
                assert_eq!(point, int.world_point)
            }
            None => {
                panic!("should intersect object 1 in t2 (exit)")
            }
        }
    }

    #[test]
    fn test_difference_without_intersections() {
        let material = Material::default();

        let object1: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, 0.0)),
            material.clone(),
        ));
        let object2: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, 10.0)),
            material,
        ));
        let csg = CSG::new(object1, object2, OperationsCSGType::Difference);
        let origin = Point::new(-10.0, 0.0, 0.0);
        let dir = Vector::new(1.0, 0.0, 0.0);
        let ray = Ray::new(origin, dir);
        let int = csg.ray_intersection(&ray).unwrap();
        assert_eq!(9.0, int.t);
        assert_eq!(Point::new(-1.0, 0.0, 0.0), int.world_point);

        // -------------------
        let material = Material::default();

        let object1: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, 0.0)),
            material.clone(),
        ));
        let object2: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, -10.0)),
            material,
        ));
        let csg = CSG::new(object1, object2, OperationsCSGType::Difference);
        let origin = Point::new(-10.0, 0.0, 0.0);
        let dir = Vector::new(1.0, 0.0, 0.0);
        let ray = Ray::new(origin, dir);
        let int = csg.ray_intersection(&ray).unwrap();
        assert_eq!(9.0, int.t);
        assert_eq!(Point::new(-1.0, 0.0, 0.0), int.world_point);

        // -------------------
        let material = Material::default();

        let object1: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, 0.0)),
            material.clone(),
        ));
        let object2: Box<dyn Volumetric> = Box::new(Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, -100.0)),
            material,
        ));
        let csg = CSG::new(object1, object2, OperationsCSGType::Difference);
        let origin = Point::new(-10.0, 0.0, 0.0);
        let dir = Vector::new(1.0, 0.0, 0.0);
        let ray = Ray::new(origin, dir);
        let int = csg.ray_intersection(&ray).unwrap();
        assert_eq!(9.0, int.t);
        assert_eq!(Point::new(-1.0, 0.0, 0.0), int.world_point);
    }
}
