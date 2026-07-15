// This file is licensed under the EUPL-1.2. See LICENSE.md.

//! Constructive Solid Geometry module.

use crate::functions::{Within, are_close};
use crate::geometry::{Dot, Normal, Point, Vec2D};
use crate::hit_record::HitRecord;
use crate::materials::Material;
use crate::ray::Ray;
use crate::shapes::{Shape, Volumetric};
use crate::transformations::Transformation;
use std::cmp::min;

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

    pub fn logic_for_csg(
        &self,
        event: &Event,
        ray: &Ray,
        is_in_a: bool,
        is_in_b: bool,
        old_state_a: bool,
        old_state_b: bool,
        is_subtracted: bool,
    ) -> Option<HitRecord> {
        match self.operation {
            OperationsCSGType::Intersection => {
                if !old_state_a || !old_state_b {
                    if is_in_a && is_in_b {
                        let mut hit = match event.kind {
                            EventKind::EnterA | EventKind::ExitA => {
                                self.object1.hit_from_t(ray, event.t, is_subtracted)?
                            }
                            EventKind::ExitB | EventKind::EnterB => {
                                self.object2.hit_from_t(ray, event.t, is_subtracted)?
                            }
                        };

                        hit.material = self.object1.material();
                        return Some(hit);
                    }
                } else {
                    //if the ray origin is inside the intersected part
                    if !is_in_a || !is_in_b {
                        let mut hit = match event.kind {
                            EventKind::EnterA | EventKind::ExitA => {
                                self.object1.hit_from_t(ray, event.t, is_subtracted)?
                            }
                            EventKind::ExitB | EventKind::EnterB => {
                                self.object2.hit_from_t(ray, event.t, is_subtracted)?
                            }
                        };

                        hit.material = self.object1.material();
                        return Some(hit);
                    }
                }
                None
            }
            OperationsCSGType::Union => {
                if !old_state_a && !old_state_b {
                    if (is_in_a || is_in_b) {
                        let mut hit = match event.kind {
                            EventKind::EnterA | EventKind::ExitA => {
                                self.object1.hit_from_t(ray, event.t, is_subtracted)?
                            }
                            EventKind::ExitB | EventKind::EnterB => {
                                self.object2.hit_from_t(ray, event.t, is_subtracted)?
                            }
                        };

                        hit.material = self.object1.material();
                        return Some(hit);
                    }
                    None
                } else {
                    if !is_in_a && !is_in_b {
                        let mut hit = match event.kind {
                            EventKind::EnterA | EventKind::ExitA => {
                                self.object1.hit_from_t(ray, event.t, is_subtracted)?
                            }
                            EventKind::ExitB | EventKind::EnterB => {
                                self.object2.hit_from_t(ray, event.t, is_subtracted)?
                            }
                        };

                        hit.material = self.object1.material();
                        return Some(hit);
                    }
                    None
                }
            }
            OperationsCSGType::Difference => {
                if old_state_b || !old_state_a {
                    if is_in_a && !is_in_b {
                        let mut hit = match event.kind {
                            EventKind::EnterA | EventKind::ExitA => {
                                self.object1.hit_from_t(ray, event.t, is_subtracted)?
                            }
                            EventKind::ExitB | EventKind::EnterB => {
                                self.object2.hit_from_t(ray, event.t, !is_subtracted)?
                            }
                        };

                        hit.material = self.object1.material();
                        return Some(hit);
                    }
                    None
                } else {
                    if !is_in_a || is_in_b {
                        let mut hit = match event.kind {
                            EventKind::EnterA | EventKind::ExitA => {
                                self.object1.hit_from_t(ray, event.t, is_subtracted)?
                            }
                            EventKind::ExitB | EventKind::EnterB => {
                                self.object2.hit_from_t(ray, event.t, !is_subtracted)?
                            }
                        };

                        hit.material = self.object1.material();
                        return Some(hit);
                    }
                    None
                }
            }
        }
    }
}

impl Shape for CSG {
    fn ray_intersection(&self, ray: &Ray) -> Option<HitRecord<'_>> {
        let mut events = Vec::new();

        self.fill_intersection_vector(ray, &mut events, false);

        events.sort_by(|e1, e2| e1.t.partial_cmp(&e2.t).unwrap());

        let mut is_in_a = false;
        let mut is_in_b = false;
        let mut old_state_a = false;
        let mut old_state_b = false;

        for event in events {
            match event.kind {
                EventKind::EnterA => {
                    is_in_a = true;
                    if event.t < 0.0 || are_close(event.t, 0.0) {
                        old_state_a = true;
                    }
                }
                EventKind::ExitA => {
                    is_in_a = false;
                    if event.t < 0.0 || are_close(event.t, 0.0) {
                        old_state_a = false;
                    }
                }
                EventKind::EnterB => {
                    is_in_b = true;
                    if event.t < 0.0 || are_close(event.t, 0.0) {
                        old_state_b = true;
                    }
                }
                EventKind::ExitB => {
                    is_in_b = false;
                    if event.t < 0.0 || are_close(event.t, 0.0) {
                        old_state_b = false;
                    }
                }
            }
            if event.t > 0.0 && !are_close(event.t, 0.0) {
                match self.logic_for_csg(
                    &event,
                    ray,
                    is_in_a,
                    is_in_b,
                    old_state_a,
                    old_state_b,
                    false,
                ) {
                    Some(hit) => {
                        return Some(hit);
                    }
                    None => {}
                }
            }
        }
        None
    }

    fn normal_at(&self, point: Point, ray: &Ray) -> Normal {
        let mut events = Vec::new();
        self.fill_intersection_vector(ray, &mut events, false);
        events.sort_by(|e1, e2| e1.t.partial_cmp(&e2.t).unwrap());

        for event in events {
            if are_close((ray.origin + ray.dir * event.t).x, point.x)
                && are_close((ray.origin + ray.dir * event.t).y, point.y)
                && are_close((ray.origin + ray.dir * event.t).z, point.z)
            {
                return match event.kind {
                    EventKind::EnterA => self.object1.normal_at(point, ray),
                    EventKind::ExitA => self.object1.normal_at(point, ray),
                    EventKind::EnterB => self.object2.normal_at(point, ray),
                    EventKind::ExitB => self.object2.normal_at(point, ray),
                };
            }
        }
        panic!("could not compute normal")
    }

    fn point_to_uv(&self, point: &Point) -> anyhow::Result<Vec2D> {
       unreachable!("there is still no implementation for this function")
    }

    // Might be useful to change it in Shape definition
    // to feature the possibility of having more than one material
    fn material(&self) -> &Material {
        &self.object1.material()
    }
}

impl Volumetric for CSG {
    /// returns first entry and first exit: to see the next interval modify origin of ray
    /// and set it to the exit point of the previous interval
    fn entry_exit_t(&self, ray: &Ray, is_subtracted: bool) -> Option<(f32, f32)> {
        /*let mut events = Vec::new();
                self.fill_intersection_vector(ray, & mut events, is_subtracted);
                events.sort_by(|e1, e2| e1.t.partial_cmp(&e2.t).unwrap());

                let mut is_in_a = false;
                let mut is_in_b = false;

                for event in events {
                    match event.kind {
                        EventKind::EnterA => is_in_a = true,
                        EventKind::ExitA => is_in_a = false,
                        EventKind::EnterB => is_in_b = true,
                        EventKind::ExitB => is_in_b = false,
                        _ => {}
                    }
        todo!();
        r

                }*/
        return None;
    }

    fn hit_from_t(&self, ray: &Ray, t: f32, is_subtracted: bool) -> Option<HitRecord> {
        let hit_a = self.object1.hit_from_t(ray, t, false);
        let hit_b = self.object2.hit_from_t(ray, t, false);
        return match hit_a {
            Some(hit_a) => match hit_b {
                Some(hit_b) => {
                    if hit_a.t < hit_b.t && !are_close(hit_a.t, hit_b.t) {
                        return Some(hit_a);
                    }
                    Some(hit_b)
                }
                None => Some(hit_a),
            },
            None => hit_b,
        };
    }

    fn fill_intersection_vector(&self, ray: &Ray, vec: &mut Vec<Event>, is_subtracted: bool) {
        self.object1
            .fill_intersection_vector(ray, vec, true == is_subtracted);
        match self.operation {
            OperationsCSGType::Difference => {
                self.object2
                    .fill_intersection_vector(ray, vec, false == is_subtracted);
            }
            _ => {
                self.object2
                    .fill_intersection_vector(ray, vec, true == is_subtracted);
                // println!("AAAAAAAAAAAAAAAAAAAAAAA");
            }
        }
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
