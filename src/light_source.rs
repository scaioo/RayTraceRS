use crate::color::{BLACK, Color};
use crate::functions::Within;
use crate::geometry::{Dot, Point, Vec2D, Vector};
use crate::hit_record::HitRecord;
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
    fn source_contribution(&self, hit_record: &HitRecord, world: &World) -> Result<Color>;
}

impl Clone for Box<dyn LightSource> {
    fn clone(&self) -> Box<dyn LightSource> {
        self.clone_light_source()
    }
}

// =================================================================
// PointLightSource
// =================================================================

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
    fn source_contribution(&self, hit_record: &HitRecord, world: &World) -> Result<Color> {
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
// Tests
// =================================================================
#[cfg(test)]

mod tests {
    use super::*;
    use crate::brdf::DiffusiveBrdf;
    use crate::functions::IDENTITY_4X4;
    use crate::geometry::{Normal, X_AXIS, Y_AXIS, Z_AXIS};
    use crate::materials::Material;
    use crate::pigments::UniformPigment;
    use crate::shapes::{Plane, Shape, Sphere};
    use crate::transformations::{IsHomogeneousMatrix, Scaling, Transformation, Translation};
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
        let color1 = Color::new(10.0, 1.0, 4.0);
        let color2 = Color::new(1.0, 2.0, 3.0);
        let blue = Color::new(0.0, 0.0, 1.0);
        let red = Color::new(10.0, 0.0, 0.0);
        let pigment1 = UniformPigment::new(color1);
        let brdf = DiffusiveBrdf {};
        let emission = UniformPigment::new(BLACK);
        let material1 = Material::new(pigment1, brdf, emission.clone());
        let transformation1 = Translation::new(Z_AXIS * 2.0);
        let sphere1 = Sphere::new(transformation1, material1.clone());

        let pigment2 = UniformPigment::new(color2);
        let transformation2 = Translation::new(Z_AXIS * 6.5) * Scaling::new([0.5, 0.5, 0.5]);
        let material2 = Material::new(pigment2, brdf, emission);
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

        let result_color = light2.source_contribution(&hit_point, &world).unwrap();
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

        let expected_color = Color::new(10.0, 1.0, 4.0) * Color::new(0.0, 0.0, 1.0);

        let result_color = light1.source_contribution(&hit_point, &world).unwrap();
        assert!(result_color.is_close(&expected_color), "{:?}", result_color);
    }

    #[test]
    fn test_point_light_source_source_contribution_cosine() {
        let color2 = Color::new(1.0, 2.0, 3.0);
        let pigment2 = UniformPigment::new(color2);
        let brdf = DiffusiveBrdf {};
        let emission = UniformPigment::new(BLACK);
        let material2 = Material::new(pigment2, brdf, emission);
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

        let result_color = light.source_contribution(&hit_point, &world).unwrap();
        let expected_color = color2 * Color::new(1.0, 2.0, 3.0) * 60.0_f32.to_radians().cos();
        assert!(result_color.is_close(&expected_color), "{:?}", result_color);
    }
}
