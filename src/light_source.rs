//! Light source types for direct illumination.
//!
//! Defines the [`LightSource`] trait and two implementations:
//! - [`PointLightSource`]: a single infinitesimal point emitter with shadow testing.
//! - [`SphericalLightSource`]: an area light approximated by Monte Carlo sampling
//!   over a disk of configurable radius and sample count.

use crate::color::{BLACK, Color};
use crate::geometry::{Dot, Normal, Point, Vector, branchless_onb};
use crate::hit_record::HitRecord;
use crate::pcg::PCG;
use crate::ray::Ray;
use crate::world::World;
use anyhow::Result;
// =================================================================
// Traits
// =================================================================

pub trait CloneLightSource {
    fn clone_light_source(&self) -> Box<dyn LightSource>;
}

impl<T> CloneLightSource for T
where
    T: LightSource + Clone + 'static,
{
    fn clone_light_source(&self) -> Box<dyn LightSource> {
        Box::new(self.clone())
    }
}
pub trait LightSource: CloneLightSource {
    /// Computes the direct illumination contribution of a light source at a hit point.
    ///
    /// Returns [`BLACK`] for points in shadow.
    fn source_contribution(
        &self,
        hit_record: &HitRecord,
        world: &World,
        pcg: &mut PCG,
    ) -> Result<Color>;
}

impl Clone for Box<dyn LightSource> {
    fn clone(&self) -> Box<dyn LightSource> {
        self.clone_light_source()
    }
}

// =================================================================
// PointLightSource
// =================================================================

/// A point light source at a fixed position in world space.
///
/// Casts a shadow ray toward the light; if unoccluded, applies
/// `contribution = pigment × light_color × max(n·l, 0)`.
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct PointLightSource {
    pub point: Point,
    pub color: Color,
}

impl PointLightSource {
    pub fn new(point: Point, color: Color) -> Self {
        Self { point, color }
    }
}

impl LightSource for PointLightSource {
    fn source_contribution(
        &self,
        hit_record: &HitRecord,
        world: &World,
        _pcg: &mut PCG,
    ) -> Result<Color> {
        let hit_point = hit_record.world_point;
        let normal = hit_record.normal;
        let pigment_color = hit_record.material.pigment.get_color(&hit_record.uv)?;

        let dir_to_light = self.point - hit_point;
        let distance_to_light = dir_to_light.norm(); // Norm calculated once
        let l_dir = dir_to_light / distance_to_light; // Normalized light direction

        let shadow_origin = hit_point + Vector::from(normal) * 1e-4;
        let shadow_ray = Ray::new(shadow_origin, l_dir);

        let in_shadow = world
            .ray_intersection(&shadow_ray)
            .map(|hit| hit.t > 1e-4 && hit.t < distance_to_light - 1e-4)
            .unwrap_or(false);

        if !in_shadow {
            let n_dot_l = normal.dot(&l_dir).max(0.0);

            // Add contribution: Pigment * Light * Angle Falloff
            return Ok((pigment_color * self.color) * n_dot_l);
        }
        Ok(BLACK)
    }
}

// =================================================================
// SphericalLightSource
// =================================================================

/// A spherical area light approximated by uniform disk sampling.
///
/// Samples `n_points` positions uniformly over a disk of the given `radius`
/// centered at `center`, averages their [`PointLightSource`] contributions.
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct SphericalLightSource {
    pub center: Point,
    pub radius: f32,
    pub color: Color,
    pub n_points: usize,
}

impl SphericalLightSource {
    pub fn new(center: Point, radius: f32, color: Color, n_points: usize) -> Self {
        Self {
            center,
            radius,
            color,
            n_points,
        }
    }
}

impl LightSource for SphericalLightSource {
    fn source_contribution(
        &self,
        hit_record: &HitRecord,
        world: &World,
        pcg: &mut PCG,
    ) -> Result<Color> {
        let mut color: Color = BLACK;
        let normal = Normal::from((hit_record.world_point - self.center).normalize());
        let (e1, e2, _) = branchless_onb(normal);

        for _ in 0..self.n_points {
            let r = self.radius * pcg.random_float().sqrt();
            let phi = 2.0 * std::f32::consts::PI * pcg.random_float();
            let vec = r * phi.cos() * e1 + r * phi.sin() * e2;
            let point = self.center + vec;
            let light_source = PointLightSource::new(point, self.color);
            color += light_source.source_contribution(hit_record, world, pcg)?;
        }

        Ok(color / self.n_points as f32)
    }
}

// =================================================================
// Tests
// =================================================================
#[cfg(test)]

mod tests {
    use super::*;
    use crate::brdf::DiffusiveBrdf;
    use crate::functions::{IDENTITY_4X4, Within};
    use crate::geometry::{Normal, Vec2D, X_AXIS, Y_AXIS, Z_AXIS};
    use crate::materials::Material;
    use crate::pigments::UniformPigment;
    use crate::shapes::{Plane, Shape, Sphere};
    use crate::transformations::{Scaling, Transformation, Translation};
    #[test]
    fn test_point_light_source_constructor() {
        let color = Color::new(1.0, 2.0, 3.0);
        let point = Point::new(4.0, 5.0, 6.0);
        let light = PointLightSource::new(point, color);
        assert_eq!(color, light.color);
        assert_eq!(point, light.point);
    }

    fn setup1() -> (
        Material,
        Material,
        PointLightSource,
        PointLightSource,
        World,
    ) {
        let color1 = Color::new(1.0, 0.1, 0.4);
        let color2 = Color::new(0.1, 0.2, 0.3);
        let blue = Color::new(0.0, 0.0, 1.0);
        let red = Color::new(10.0, 0.0, 0.0);
        let pigment1 = UniformPigment::new(color1);
        let brdf = DiffusiveBrdf {};
        let emission = UniformPigment::new(BLACK);
        let material1 = Material::new(pigment1, brdf, emission.clone()).unwrap();
        let transformation1 = Translation::new(Z_AXIS * 2.0);
        let sphere1 = Sphere::new(transformation1, material1.clone());

        let pigment2 = UniformPigment::new(color2);
        let transformation2 = Translation::new(Z_AXIS * 6.5) * Scaling::from(0.5);
        let material2 = Material::new(pigment2, brdf, emission).unwrap();
        let sphere2 = Sphere::new(transformation2, material2.clone());

        let objects: Vec<Box<dyn Shape>> = vec![Box::new(sphere1), Box::new(sphere2)];
        let light1 = PointLightSource::new(Point::new(-10.0, 00.0, 2.0), blue);
        let light2 = PointLightSource::new(Point::new(0.0, 0.0, 10.0), red);
        let world = World {
            objects: objects.clone(),
            light_sources: vec![Box::new(light1.clone()), Box::new(light2.clone())],
        };

        (material1, material2, light1, light2, world)
    }

    #[test]
    fn test_point_light_source_source_contribution_shadow() {
        let (material1, _, _, light2, world) = setup1();

        let world_point = Point::new(0.0, 0.0, 1.0);
        let hit_point = HitRecord {
            world_point,
            normal: Normal::from(-Z_AXIS),
            uv: world.objects[0].point_to_uv(&world_point).unwrap(),
            t: 1.0,
            ray: Ray::new(Point::new(0.0, 0.0, 0.0), Z_AXIS),
            material: &material1.clone(),
        };

        let expected_color = BLACK;

        let result_color = light2
            .source_contribution(&hit_point, &world, &mut PCG::default())
            .unwrap();
        assert!(result_color.is_close(&expected_color));
    }

    #[test]
    fn test_point_light_source_source_contribution_colored() {
        let (material1, _, light1, _, world) = setup1();

        let world_point = Point::new(-1.0, 0.0, 2.0);
        let hit_point = HitRecord {
            world_point,
            normal: Normal::from(-X_AXIS),
            uv: world.objects[0].point_to_uv(&world_point).unwrap(),
            t: 14.0,
            ray: Ray::new(Point::new(-15.0, 0.0, 2.0), X_AXIS),
            material: &material1.clone(),
        };

        let expected_color = Color::new(1.0, 0.1, 0.4) * Color::new(0.0, 0.0, 1.0);

        let result_color = light1
            .source_contribution(&hit_point, &world, &mut PCG::default())
            .unwrap();
        assert!(result_color.is_close(&expected_color), "{:?}", result_color);
    }

    #[test]
    fn test_point_light_source_source_contribution_cosine() {
        let color2 = Color::new(0.1, 0.2, 0.3);
        let pigment2 = UniformPigment::new(color2);
        let brdf = DiffusiveBrdf {};
        let emission = UniformPigment::new(BLACK);
        let material2 = Material::new(pigment2, brdf, emission).unwrap();
        let plane = Plane::new(Transformation::new(IDENTITY_4X4), material2.clone(), false);

        let y = 3.0_f32.sqrt() / 2.0;
        let z = 0.5;
        let point = Point::new(0.0, y, z);

        let light = PointLightSource::new(point, Color::new(1.0, 2.0, 3.0));

        let world = World {
            objects: vec![Box::new(plane)],
            light_sources: vec![Box::new(light)],
        };

        let hit_point = HitRecord {
            world_point: Point::new(0.0, 0.0, 0.0),
            normal: Normal::from(Z_AXIS),
            uv: Vec2D::new(0.0, 0.0),
            t: 1.0,
            ray: Ray::new(point, -(y * Y_AXIS + z * Z_AXIS)),
            material: &material2.clone(),
        };

        let result_color = light
            .source_contribution(&hit_point, &world, &mut PCG::default())
            .unwrap();
        let expected_color = color2 * Color::new(1.0, 2.0, 3.0) * 60.0_f32.to_radians().cos();
        assert!(result_color.is_close(&expected_color), "{:?}", result_color);
    }

    #[test]
    fn test_spherical_light_source_constructor() {
        let color: Color = Color::new(1.0, 2.0, 3.0);
        let point: Point = Point::new(4.0, 5.0, 6.0);
        let n_points: usize = 150;
        let radius: f32 = 1.5;

        let sun = SphericalLightSource::new(point, radius, color, n_points);

        assert_eq!(sun.color, color);
        assert_eq!(sun.radius, 1.5);
        assert_eq!(sun.n_points, 150);
        assert_eq!(sun.center, point);
    }

    fn give_sun() -> SphericalLightSource {
        let color: Color = Color::new(1.0, 2.0, 3.0);
        let point: Point = Point::new(0.0, 0.0, -1.0);
        let n_points: usize = 150;
        let radius: f32 = 1.0;
        SphericalLightSource::new(point, radius, color, n_points)
    }

    #[test]
    fn test_spherical_light_source_contribution_hit() {
        // Build the light
        let sun = give_sun();
        // World
        let (material1, _, _, _, world) = setup1();
        let world_point = Point::new(0.0, 0.0, 1.0);
        let hit_point = HitRecord {
            world_point,
            normal: Normal::from(-Z_AXIS),
            uv: world.objects[0].point_to_uv(&world_point).unwrap(),
            t: 1.0,
            ray: Ray::new(world_point - Z_AXIS, Z_AXIS),
            material: &material1,
        };

        let expected_color = Color::new(1.0, 0.1, 0.4) * Color::new(1.0, 2.0, 3.0);
        println!("\n{:?}\n", expected_color);
        let result_color = sun
            .source_contribution(&hit_point, &world, &mut PCG::default())
            .unwrap();

        assert!(
            result_color
                .r
                .is_between_open(&(expected_color.r - 1.0), &(expected_color.r + 1.0_f32)),
            "red: {}",
            result_color.r
        );
        assert!(
            result_color
                .g
                .is_between_open(&(expected_color.g - 1.0), &(expected_color.g + 1.0_f32)),
            "green: {}",
            result_color.g
        );
        assert!(
            result_color
                .b
                .is_between_open(&(expected_color.b - 1.0), &(expected_color.b + 1.0_f32)),
            "blue: {}",
            result_color.b
        );
    }

    #[test]
    fn test_spherical_light_source_contribution_shadow() {
        let sun = give_sun();
        let (_, material2, _, _, world) = setup1();
        let world_point = Point::new(0.0, 0.0, 8.0);
        let hit_point = HitRecord {
            world_point,
            normal: Normal::from(Z_AXIS),
            uv: world.objects[1].point_to_uv(&world_point).unwrap(),
            t: 1.0,
            ray: Ray::new(world_point + Z_AXIS, -Z_AXIS),
            material: &material2,
        };

        let expected_color = BLACK;
        let result_color = sun
            .source_contribution(&hit_point, &world, &mut PCG::default())
            .unwrap();
        assert!(result_color.is_close(&expected_color), "{:?}", result_color);
    }
}
