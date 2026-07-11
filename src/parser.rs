// This file is licensed under the EUPL-1.2. See LICENSE.md.

//! Parser for the raytracer scene description language.
//!
//! Consumes the [`Token`](crate::lexer::Token)s produced by
//! [`InputStream`](crate::lexer::InputStream) and builds a [`Scene`]: a fully
//! resolved description of the 3D world (shapes, materials, light sources) and
//! the camera, ready to be handed to a [`Renderer`](crate::renderer::Renderer).
//!
//! # Grammar overview
//!
//! A scene file is a sequence of zero or more top-level statements, read until
//! [`TokenKind::StopToken`](crate::lexer::TokenKind::StopToken):
//!
//! ```text
//! scene           ::= statement*
//! statement       ::= float_decl | material_decl | shape | light | camera
//!
//! float_decl      ::= "float" IDENTIFIER "(" number ")"
//! material_decl   ::= "material" IDENTIFIER "(" pigment "," brdf "," pigment ")"
//!
//! shape           ::= sphere | plane | box | simple_mesh
//! sphere          ::= "sphere" "(" IDENTIFIER "," transformation ")"
//! plane           ::= "plane" "(" IDENTIFIER "," transformation ["," bool] ")"
//! box             ::= "box" "(" IDENTIFIER "," "point" "(" vector ")" "," "point" "(" vector ")" ")"
//! simple_mesh     ::= "simple_mesh" "(" IDENTIFIER "," STRING "," transformation ")"
//!
//! light           ::= point_light | spherical_light
//! point_light     ::= "point_light" "(" "point" "(" vector ")" "," color ")"
//! spherical_light ::= "spherical_light" "(" "point" "(" vector ")" "," number "," color "," number ")"
//!
//! camera          ::= "camera" "(" ("perspective" | "orthogonal") "," transformation "," number ")"
//!
//! transformation  ::= transform_atom ("*" transform_atom)*
//! transform_atom  ::= "identity" | "translation" "(" vector ")"
//!                   | "rotation_x" "(" number ")" | "rotation_y" "(" number ")" | "rotation_z" "(" number ")"
//!                   | "scaling" "(" (vector | number) ")"
//!
//! pigment ::= "uniform" "(" color ")"
//!           | "checkered" "(" color "," color "," number ")"
//!           | "image" "(" STRING ")"
//!           | "image" "(" STRING "," number "," number "," number ")"
//!           | "gradient" "(" color "," color "," number ")"
//! brdf    ::= "diffuse" "(" ")" | "specular" "(" ")"
//!
//! color   ::= "<" number "," number "," number ">" | "black" | "white"
//! vector  ::= "[" number "," number "," number "]"
//! number  ::= LITERAL_NUMBER | IDENTIFIER   (identifiers are resolved via `Scene::float_variables`)
//! ```
//!
//! # Variables
//!
//! Anywhere a `number` is expected, an identifier can be used instead of a
//! literal: it's looked up in [`Scene::float_variables`] (see [`expect_number`]).
//! Variables can be declared inside the file with `float NAME(VALUE)`, or
//! pre-populated from the command line via the `initial_variables` argument of
//! [`parse_scene`] (e.g. `--declare-float clock:150`). Names present in
//! [`Scene::overridden_variables`] are protected: a later `float` statement in
//! the file will *not* overwrite a value supplied on the command line.
//!
//! # Error handling
//!
//! All parsing functions return [`anyhow::Result`]; grammar errors are reported
//! with the offending token's [`SourceLocation`](crate::lexer::SourceLocation)
//! (line and column) to make debugging scene files easier.
//!
//! # Example
//!
//! ```text
//! float clock(150)
//!
//! material sky_material(
//!     uniform(<0, 0, 0>),
//!     diffuse(),
//!     uniform(<0.7, 0.5, 1>)
//! )
//!
//! plane(sky_material, translation([0, 0, 100]) * rotation_y(clock))
//!
//! camera(perspective, identity, 1.0)
//! ```

use crate::brdf::{BRDF, DiffusiveBrdf, SpecularBrdf};
use crate::camera::{Camera, OrthogonalCamera, PerspectiveCamera};
use crate::color::{BLACK, Color, WHITE};
use crate::geometry::{Point, Vector};
use crate::hdr_image::HDR;
use crate::lexer::{InputStream, Keyword, TokenKind};
use crate::light_source::{PointLightSource, SphericalLightSource};
use crate::materials::{ClampPigment, Material};
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

/// A scene parsed from a scene-description file (or a fragment thereof).
///
/// [`Scene`] plays two roles:
/// - **Symbol table**: while [`parse_scene`] consumes the token stream from
///   top to bottom, it accumulates named materials and variables here, so
///   that later declarations (e.g. a `sphere` referencing a `material`
///   defined earlier) can resolve them by name.
/// - **Output**: once the [`StopToken`](crate::lexer::TokenKind::StopToken)
///   is reached, the same struct holds the complete [`World`] and [`Camera`],
///   ready to be handed to the renderer.
///
/// Statements are processed strictly in file order: a `sphere`/`plane` must
/// appear *after* the `material` it references, otherwise parsing fails with
/// an "Unknown material" error.
///
/// # Field lifecycle
///
/// | Field | Populated by | Consumed by |
/// |---|---|---|
/// | `materials` | `material` declarations | `sphere`, `plane`, `box`, `simple_mesh` |
/// | `float_variables` | `float` declarations, or `initial_variables` passed to [`parse_scene`] | [`expect_number`], anywhere an identifier stands in for a number |
/// | `overridden_variables` | names present in the `initial_variables` passed to [`parse_scene`] | the `float` branch, to avoid overwriting a CLI-supplied value |
/// | `world` | `sphere`, `plane`, `box`, `simple_mesh`, `point_light`, `spherical_light` | the renderer |
/// | `camera` | the (at most one) `camera` declaration | the caller of `parse_scene`, to build the `ImageTracer` |
///
/// # Example
///
/// ```no_run
/// # use std::collections::HashMap;
/// # use std::io::Cursor;
/// # use rstrace::lexer::InputStream;
/// # use rstrace::parser::parse_scene;
/// let mut stream = InputStream::new(Cursor::new("camera(perspective, identity, 1.0)"), 0, 4);
/// let scene = parse_scene(&mut stream, HashMap::new())?;
/// assert!(scene.camera.is_some());
/// # Ok::<(), anyhow::Error>(())
/// ```
pub struct Scene {
    /// Symbol table of materials, indexed by name (`material NAME(...)`).
    pub materials: HashMap<String, Material>,

    /// Shapes and light sources accumulated so far, in declaration order.
    pub world: World,

    /// The observer, once a `camera(...)` statement has been encountered.
    /// Remains `None` until then; the caller decides how to handle a missing
    /// camera at the end of parsing.
    pub camera: Option<Box<dyn Camera>>,

    /// Symbol table of user-defined float variables.
    /// Written by `float NAME(VALUE)` declarations, read by
    /// [`expect_number`] whenever an identifier appears in place of a number.
    pub float_variables: HashMap<String, f32>,

    /// Names of the variables supplied via `initial_variables` in
    /// [`parse_scene`] (typically through the `--declare-float` CLI flag),
    /// which must therefore not be overwritten by a later `float`
    /// declaration in the file.
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

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum ReflectancePolicy {
    #[default]
    Reject,
    Rescale,
    Ignore,
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

/// Parses a color.
///
/// Supported formats are:
/// - `<r, g, b>`
/// - `black`
/// - `white`
pub fn parse_color<B: BufRead>(stream: &mut InputStream<B>, scene: &Scene) -> Result<Color> {
    let token = stream.read_token()?;
    match token.kind {
        TokenKind::Keyword(Keyword::White) => Ok(WHITE),
        TokenKind::Keyword(Keyword::Black) => Ok(BLACK),
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

/// Parses a point in the format `point([x, y, z])`.
pub fn parse_point<B: BufRead>(stream: &mut InputStream<B>, scene: &Scene) -> Result<Point> {
    expect_symbol(stream, '(')?;
    let v = parse_vector(stream, scene)?;
    expect_symbol(stream, ')')?;
    Ok(Point::new(v.x, v.y, v.z))
}

/// Parses a pigment.
///
/// Supported pigment types are:
/// - uniform
/// - checkered
/// - image: `image(STRING)` loads a `.pfm` texture as-is; `image(STRING, factor_a,
///   avr_lum, gamma)` loads a `.png`/`.jpg`/`.jpeg` texture and reconstructs an
///   approximate HDR image from it via [`HDR::load_from_ldr`]. The file extension
///   determines which form is required.
/// - gradient
pub fn parse_pigment<B: BufRead>(
    stream: &mut InputStream<B>,
    scene: &Scene,
) -> Result<Box<dyn Pigment>> {
    let keyword = expect_keywords(
        stream,
        &[
            Keyword::Uniform,
            Keyword::Checkered,
            Keyword::Image,
            Keyword::Gradient,
        ],
    )?;
    expect_symbol(stream, '(')?;

    let result: Box<dyn Pigment> = match keyword {
        Keyword::Uniform => {
            let color = parse_color(stream, scene)?;
            Box::new(UniformPigment::new(color))
        }
        Keyword::Checkered => {
            let color1 = parse_color(stream, scene)?;
            expect_symbol(stream, ',')?;
            let color2 = parse_color(stream, scene)?;
            expect_symbol(stream, ',')?;
            let steps = expect_number(stream, scene)? as u32;
            Box::new(CheckeredPigment::new(color1, color2, steps))
        }
        Keyword::Image => {
            let file_name = expect_string(stream)?;
            let extension = PathBuf::from(&file_name)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            // Lookahead: a bare `image("path.pfm")` has no extra arguments, while
            // an LDR source needs the tone-mapping parameters used to invert it
            // back to HDR: `image("path.png", factor_a, avr_lum, gamma)`.
            let next_token = stream.read_token()?;
            let is_ldr = matches!(next_token.kind, TokenKind::Symbol(','));
            stream.unread_token(next_token)?;

            let image = match extension.as_deref() {
                Some("png") | Some("jpg") | Some("jpeg") => {
                    if !is_ldr {
                        bail!(
                            "Grammar Error: LDR image '{}' requires factor_a, avr_lum, and gamma, \
                             e.g. image(\"{}\", 0.18, 1.0, 2.2)",
                            file_name,
                            file_name
                        );
                    }
                    expect_symbol(stream, ',')?;
                    let factor_a = expect_number(stream, scene)?;
                    expect_symbol(stream, ',')?;
                    let avr_lum = expect_number(stream, scene)?;
                    expect_symbol(stream, ',')?;
                    let gamma = expect_number(stream, scene)?;
                    HDR::load_from_ldr(&file_name, factor_a, avr_lum, gamma)?
                }
                Some("pfm") => {
                    if is_ldr {
                        bail!(
                            "Grammar Error: factor_a, avr_lum, and gamma are only valid for \
                             LDR images (.png, .jpg, .jpeg), found '{}'",
                            file_name
                        );
                    }
                    read_pfm_file(&file_name)?
                }
                _ => {
                    bail!(
                        "Image reader accepts only .png, .jpg, .jpeg or .pfm. Filename: '{}'",
                        file_name
                    );
                }
            };
            Box::new(ImagePigment::new(image))
        }
        Keyword::Gradient => {
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
    let keyword = expect_keywords(stream, &[Keyword::Diffuse, Keyword::Specular])?;
    expect_symbol(stream, '(')?;

    let result: Box<dyn BRDF> = match keyword {
        Keyword::Diffuse => Box::new(DiffusiveBrdf {}),
        Keyword::Specular => Box::new(SpecularBrdf {}),
        _ => unreachable!(),
    };

    expect_symbol(stream, ')')?;
    Ok(result)
}

/// Parses a material definition.
///
/// Expected syntax:
/// `material_name(base_pigment, brdf, emitted_radiance)`
pub fn parse_material<B: BufRead>(
    stream: &mut InputStream<B>,
    scene: &Scene,
    policy: ReflectancePolicy,
) -> Result<(String, Material)> {
    let material_loc = stream.source_location;
    let name = expect_identifier(stream)?;
    expect_symbol(stream, '(')?;

    let base_pigment = parse_pigment(stream, scene)?;
    let base_pigment: Box<dyn Pigment> = match policy {
        ReflectancePolicy::Reject => {
            base_pigment.validate_reflectance().map_err(|e| {
                anyhow!(
            "Invalid material '{name}' at {}:{}: {e} (reflectance channels must lie in [0,1])",
            material_loc.line_number,
            material_loc.col_number
        )
            })?;
            base_pigment
        }
        ReflectancePolicy::Rescale => {
            if base_pigment.validate_reflectance().is_err() {
                eprintln!(
                    "Warning: material '{name}' at {}:{}: pigment reflectance outside [0,1], colors rescaled",
                    material_loc.line_number, material_loc.col_number
                );
                Box::new(ClampPigment::new(base_pigment))
            } else {
                base_pigment
            }
        }
        ReflectancePolicy::Ignore => base_pigment,
    };

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

/// Parses one or more transformations chained by `*`.
///
/// Supported transformations include identity, translation,
/// rotations about the principal axes, and uniform or non-uniform
/// scaling.
pub fn parse_transformation<B: BufRead>(
    stream: &mut InputStream<B>,
    scene: &Scene,
) -> Result<Transformation> {
    let mut result = Transformation::new(crate::functions::IDENTITY_4X4);

    loop {
        let keyword = expect_keywords(
            stream,
            &[
                Keyword::Identity,
                Keyword::Translation,
                Keyword::RotationX,
                Keyword::RotationY,
                Keyword::RotationZ,
                Keyword::Scaling,
            ],
        )?;

        match keyword {
            Keyword::Identity => {}
            Keyword::Translation => {
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
            Keyword::Scaling => {
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

/// Parses a plane.
///
/// Expected syntax:
/// `plane(material_name, transformation[, procedural_texture])`
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
            let key = expect_keywords(stream, &[Keyword::True, Keyword::False])?;
            expect_symbol(stream, ')')?;
            key == Keyword::True
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

/// Parses an axis-aligned bounding box.
///
/// Expected syntax:
/// `box(material_name, point(min), point(max))`
pub fn parse_box<B: BufRead>(stream: &mut InputStream<B>, scene: &Scene) -> Result<AABB> {
    expect_symbol(stream, '(')?;
    let material_name = expect_identifier(stream)?;
    expect_symbol(stream, ',')?;
    expect_keywords(stream, &[Keyword::Point])?;
    let point1 = parse_point(stream, scene)?;
    expect_symbol(stream, ',')?;
    expect_keywords(stream, &[Keyword::Point])?;
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

/// Parses a simple mesh loaded from an OBJ file.
///
/// Expected syntax:
/// `simple_mesh(material_name, "path/to/file.obj", transformation)`
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

/// Parses a point light source.
///
/// Expected syntax:
/// `point_light(point(position), color)`
pub fn parse_point_light<B: BufRead>(
    stream: &mut InputStream<B>,
    scene: &Scene,
) -> Result<PointLightSource> {
    expect_symbol(stream, '(')?;
    expect_keywords(stream, &[Keyword::Point])?;
    let point = parse_point(stream, scene)?;
    expect_symbol(stream, ',')?;
    let color = parse_color(stream, scene)?;
    expect_symbol(stream, ')')?;
    Ok(PointLightSource::new(point, color))
}

/// Parses a spherical area light source.
///
/// Expected syntax:
/// `spherical_light(point(position), radius, color, samples)`
pub fn parse_spherical_light<B: BufRead>(
    stream: &mut InputStream<B>,
    scene: &Scene,
) -> Result<SphericalLightSource> {
    expect_symbol(stream, '(')?;
    expect_keywords(stream, &[Keyword::Point])?;
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

/// Parses a camera.
///
/// Expected syntax:
/// `camera(camera_type, transformation, distance)`
pub fn parse_camera<B: BufRead>(
    stream: &mut InputStream<B>,
    scene: &Scene,
) -> Result<Box<dyn Camera>> {
    expect_symbol(stream, '(')?;
    let cam_type = expect_keywords(stream, &[Keyword::Perspective, Keyword::Orthogonal])?;
    expect_symbol(stream, ',')?;
    let transformation = parse_transformation(stream, scene)?;
    expect_symbol(stream, ',')?;
    let distance = expect_number(stream, scene)?;
    if distance < 0.0 {
        bail!(
            "Invalid distance: {}.\nDistance cannot be negative",
            distance
        );
    }
    expect_symbol(stream, ')')?;

    let camera: Box<dyn Camera> = match cam_type {
        Keyword::Perspective => {
            let mut cam = PerspectiveCamera::new(transformation);
            cam.set_distance(distance);
            Box::new(cam)
        }
        Keyword::Orthogonal => {
            let cam = OrthogonalCamera::new(transformation);
            Box::new(cam)
        }
        _ => unreachable!(),
    };

    Ok(camera)
}

/// Parses an entire scene file until the end-of-file token is reached.
///
/// The parser recognizes variable declarations, material definitions,
/// geometric primitives, meshes, light sources, and the camera, building
/// a complete [`Scene`] as it consumes the input stream.
pub fn parse_scene<B: BufRead>(
    stream: &mut InputStream<B>,
    initial_variables: HashMap<String, f32>,
) -> Result<Scene> {
    parse_scene_with_policy(stream, initial_variables, ReflectancePolicy::Reject)
}

pub fn parse_scene_with_policy<B: BufRead>(
    stream: &mut InputStream<B>,
    initial_variables: HashMap<String, f32>,
    policy: ReflectancePolicy,
) -> Result<Scene> {
    let mut scene = Scene::new();
    scene.overridden_variables = initial_variables.keys().cloned().collect();
    scene.float_variables = initial_variables;

    loop {
        let token = stream.read_token()?;

        match token.kind {
            TokenKind::Keyword(Keyword::Float) => {
                let name = expect_identifier(stream)?;
                expect_symbol(stream, '(')?;
                let value = expect_number(stream, &scene)?;
                expect_symbol(stream, ')')?;

                // Only override if it wasn't provided via command line arguments
                if !scene.overridden_variables.contains(&name) {
                    scene.float_variables.insert(name, value);
                }
            }
            TokenKind::Keyword(Keyword::Material) => {
                let (name, material) = parse_material(stream, &scene, policy)?;
                scene.materials.insert(name, material);
            }
            TokenKind::Keyword(Keyword::Sphere) => {
                let sphere = parse_sphere(stream, &scene)?;
                scene.world.objects.push(Box::new(sphere));
            }
            TokenKind::Keyword(Keyword::Plane) => {
                let plane = parse_plane(stream, &scene)?;
                scene.world.objects.push(Box::new(plane));
            }
            TokenKind::Keyword(Keyword::Box) => {
                let box_shape = parse_box(stream, &scene)?;
                scene.world.objects.push(Box::new(box_shape));
            }
            TokenKind::Keyword(Keyword::SimpleMesh) => {
                let simple_mesh = parse_simple_mesh(stream, &scene)?;
                scene.world.objects.push(Box::new(simple_mesh));
            }
            TokenKind::Keyword(Keyword::PtLightSource) => {
                let point_light_source: PointLightSource = parse_point_light(stream, &scene)?;
                scene.world.light_sources.push(Box::new(point_light_source));
            }
            TokenKind::Keyword(Keyword::SphLightSource) => {
                let spherical_light_source: SphericalLightSource =
                    parse_spherical_light(stream, &scene)?;
                scene
                    .world
                    .light_sources
                    .push(Box::new(spherical_light_source));
            }
            TokenKind::Keyword(Keyword::Camera) => {
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
    use crate::pfm_func::Endianness;
    use crate::ray::Ray;
    use image::{Rgb, RgbImage};
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
    #[should_panic(
        expected = "Invalid material 'ground_material' at 1:9: UniformPigment has invalid reflection:"
    )]
    fn test_parse_material_fail() {
        let text = r#"material ground_material(
            uniform(<0.0, 5.1, 0.1>),
            diffuse(),
            uniform(<0, 0, 0>)
        )"#;

        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let initial_vars = HashMap::new();
        let _ = parse_scene(&mut stream, initial_vars).unwrap();
    }

    /// Creates a tiny PFM fixture in `dir` with a safe, always-valid
    /// reflectance color, and returns its path.
    fn valid_pfm(dir: &tempfile::TempDir) -> std::path::PathBuf {
        let path = dir.path().join("fixture.pfm");
        let mut hdr = HDR::new(1, 1);
        hdr.set_pixel(0, 0, Color::new(0.2, 0.3, 0.4)).unwrap();
        let file = File::create(&path).unwrap();
        hdr.write_pfm(file, &Endianness::LittleEndian).unwrap();
        path
    }

    /// Creates a tiny PNG fixture in `dir`, dim enough that `load_from_ldr`'s
    /// inverse tone-mapping stays within [0,1], and returns its path.
    fn valid_png(dir: &tempfile::TempDir) -> std::path::PathBuf {
        let path = dir.path().join("fixture.png");
        let mut image = RgbImage::new(1, 1);
        image.put_pixel(0, 0, Rgb([40, 40, 40]));
        image.save(&path).unwrap();
        path
    }

    #[test]
    fn test_parse_pfm_image_pigment() -> Result<()> {
        let dir = tempdir()?;
        let path = valid_pfm(&dir);

        let text = format!(
            r#"
        material ball(
            image("{}"),
            diffuse(),
            uniform(<0, 0, 0>)
        )
        "#,
            path.display()
        );

        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let scene = parse_scene(&mut stream, HashMap::new())?;

        assert!(scene.materials.contains_key("ball"));
        Ok(())
    }

    #[test]
    #[should_panic(expected = "ImagePigment has invalid reflection:")]
    fn test_parse_image_pigment_fail() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("fixture.pfm");

        let mut hdr = HDR::new(2, 2);
        hdr.set_pixel(0, 0, Color::new(10.0, 0.0, 0.0)).unwrap();
        hdr.set_pixel(1, 0, Color::new(0.0, 1.0, 0.0)).unwrap();
        hdr.set_pixel(0, 1, Color::new(0.0, 0.0, 1.0)).unwrap();
        hdr.set_pixel(1, 1, Color::new(1.0, 1.0, 1.0)).unwrap();
        let file = File::create(&path).unwrap();
        hdr.write_pfm(file, &Endianness::LittleEndian).unwrap();

        let text = format!(
            r#"
        material ball(
            image("{}"),
            diffuse(),
            uniform(<0, 0, 0>)
        )
        "#,
            path.display()
        );

        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let _ = parse_scene(&mut stream, HashMap::new()).unwrap();
    }

    #[test]
    fn test_parse_ldr_image_pigment() -> Result<()> {
        let dir = tempdir()?;
        let path = valid_png(&dir);

        let text = format!(
            r#"
        material ball(
            image("{}", 0.18, 1.0, 2.2),
            diffuse(),
            uniform(<0, 0, 0>)
        )
        "#,
            path.display()
        );

        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let scene = parse_scene(&mut stream, HashMap::new())?;

        assert!(scene.materials.contains_key("ball"));
        Ok(())
    }

    #[test]
    #[should_panic(expected = "requires factor_a, avr_lum, and gamma")]
    fn test_parse_ldr_image_pigment_missing_parameters() {
        let dir = tempdir().unwrap();
        let path = valid_png(&dir);
        let text = format!(
            r#"
        material ball(
            image("{}"),
            diffuse(),
            uniform(<0, 0, 0>)
        )
        "#,
            path.display()
        );

        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        parse_scene(&mut stream, HashMap::new()).unwrap();
    }

    #[test]
    #[should_panic(expected = "only valid for LDR images")]
    fn test_parse_pfm_image_pigment_extra_params_fails() {
        let dir = tempdir().unwrap();
        let path = valid_pfm(&dir);
        let text = format!(
            r#"
        material ball(
            image("{}", 0.18, 1.0, 2.2),
            diffuse(),
            uniform(<0, 0, 0>)
        )
        "#,
            path.display()
        );

        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        parse_scene(&mut stream, HashMap::new()).unwrap();
    }

    #[test]
    #[should_panic(expected = "expected symbol ')'")]
    fn test_parse_ldr_image_pigment_fail() {
        let dir = tempdir().unwrap();
        let path = valid_png(&dir);
        let text = format!(
            r#"
        material ball(
            image("{}", 0.18, 1.0, 2.2 extra),
            diffuse(),
            uniform(<0, 0, 0>)
        )
        "#,
            path.display()
        );

        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        parse_scene(&mut stream, HashMap::new()).unwrap();
    }

    #[test]
    #[should_panic(expected = "expected symbol ')'")]
    fn test_parse_ldr_image_pigment_missing_parenthesis() {
        let dir = tempdir().unwrap();
        let path = valid_png(&dir);
        let text = format!(
            r#"
        material ball(
            image("{}", 0.18, 1.0, 2.2
            diffuse(),
            uniform(<0, 0, 0>)
        )
        "#,
            path.display()
        );

        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        parse_scene(&mut stream, HashMap::new()).unwrap();
    }

    #[test]
    #[should_panic(expected = "expected symbol ')'")]
    fn test_parse_pfm_image_pigment_fail2() {
        let dir = tempdir().unwrap();
        let path = valid_pfm(&dir);
        let text = format!(
            r#"
        material ball(
            image("{}" garbage),
            diffuse(),
            uniform(<0, 0, 0>)
        )
        "#,
            path.display()
        );

        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        parse_scene(&mut stream, HashMap::new()).unwrap();
    }

    #[test]
    fn test_parse_image_pigment_unknown_extension() {
        let text = r#"
        material ball(
            image("tests/assets/texture.bmp"),
            diffuse(),
            uniform(<0, 0, 0>)
        )
        "#;

        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        assert!(parse_scene(&mut stream, HashMap::new()).is_err());
    }

    #[test]
    fn test_parse_scene_with_policy_no_rescale() -> Result<()> {
        let text = r#"material ball(
        uniform(<0.1, 0.2, 0.3>), diffuse(), uniform(black))"#;
        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let scene =
            parse_scene_with_policy(&mut stream, HashMap::new(), ReflectancePolicy::Rescale)?;
        let color = scene.materials["ball"]
            .pigment
            .get_color(&Vec2D::new(0.0, 0.0))?;
        assert!(
            color.is_close(&Color::new(0.1, 0.2, 0.3)),
            "expected pigment to be left untouched, got {:?}",
            color
        );
        Ok(())
    }

    #[test]
    fn test_parse_scene_with_policy_rescale() -> Result<()> {
        let text = r#"material ball(
        uniform(<1, 2, 0.3>), diffuse(), uniform(black))"#;
        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let scene =
            parse_scene_with_policy(&mut stream, HashMap::new(), ReflectancePolicy::Rescale)?;
        let color = scene.materials["ball"]
            .pigment
            .get_color(&Vec2D::new(0.0, 0.0))?;
        assert!(
            color.is_close(&Color::new(0.5, 1.0, 0.15)),
            "expected pigment to be half the original, got {:?}",
            color
        );
        Ok(())
    }

    #[test]
    fn test_parse_scene_with_policy_ignore() -> Result<()> {
        let text = r#"material ball(
        uniform(<1, 2, 0.3>), diffuse(), uniform(black))"#;
        let cursor = std::io::Cursor::new(text);
        let mut stream = InputStream::new(cursor, 0, 4);
        let scene =
            parse_scene_with_policy(&mut stream, HashMap::new(), ReflectancePolicy::Ignore)?;
        let color = scene.materials["ball"]
            .pigment
            .get_color(&Vec2D::new(0.0, 0.0))?;
        assert!(
            color.is_close(&Color::new(1.0, 2.0, 0.3)),
            "expected pigment to be left untouched, got {:?}",
            color
        );
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
