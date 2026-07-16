//! Helper functions for the `rstrace` command-line binary.
//!
//! These are CLI-specific concerns (parsing `--declare-float` entries,
//! normalizing output filenames) rather than core ray tracing logic, so
//! they live alongside the binary instead of in the library crate.

use anyhow::{Result, anyhow};
use rstrace::parser::ReflectancePolicy;
use std::collections::HashMap;

/// CLI-facing mirror of [`ReflectancePolicy`], kept separate so the
/// `rstrace` library doesn't depend on `clap`. Converts into the library
/// type via [`From`] before being passed to
/// [`parse_scene_with_policy`](rstrace::parser::parse_scene_with_policy).
#[derive(Copy, Clone, clap::ValueEnum, Default)]
pub enum CliReflectancePolicy {
    #[default]
    Reject,
    Rescale,
    Ignore,
}

impl From<CliReflectancePolicy> for ReflectancePolicy {
    fn from(p: CliReflectancePolicy) -> Self {
        match p {
            CliReflectancePolicy::Reject => Self::Reject,
            CliReflectancePolicy::Rescale => Self::Rescale,
            CliReflectancePolicy::Ignore => Self::Ignore,
        }
    }
}

/// Helper function to parse variables from the CLI into a HashMap
pub fn build_variable_table(declare_float: &[String]) -> HashMap<String, f32> {
    let mut variables = HashMap::new();
    for decl in declare_float {
        let parts: Vec<&str> = decl.split(':').collect();
        if parts.len() == 2 {
            if let Ok(val) = parts[1].parse::<f32>() {
                variables.insert(parts[0].to_string(), val);
            } else {
                eprintln!("Warning: Could not parse value for variable '{}'", parts[0]);
            }
        } else {
            eprintln!(
                "Warning: Invalid variable declaration format: '{}'. Use VAR:VALUE",
                decl
            );
        }
    }
    variables
}

/// Ensures a filename has the given extension, adding it only if missing.
/// E.g. "output" -> "output.pfm", "output.pfm" -> "output.pfm"
pub fn ensure_extension(name: &str, ext: &str) -> String {
    let suffix = format!(".{}", ext);
    if name.ends_with(&suffix) {
        name.to_string()
    } else {
        format!("{}.{}", name, ext)
    }
}

/// Raster image extensions recognized by `--format`.
const IMAGE_EXTENSIONS: [&str; 3] = ["png", "jpg", "jpeg"];

/// Ensures a raster image filename ends with the given `format` extension,
/// replacing any other recognized image extension already present instead
/// of appending on top of it.
/// E.g. ("output", "jpeg") -> "output.jpeg",
///      ("output.png", "jpeg") -> "output.jpeg" (not "output.png.jpeg"),
///      ("output.jpeg", "jpeg") -> "output.jpeg"
pub fn ensure_image_extension(name: &str, format: &str) -> String {
    let base = IMAGE_EXTENSIONS
        .iter()
        .find_map(|known_ext| {
            let suffix = format!(".{}", known_ext);
            if name.len() > suffix.len()
                && name[name.len() - suffix.len()..].eq_ignore_ascii_case(&suffix)
            {
                Some(&name[..name.len() - suffix.len()])
            } else {
                None
            }
        })
        .unwrap_or(name);
    format!("{}.{}", base, format)
}

/// Creates the parent directory of `path` (and any missing ancestors).
/// A bare filename with no parent is left untouched.
pub fn create_parent_dir(path: &str) -> Result<()> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| {
                anyhow!(
                    "Could not create output directory {}: {}",
                    parent.display(),
                    e
                )
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_image_extension_replaces_different_extension() {
        assert_eq!(
            ensure_image_extension("provaprova.png", "jpeg"),
            "provaprova.jpeg"
        );
    }

    #[test]
    fn test_ensure_image_extension_no_extension() {
        assert_eq!(ensure_image_extension("output", "jpeg"), "output.jpeg");
    }

    #[test]
    fn test_ensure_image_extension_same_extension() {
        assert_eq!(ensure_image_extension("output.jpeg", "jpeg"), "output.jpeg");
    }

    #[test]
    fn test_ensure_image_extension_case() {
        assert_eq!(ensure_image_extension("output.PNG", "jpg"), "output.jpg");
    }

    #[test]
    fn test_ensure_extension_no_extension() {
        assert_eq!(ensure_extension("output", "pfm"), "output.pfm");
    }

    #[test]
    fn test_ensure_extension_same_extension() {
        assert_eq!(ensure_extension("output.pfm", "pfm"), "output.pfm");
    }

    #[test]
    fn test_ensure_extension_different_extension() {
        assert_eq!(ensure_extension("output.pfm", "hdr"), "output.pfm.hdr");
    }

    #[test]
    fn test_build_variable_table() {
        let decls = vec!["clock:150".to_string(), "scale:2.5".to_string()];
        let variables = build_variable_table(&decls);

        assert_eq!(variables.len(), 2);
        assert_eq!(variables["clock"], 150.0);
        assert_eq!(variables["scale"], 2.5);
    }

    #[test]
    fn test_build_variable_table_bad_value() {
        let decls = vec!["clock:not_a_number".to_string()];
        let variables = build_variable_table(&decls);

        assert!(variables.is_empty());
    }

    #[test]
    fn test_build_variable_table_bad_format() {
        let decls = vec!["clock".to_string(), "clock:1:2".to_string()];
        let variables = build_variable_table(&decls);

        assert!(variables.is_empty());
    }

    #[test]
    fn test_build_variable_table_empty() {
        let variables = build_variable_table(&[]);

        assert!(variables.is_empty());
    }

    #[test]
    fn test_create_parent_dir_missing_ancestors() {
        let base_dir = std::env::temp_dir();
        let base = base_dir.join("dir_test");

        let _ = std::fs::remove_dir_all(&base);
        let target = base.join("nested/sub/render.pfm");

        create_parent_dir(target.to_str().unwrap()).unwrap();

        assert!(target.parent().unwrap().is_dir());
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn test_create_parent_dir_bare_filename() {
        create_parent_dir("output.pfm").unwrap();
        assert!(!std::path::Path::new("output.pfm").exists());
    }
}
