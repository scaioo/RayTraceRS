use crate::brdf::{BRDF, DiffusiveBrdf, SpecularBrdf};
use crate::camera::{Camera, OrthogonalCamera, PerspectiveCamera};
use crate::color::{BLACK, Color, WHITE};
use crate::geometry::{Point, Vector};
use crate::lexer::{InputStream, Keyword, TokenKind};
use crate::light_source::{PointLightSource, SphericalLightSource};
use crate::materials::Material;
use crate::mesh::SimpleMesh;
use crate::pfm_func::read_pfm_file;
use crate::pigments::{CheckeredPigment, GradientPigment, ImagePigment, Pigment, UniformPigment};
use crate::shapes::{AABB, Plane, Sphere};
use crate::transformations::{
    Scaling, Transformation, Translation, XRotation, YRotation, ZRotation,
};
use crate::world::World;
use anyhow::{Result, anyhow, bail};
use std::collections::{HashMap, HashSet};
use std::io::BufRead;
use std::path::PathBuf;

/// A scene read from a scene file.
///
/// It acts as a container for all the elements parsed from the script,
/// acting as a symbol table for variables and materials, and storing
/// the final 3D world and camera.
pub struct Scene {
    /// A dictionary (symbol table) storing parsed materials by their name.
    pub materials: HashMap<String, Material>,

    /// The geometric shapes parsed into the scene.
    pub world: World,

    /// The observer. It's an Option because it might not have been parsed yet.
    /// We use `Box<dyn Camera>` because `Camera` is a trait.
    pub camera: Option<Box<dyn Camera>>,

    /// A dictionary storing user-defined floating point variables.
    pub float_variables: HashMap<String, f32>,

    /// A set keeping track of variables that have been overridden from the command line.
    pub overridden_variables: HashSet<String>,
}

impl Scene {
    /// Creates a new, empty Scene with default values.
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
            world: World {
                objects: vec![],
                light_sources: vec![],
            },
            camera: None,
            float_variables: HashMap::new(),
            overridden_variables: HashSet::new(),
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

// ==========================================
// EXPECT FUNCTIONS
// ==========================================

/// Expects to find a specific symbol  (ex. '[', '<', ',').
pub fn expect_symbol<B: BufRead>(stream: &mut InputStream<B>, symbol: char) -> Result<()> {
    let token = stream.read_token()?;
    match token.kind {
        TokenKind::Symbol(s) if s == symbol => Ok(()),
        _ => bail!(
            "Grammar Error at {}:{}: expected symbol '{}', found '{:?}'",
            token.loc.line_number,
            token.loc.col_number,
            symbol,
            token.kind
        ),
    }
}

/// Expects to find one of the keywords provided in the array.
/// Returns the keyword that was actually found.
pub fn expect_keywords<B: BufRead>(
    stream: &mut InputStream<B>,
    keywords: &[Keyword],
) -> Result<Keyword> {
    let token = stream.read_token()?;
    match token.kind {
        TokenKind::Keyword(k) => {
            if keywords.contains(&k) {
                Ok(k)
            } else {
                bail!(
                    "Grammar Error at {}:{}: expected one of {:?}, found keyword '{:?}'",
                    token.loc.line_number,
                    token.loc.col_number,
                    keywords,
                    k
                )
            }
        }
        _ => bail!(
            "Grammar Error at {}:{}: expected a keyword, found '{:?}'",
            token.loc.line_number,
            token.loc.col_number,
            token.kind
        ),
    }
}

/// Expects to find a number and extracts its f32 value.
/// If it finds an identifier, it looks up its value in `scene.float_variables`.
pub fn expect_number<B: BufRead>(stream: &mut InputStream<B>, scene: &Scene) -> Result<f32> {
    let token = stream.read_token()?;
    match token.kind {
        // Case 1: It is a numeric literal (e.g. 3.14)
        TokenKind::LiteralNumber(n) => Ok(n),

        // Case 2: It is a variable (e.g. “clock”)
        TokenKind::Identifier(name) => {
            if let Some(&value) = scene.float_variables.get(&name) {
                Ok(value)
            } else {
                bail!(
                    "Grammar Error at {}:{}: undefined variable '{}'",
                    token.loc.line_number,
                    token.loc.col_number,
                    name
                )
            }
        }

        // Case 3: Syntax error
        _ => bail!(
            "Grammar Error at {}:{}: expected a number, found '{:?}'",
            token.loc.line_number,
            token.loc.col_number,
            token.kind
        ),
    }
}

/// It expects to find a string enclosed in quotation marks and extracts its content.
pub fn expect_string<B: BufRead>(stream: &mut InputStream<B>) -> Result<String> {
    let token = stream.read_token()?;
    match token.kind {
        TokenKind::LiteralString(s) => Ok(s),
        _ => bail!(
            "Grammar Error at {}:{}: expected a string, found '{:?}'",
            token.loc.line_number,
            token.loc.col_number,
            token.kind
        ),
    }
}

/// It expects to find an identifier (e.g. a name given to a material) and extracts it.
pub fn expect_identifier<B: BufRead>(stream: &mut InputStream<B>) -> Result<String> {
    let token = stream.read_token()?;
    match token.kind {
        TokenKind::Identifier(id) => Ok(id),
        _ => bail!(
            "Grammar Error at {}:{}: expected an identifier, found '{:?}'",
            token.loc.line_number,
            token.loc.col_number,
            token.kind
        ),
    }
}

// ==========================================
// PARSER FUNCTIONS
// ==========================================
/// Parses a vector in the format `[x, y, z]`
pub fn parse_vector<B: BufRead>(stream: &mut InputStream<B>, scene: &Scene) -> Result<Vector> {
    expect_symbol(stream, '[')?;
    let x = expect_number(stream, scene)?;
    expect_symbol(stream, ',')?;
    let y = expect_number(stream, scene)?;
    expect_symbol(stream, ',')?;
    let z = expect_number(stream, scene)?;
    expect_symbol(stream, ']')?;

    Ok(Vector::new(x, y, z))
}

/// Parses a color in the format `<r, g, b>`
pub fn parse_color<B: BufRead>(stream: &mut InputStream<B>, scene: &Scene) -> Result<Color> {
    let token = stream.read_token()?;
    match token.kind {
        TokenKind::Keyword(Keyword::WHITE) => Ok(WHITE),
        TokenKind::Keyword(Keyword::BLACK) => Ok(BLACK),
        TokenKind::Symbol('<') => {
            let r = expect_number(stream, scene)?;
            expect_symbol(stream, ',')?;
            let g = expect_number(stream, scene)?;
            expect_symbol(stream, ',')?;
            let b = expect_number(stream, scene)?;
            expect_symbol(stream, '>')?;

            Ok(Color::new(r, g, b))
        }
        _ => bail!(
            "Grammar Error at {}:{}: expected '<', 'black', or 'white', found '{:?}'",
            token.loc.line_number,
            token.loc.col_number,
            token.kind
        ),
    }
}

pub fn parse_point<B: BufRead>(stream: &mut InputStream<B>, scene: &Scene) -> Result<Point> {
    expect_symbol(stream, '(')?;
    let v = parse_vector(stream, scene)?;
    expect_symbol(stream, ')')?;
    Ok(Point::new(v.x, v.y, v.z))
}

/// Parses a pigment. Supports uniform, checkered, and image pigments.
pub fn parse_pigment<B: BufRead>(
    stream: &mut InputStream<B>,
    scene: &Scene,
) -> Result<Box<dyn Pigment>> {
    let keyword = expect_keywords(
        stream,
        &[
            Keyword::UNIFORM,
            Keyword::CHECKERED,
            Keyword::IMAGE,
            Keyword::GRADIENT,
        ],
    )?;
    expect_symbol(stream, '(')?;

    let result: Box<dyn Pigment> = match keyword {
        Keyword::UNIFORM => {
            let color = parse_color(stream, scene)?;
            Box::new(UniformPigment::new(color))
        }
        Keyword::CHECKERED => {
            let color1 = parse_color(stream, scene)?;
            expect_symbol(stream, ',')?;
            let color2 = parse_color(stream, scene)?;
            expect_symbol(stream, ',')?;
            let steps = expect_number(stream, scene)? as u32;
            Box::new(CheckeredPigment::new(color1, color2, steps))
        }
        Keyword::IMAGE => {
            let file_name = expect_string(stream)?;
            // Assuming you have a way to load the HDR from a file.
            // In a real scenario, you might want to cache images.
            let image = read_pfm_file(&file_name)?;
            Box::new(ImagePigment::new(image))
        }
        Keyword::GRADIENT => {
            let color1 = parse_color(stream, scene)?;
            expect_symbol(stream, ',')?;
            let color2 = parse_color(stream, scene)?;
            expect_symbol(stream, ',')?;
            let angle = expect_number(stream, scene)?;
            Box::new(GradientPigment::new(color1, color2, angle.to_radians()))
        }
        _ => unreachable!(),
    };

    expect_symbol(stream, ')')?;
    Ok(result)
}

/// Parses a BRDF. Supports diffuse and specular.
pub fn parse_brdf<B: BufRead>(stream: &mut InputStream<B>) -> Result<Box<dyn BRDF>> {
    let keyword = expect_keywords(stream, &[Keyword::DIFFUSE, Keyword::SPECULAR])?;
    expect_symbol(stream, '(')?;

    let result: Box<dyn BRDF> = match keyword {
        Keyword::DIFFUSE => Box::new(DiffusiveBrdf {}),
        Keyword::SPECULAR => Box::new(SpecularBrdf {}),
        _ => unreachable!(),
    };

    expect_symbol(stream, ')')?;
    Ok(result)
}

/// Parses a material definition: `material_name(brdf, emitted_radiance)`
pub fn parse_material<B: BufRead>(
    stream: &mut InputStream<B>,
    scene: &Scene,
) -> Result<(String, Material)> {
    let name = expect_identifier(stream)?;
    expect_symbol(stream, '(')?;

    let base_pigment = parse_pigment(stream, scene)?;
    expect_symbol(stream, ',')?;
    let brdf = parse_brdf(stream)?;
    expect_symbol(stream, ',')?;
    let emitted_radiance = parse_pigment(stream, scene)?;

    expect_symbol(stream, ')')?;

    let material = Material {
        pigment: base_pigment,
        brdf,
        emitted_radiance,
    };

    Ok((name, material))
}

/// Parses a sequence of transformations chained by `*`
pub fn parse_transformation<B: BufRead>(
    stream: &mut InputStream<B>,
    scene: &Scene,
) -> Result<Transformation> {
    let mut result = Transformation::new(crate::functions::IDENTITY_4X4);

    loop {
        let keyword = expect_keywords(
            stream,
            &[
                Keyword::IDENTITY,
                Keyword::TRANSLATION,
                Keyword::RotationX,
                Keyword::RotationY,
                Keyword::RotationZ,
                Keyword::SCALING,
            ],
        )?;

        match keyword {
            Keyword::IDENTITY => {}
            Keyword::TRANSLATION => {
                expect_symbol(stream, '(')?;
                let vec = parse_vector(stream, scene)?;
                expect_symbol(stream, ')')?;
                result = result * Translation::new(vec);
            }
            Keyword::RotationX => {
                expect_symbol(stream, '(')?;
                let angle = expect_number(stream, scene)?.to_radians();
                expect_symbol(stream, ')')?;
                result = result * XRotation::new(angle);
            }
            Keyword::RotationY => {
                expect_symbol(stream, '(')?;
                let angle = expect_number(stream, scene)?.to_radians();
                expect_symbol(stream, ')')?;
                result = result * YRotation::new(angle);
            }
            Keyword::RotationZ => {
                expect_symbol(stream, '(')?;
                let angle = expect_number(stream, scene)?.to_radians();
                expect_symbol(stream, ')')?;
                result = result * ZRotation::new(angle);
            }
            Keyword::SCALING => {
                expect_symbol(stream, '(')?;
                let token = stream.read_token()?;
                match token.kind {
                    TokenKind::Symbol('[') => {
                        stream.unread_token(token)?;
                        let vec = parse_vector(stream, scene)?;
                        result = result * Scaling::new([vec.x, vec.y, vec.z]);
                    }
                    _ => {
                        stream.unread_token(token)?;
                        let scale = expect_number(stream, scene)?;
                        result = result * Scaling::from(scale);
                    }
                }
                expect_symbol(stream, ')')?;
            }
            _ => unreachable!(),
        }

        // ====================================================
        // HANDLING THE “*” SYMBOL (BEWARE OF LOOKAHEAD!)
        // ====================================================
        let next_token = stream.read_token()?;
        if let TokenKind::Symbol(s) = next_token.kind
            && s == '*'
        {
            // There's an asterisk, so let's continue the loop to read the next transformation!
            continue;
        }

        // If we get this far, the token was NOT a “*”.
        // This means that the transformation chain has ended (e.g. we’ve found a comma or a closing bracket).
        // We must PUT the token BACK into the stream, otherwise the next parser will miss it!
        stream.unread_token(next_token)?;
        break;
    }

    Ok(result)
}

/// Parses a sphere: `sphere(material_name, transformation)`
pub fn parse_sphere<B: BufRead>(
    stream: &mut InputStream<B>,
    scene: &Scene,
) -> Result<Sphere<Transformation>> {
    expect_symbol(stream, '(')?;
    let material_name = expect_identifier(stream)?;
    expect_symbol(stream, ',')?;
    let transformation = parse_transformation(stream, scene)?;
    expect_symbol(stream, ')')?;

    let material = scene
        .materials
        .get(&material_name)
        .ok_or_else(|| anyhow!("Unknown material '{}' for sphere", material_name))?
        .clone();
    Ok(Sphere {
        transformation,
        material,
    })
}

/// Parses a plane: `plane(material_name, transformation)`
pub fn parse_plane<B: BufRead>(
    stream: &mut InputStream<B>,
    scene: &Scene,
) -> Result<Plane<Transformation>> {
    expect_symbol(stream, '(')?;
    let material_name = expect_identifier(stream)?;
    expect_symbol(stream, ',')?;
    let transformation = parse_transformation(stream, scene)?;
    let token = stream.read_token()?;
    let procedural_texture = match token.kind {
        TokenKind::Symbol(')') => false,
        TokenKind::Symbol(',') => {
            let key = expect_keywords(stream, &[Keyword::TRUE, Keyword::FALSE])?;
            expect_symbol(stream, ')')?;
            key == Keyword::TRUE
        }
        _ => bail!(
            "Grammar Error at {}:{}: expected ')' or ', <bool>', found '{:?}'",
            token.loc.line_number,
            token.loc.col_number,
            token.kind
        ),
    };

    let material = scene
        .materials
        .get(&material_name)
        .ok_or_else(|| anyhow!("Unknown material '{}' for plane", material_name))?
        .clone();

    Ok(Plane {
        transformation,
        material,
        procedural_texture,
    })
}

pub fn parse_box<B: BufRead>(stream: &mut InputStream<B>, scene: &Scene) -> Result<AABB> {
    expect_symbol(stream, '(')?;
    let material_name = expect_identifier(stream)?;
    expect_symbol(stream, ',')?;
    expect_keywords(stream, &[Keyword::POINT])?;
    let point1 = parse_point(stream, scene)?;
    expect_symbol(stream, ',')?;
    expect_keywords(stream, &[Keyword::POINT])?;
    let point2 = parse_point(stream, scene)?;
    expect_symbol(stream, ')')?;
    let material = scene
        .materials
        .get(&material_name)
        .ok_or_else(|| anyhow!("Unknown material '{}' for box", material_name))?
        .clone();
    let aabb = AABB::new(point1, point2, material)?;
    Ok(aabb)
}

pub fn parse_simple_mesh<B: BufRead>(
    stream: &mut InputStream<B>,
    scene: &Scene,
) -> Result<SimpleMesh> {
    expect_symbol(stream, '(')?;
    let material_name = expect_identifier(stream)?;
    expect_symbol(stream, ',')?;
    let path = PathBuf::from(expect_string(stream)?);
    expect_symbol(stream, ',')?;
    let transformation = parse_transformation(stream, scene)?;
    expect_symbol(stream, ')')?;
    let material = scene
        .materials
        .get(&material_name)
        .ok_or_else(|| anyhow!("Unknown material '{}' for SimpleMesh", material_name))?
        .clone();
    SimpleMesh::from_obj(&path, material, transformation)
}

pub fn parse_point_light<B: BufRead>(
    stream: &mut InputStream<B>,
    scene: &Scene,
) -> Result<PointLightSource> {
    expect_symbol(stream, '(')?;
    expect_keywords(stream, &[Keyword::POINT])?;
    let point = parse_point(stream, scene)?;
    expect_symbol(stream, ',')?;
    let color = parse_color(stream, scene)?;
    expect_symbol(stream, ')')?;
    Ok(PointLightSource::new(point, color))
}

pub fn parse_spherical_light<B: BufRead>(
    stream: &mut InputStream<B>,
    scene: &Scene,
) -> Result<SphericalLightSource> {
    expect_symbol(stream, '(')?;
    expect_keywords(stream, &[Keyword::POINT])?;
    let point = parse_point(stream, scene)?;
    expect_symbol(stream, ',')?;
    let radius = expect_number(stream, scene)?;
    expect_symbol(stream, ',')?;
    let color = parse_color(stream, scene)?;
    expect_symbol(stream, ',')?;
    let n_tests = expect_number(stream, scene)? as usize;
    expect_symbol(stream, ')')?;
    Ok(SphericalLightSource::new(point, radius, color, n_tests))
}

/// Parses a camera: `camera(perspective, transformation, aspect_ratio, distance)`
pub fn parse_camera<B: BufRead>(
    stream: &mut InputStream<B>,
    scene: &Scene,
) -> Result<Box<dyn Camera>> {
    expect_symbol(stream, '(')?;
    let cam_type = expect_keywords(stream, &[Keyword::PERSPECTIVE, Keyword::ORTHOGONAL])?;
    expect_symbol(stream, ',')?;
    let transformation = parse_transformation(stream, scene)?;
    expect_symbol(stream, ',')?;
    let distance = expect_number(stream, scene)?;
    if distance < 0.0 {
        bail!("Invalid distance: {}.\nDistance cannot be negative", distance);
    }
    expect_symbol(stream, ')')?;

    let camera: Box<dyn Camera> = match cam_type {
        Keyword::PERSPECTIVE => {
            let mut cam = PerspectiveCamera::new(transformation);
            cam.set_distance(distance);
            Box::new(cam)
        }
        Keyword::ORTHOGONAL => {
            let cam = OrthogonalCamera::new(transformation);
            Box::new(cam)
        }
        _ => unreachable!(),
    };

    Ok(camera)
}

/// Main loop: parses the entire scene file until StopToken is reached.
pub fn parse_scene<B: BufRead>(
    stream: &mut InputStream<B>,
    initial_variables: HashMap<String, f32>,
) -> Result<Scene> {
    let mut scene = Scene::new();
    scene.float_variables = initial_variables;

    loop {
        let token = stream.read_token()?;

        match token.kind {
            TokenKind::Keyword(Keyword::FLOAT) => {
                let name = expect_identifier(stream)?;
                expect_symbol(stream, '(')?;
                let value = expect_number(stream, &scene)?;
                expect_symbol(stream, ')')?;

                // Only override if it wasn't provided via command line arguments
                if !scene.overridden_variables.contains(&name) {
                    scene.float_variables.insert(name, value);
                }
            }
            TokenKind::Keyword(Keyword::MATERIAL) => {
                let (name, material) = parse_material(stream, &scene)?;
                scene.materials.insert(name, material);
            }
            TokenKind::Keyword(Keyword::SPHERE) => {
                let sphere = parse_sphere(stream, &scene)?;
                scene.world.objects.push(Box::new(sphere));
            }
            TokenKind::Keyword(Keyword::PLANE) => {
                let plane = parse_plane(stream, &scene)?;
                scene.world.objects.push(Box::new(plane));
            }
            TokenKind::Keyword(Keyword::BOX) => {
                let box_shape = parse_box(stream, &scene)?;
                scene.world.objects.push(Box::new(box_shape));
            }
            TokenKind::Keyword(Keyword::SIMPLEMESH) => {
                let simple_mesh = parse_simple_mesh(stream, &scene)?;
                scene.world.objects.push(Box::new(simple_mesh));
            }
            TokenKind::Keyword(Keyword::PTLIGHTSOURCE) => {
                let point_light_source: PointLightSource = parse_point_light(stream, &scene)?;
                scene.world.light_sources.push(Box::new(point_light_source));
            }
            TokenKind::Keyword(Keyword::SPHLIGHTSOURCE) => {
                let spherical_light_source: SphericalLightSource =
                    parse_spherical_light(stream, &scene)?;
                scene
                    .world
                    .light_sources
                    .push(Box::new(spherical_light_source));
            }
            TokenKind::Keyword(Keyword::CAMERA) => {
                let camera = parse_camera(stream, &scene)?;
                scene.camera = Some(camera);
            }
            TokenKind::StopToken => {
                break; // End of file reached!
            }
            _ => bail!(
                "Grammar Error at {}:{}: unexpected token '{:?}'. Expected float, material, sphere, plane, or camera.",
                token.loc.line_number,
                token.loc.col_number,
                token.kind
            ),
        }
    }

    Ok(scene)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::color::Color;
    use crate::geometry::{Normal, Vec2D, X_AXIS, Z_AXIS};
    use crate::hit_record::HitRecord;
    use crate::lexer::InputStream;
    use crate::pcg::PCG;
    use crate::ray::Ray;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_input_stream_python_translation() {
        // We initialize the stream with the same string as the Python test
        let text = "abc   \nd\nef";
        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);

        assert_eq!(stream.source_location.line_number, 1);
        assert_eq!(stream.source_location.col_number, 0);

        // Read 'a'
        assert_eq!(stream.read_char().unwrap(), Some('a'));
        assert_eq!(stream.source_location.line_number, 1);
        assert_eq!(stream.source_location.col_number, 1);

        // Unread 'A'
        stream.unread_char('A').unwrap();
        assert_eq!(stream.source_location.line_number, 1);
        assert_eq!(stream.source_location.col_number, 1);

        // Read 'A' again (taken from the unread buffer)
        assert_eq!(stream.read_char().unwrap(), Some('A'));
        assert_eq!(stream.source_location.line_number, 1);
        assert_eq!(stream.source_location.col_number, 1); // The column does not advance if it is marked as unread

        // Read 'b' and 'c'
        assert_eq!(stream.read_char().unwrap(), Some('b'));
        assert_eq!(stream.source_location.line_number, 1);
        assert_eq!(stream.source_location.col_number, 2);

        assert_eq!(stream.read_char().unwrap(), Some('c'));
        assert_eq!(stream.source_location.line_number, 1);
        assert_eq!(stream.source_location.col_number, 3);

        // Skip spaces ("   \n").
        // Behind the scenes it reads up to and including 'd', updates the location, and then does unread('d').
        let _ = stream.skip_whitespace().unwrap();

        // Read 'd' (which was in unread
        assert_eq!(stream.read_char().unwrap(), Some('d'));
        assert_eq!(stream.source_location.line_number, 2);
        assert_eq!(stream.source_location.col_number, 1);

        // Read '\n'
        assert_eq!(stream.read_char().unwrap(), Some('\n'));
        assert_eq!(stream.source_location.line_number, 3);
        assert_eq!(stream.source_location.col_number, 0);

        // Read 'e' and 'f'
        assert_eq!(stream.read_char().unwrap(), Some('e'));
        assert_eq!(stream.source_location.line_number, 3);
        assert_eq!(stream.source_location.col_number, 1);

        assert_eq!(stream.read_char().unwrap(), Some('f'));
        assert_eq!(stream.source_location.line_number, 3);
        assert_eq!(stream.source_location.col_number, 2);

        // End of file
        assert_eq!(stream.read_char().unwrap(), None);
    }

    #[test]
    fn test_parse_scene() -> Result<()> {
        let text = r#"
        float clock(150)

        material sky_material(
            uniform(<0, 0, 0>),
            diffuse(),
            uniform(<0.7, 0.5, 1>)
        )

        # Here is a comment

        material ground_material(
            checkered(<0.3, 0.5, 0.1>,
                      <0.1, 0.2, 0.5>, 4),
            diffuse(),
            uniform(<0, 0, 0>)
        )

        material sphere_material(
            uniform(<0.5, 0.5, 0.5>),
            specular(),
            uniform(<0, 0, 0>)
        )

        plane (sky_material, translation([0, 0, 100]) * rotation_y(clock))
        plane (ground_material, identity)

        sphere(sphere_material, translation([0, 0, 1]))

        camera(perspective, rotation_z(30) * translation([-4, 0, 1]) * scaling(1), 2.0)
        "#;

        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);

        let initial_vars = HashMap::new();
        let scene = parse_scene(&mut stream, initial_vars)?;

        // ==========================================
        // 1. Check float variables
        // ==========================================
        assert_eq!(scene.float_variables.len(), 1);
        assert!(scene.float_variables.contains_key("clock"));
        assert_eq!(scene.float_variables["clock"], 150.0);

        // ==========================================
        // 2. Check materials
        // ==========================================
        assert_eq!(scene.materials.len(), 3);
        assert!(scene.materials.contains_key("sphere_material"));
        assert!(scene.materials.contains_key("sky_material"));
        assert!(scene.materials.contains_key("ground_material"));

        let sky_material = &scene.materials["sky_material"];
        let ground_material = &scene.materials["ground_material"];
        let sphere_material = &scene.materials["sphere_material"];

        // Let's test the BEHAVIOR rather than the TYPE (no `isinstance`!)
        let uv_test = Vec2D { x: 0.0, y: 0.0 }; // A random UV spot to test the pigments

        // Sky Material
        assert!(
            sky_material
                .pigment
                .get_color(&uv_test)?
                .is_close(&Color::new(0.0, 0.0, 0.0))
        );
        assert!(
            sky_material
                .emitted_radiance
                .get_color(&uv_test)?
                .is_close(&Color::new(0.7, 0.5, 1.0))
        );

        // Ground Material
        assert!(
            ground_material
                .pigment
                .get_color(&uv_test)?
                .is_close(&Color::new(0.3, 0.5, 0.1))
        );
        assert!(
            ground_material
                .emitted_radiance
                .get_color(&uv_test)?
                .is_close(&Color::new(0.0, 0.0, 0.0))
        );

        // Sphere Material
        assert!(
            sphere_material
                .pigment
                .get_color(&uv_test)?
                .is_close(&Color::new(0.5, 0.5, 0.5))
        );
        assert!(
            sphere_material
                .emitted_radiance
                .get_color(&uv_test)?
                .is_close(&Color::new(0.0, 0.0, 0.0))
        );

        // ==========================================
        // 3. Check shapes
        // ==========================================
        assert_eq!(scene.world.objects.len(), 3);

        // ==========================================
        // 4. Check camera
        // ==========================================
        assert!(scene.camera.is_some());

        Ok(())
    }

    #[test]
    fn test_parse_gradient_box() -> Result<()> {
        let text = r#"
        # updates parsing
        material ball(
        gradient(
        <0.1, 0.2, 0.5>,
        <0.6, 0.8, 0.9>,
        0
        ), diffuse(), uniform(<0, 0, 0>)
        )

        box(
        ball, point([0.0, -1.0, 0.0]),
point([2.0, 1.0, 1.0])
        )"#;

        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);

        let initial_vars = HashMap::new();
        let scene = parse_scene(&mut stream, initial_vars)?;

        assert_eq!(
            scene.materials.len(),
            1,
            "Expected 1 material but found {}",
            scene.materials.len()
        );

        // ---- check gradient -----------------------
        let uv = Vec2D { x: 0.0, y: 0.0 };
        let expected_color = Color::new(0.1, 0.2, 0.5);
        let color = scene.materials["ball"].pigment.get_color(&uv)?;
        assert!(
            expected_color.is_close(&color),
            "expected: {:?}\n found: {:?}",
            expected_color,
            color
        );

        // ---- check box -----------------------
        assert_eq!(scene.world.objects.len(), 1);

        Ok(())
    }

    #[test]
    fn test_simple_mesh_light_sources() -> Result<()> {
        // Build the test-file
        let dir = tempdir()?;
        let path = dir.path().join("parallelepiped.obj");
        let mut file = File::create(path.clone())?;

        let content = r#"# Parallelepiped Mesh
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 2.0 0.0
v 0.0 2.0 0.0
v 0.0 0.0 3.0
v 1.0 0.0 3.0
v 1.0 2.0 3.0
v 0.0 2.0 3.0

f 1 2 3
f 1 3 4
f 5 6 1
f 1 2 6
f 6 7 8
f 5 6 8
f 4 3 8
f 8 3 7
f 1 8 4
f 1 5 8
f 2 3 6
f 6 3 7
"#;

        file.write_all(content.as_bytes())?;

        // Actual parser test
        let text = format!(
            r#"
    material mesh_material(
        uniform(<0.0, 0.5, 0.9>),
        specular(),
        uniform(<0, 0, 0>)
    )

    simple_mesh(mesh_material, "{}", scaling([0.1, 0.2, 0.3]))
    "#,
            path.display()
        );

        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);

        let initial_vars = HashMap::new();
        let scene = parse_scene(&mut stream, initial_vars)?;

        assert_eq!(
            scene.materials.len(),
            1,
            "Expected 1 material but found {}",
            scene.materials.len()
        );

        assert_eq!(
            scene.world.objects.len(),
            1,
            "Expected 1 object but found {}",
            scene.world.objects.len()
        );

        Ok(())
    }

    #[test]
    fn test_point_light_source() -> Result<()> {
        let text = r#"point_light(point([0,0,10]),<0.0, 0.1, 0.5>)"#;
        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);

        let initial_vars = HashMap::new();
        let scene = parse_scene(&mut stream, initial_vars)?;

        assert_eq!(
            scene.world.light_sources.len(),
            1,
            "Expected 1 light source but found {}",
            scene.world.light_sources.len()
        );
        assert_eq!(
            scene.world.objects.len(),
            0,
            "Expected no objects but found {}",
            scene.world.objects.len()
        );

        let hit = HitRecord {
            world_point: Point::new(0.0, 0.0, 0.0),
            normal: Normal::from(Z_AXIS),
            uv: Vec2D::new(0.0, 0.0),
            t: 0.0,
            ray: Ray::new(Point::new(0.0, 0.0, -10.0), Z_AXIS),
            material: &Default::default(),
        };
        let mut pcg = PCG::default();
        let color = scene
            .world
            .light_sources
            .iter()
            .next()
            .unwrap()
            .source_contribution(&hit, &scene.world, &mut pcg)?;
        let expected_color = Color::new(0.0, 0.1, 0.5);
        assert!(
            color.is_close(&expected_color),
            "expected: {:?}\n found: {:?}",
            expected_color,
            color
        );
        Ok(())
    }

    #[test]
    fn test_spherical_light_source() -> Result<()> {
        let text = r#"spherical_light(
point([-10., 0.0, 0.0]),
1, <0.1, 0.2, 0.3>, 10)"#;
        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);

        let initial_vars = HashMap::new();
        let scene = parse_scene(&mut stream, initial_vars)?;

        assert_eq!(
            scene.world.light_sources.len(),
            1,
            "Expected 1 light source but found {}",
            scene.world.light_sources.len()
        );
        assert_eq!(
            scene.world.objects.len(),
            0,
            "Expected no objects but found {}",
            scene.world.objects.len()
        );

        let hit = HitRecord {
            world_point: Point::new(0.0, 0.0, 0.0),
            normal: Normal::from(-X_AXIS),
            uv: Vec2D::new(0.0, 0.0),
            t: 0.0,
            ray: Ray::new(Point::new(0.0, 0.0, 0.0), -X_AXIS),
            material: &Default::default(),
        };
        let mut pcg = PCG::default();

        // no objects in scene -> all samples unoccluded, result is deterministic
        let color = scene
            .world
            .light_sources
            .iter()
            .next()
            .unwrap()
            .source_contribution(&hit, &scene.world, &mut pcg)?;
        let expected_color = Color::new(0.1, 0.2, 0.3);
        assert!(
            (color.r - expected_color.r).abs() < 0.001,
            "expected: {:?}\n found: {:?}",
            expected_color.r,
            color.r
        );
        assert!(
            (color.g - expected_color.g).abs() < 0.001,
            "expected: {:?}\n found: {:?}",
            expected_color.g,
            color.g
        );
        assert!(
            (color.b - expected_color.b).abs() < 0.001,
            "expected: {:?}\n found: {:?}",
            expected_color.b,
            color.b
        );

        Ok(())
    }

    #[test]
    fn test_plane_procedural_flag() -> Result<()> {
        let cases: &[(&str, bool)] = &[
            (", true", true),
            (", True", true),
            (", false", false),
            (", False", false),
            ("", false),
        ];

        for (suffix, expected) in cases {
            let text = format!(
                r#"material floor_material(
checkered(black, White, 3),
diffuse(),
uniform(<0, 0, 0>)
)
plane(floor_material, identity{})"#,
                suffix
            );

            let cursor = std::io::Cursor::new(text);
            let mut stream = InputStream::new(cursor, 0, 4);
            let scene = parse_scene(&mut stream, HashMap::new())?;

            assert_eq!(scene.world.objects.len(), 1);

            // Hit at local point (-0.5, 0.5, 0.0)
            let ray_left = Ray::new(Point::new(-0.5, 0.5, 10.0), -Z_AXIS);
            let hit_left = scene.world.objects[0].ray_intersection(&ray_left).unwrap();

            // Hit at local point (0.5, 0.5, 0.0)
            let ray_right = Ray::new(Point::new(0.5, 0.5, 10.0), -Z_AXIS);
            let hit_right = scene.world.objects[0].ray_intersection(&ray_right).unwrap();

            if *expected {
                // procedural: uv = (u, v) raw
                assert!(
                    hit_left.uv.is_close(&Vec2D::new(-0.5, 0.5)),
                    "[{}] expected UV (-0.5, 0.5), got {:?}",
                    suffix,
                    hit_left.uv
                );
                assert!(
                    hit_right.uv.is_close(&Vec2D::new(0.5, 0.5)),
                    "[{}] expected UV (0.5, 0.5), got {:?}",
                    suffix,
                    hit_right.uv
                );
            } else {
                // tiled: uv = (frac(u), frac(v)) ∈ [0,1)
                assert!(
                    hit_left.uv.is_close(&Vec2D::new(0.5, 0.5)),
                    "[{}] expected UV (0.5, 0.5), got {:?}",
                    suffix,
                    hit_left.uv
                );
                assert!(
                    hit_right.uv.is_close(&Vec2D::new(0.5, 0.5)),
                    "[{}] expected UV (0.5, 0.5), got {:?}",
                    suffix,
                    hit_right.uv
                );
            }
        }

        Ok(())
    }

    #[test]
    fn test_parse_color_keywords() -> Result<()> {
        let cases = [
            ("uniform(black)", BLACK),
            ("uniform(BLACK)", BLACK),
            ("uniform(white)", WHITE),
            ("uniform(WHITE)", WHITE),
            ("uniform(<0.5, 0.5, 0.5>)", Color::new(0.5, 0.5, 0.5)),
        ];

        for (input, expected) in cases {
            let cursor = std::io::Cursor::new(input);
            let mut stream = InputStream::new(cursor, 0, 4);
            let scene = Scene::new();
            stream.read_token()?;
            stream.read_token()?;
            let color = parse_color(&mut stream, &scene)?;
            assert!(
                color.is_close(&expected),
                "input '{}': expected {:?}, got {:?}",
                input,
                expected,
                color
            );
        }
        Ok(())
    }
}
