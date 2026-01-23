use anyhow::{Context, Result};
use std::path::Path;
use which::which;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    Uv,
    Poetry,
    Cargo,
}

pub fn detect() -> Result<Option<ProjectType>> {
    let detection_rules: Vec<(bool, &str, ProjectType)> = vec![
        (
            Path::new("Cargo.toml").exists(),
            "cargo",
            ProjectType::Cargo,
        ),
        (
            Path::new("uv.lock").exists() || has_project_section(),
            "uv",
            ProjectType::Uv,
        ),
        (
            Path::new("poetry.lock").exists() || has_poetry_section(),
            "poetry",
            ProjectType::Poetry,
        ),
    ];

    for (condition, tool, project_type) in detection_rules {
        if condition {
            which(tool).with_context(|| format!("{} not found on PATH", tool))?;
            return Ok(Some(project_type));
        }
    }

    Ok(None)
}

fn has_project_section() -> bool {
    if Path::new("pyproject.toml")
        .exists() { {
            std::fs::read_to_string("pyproject.toml")
                .map(|content| content.contains("[project]"))
                .unwrap_or(false)
        } } else { false }
}

fn has_poetry_section() -> bool {
    if Path::new("pyproject.toml")
        .exists() { {
            std::fs::read_to_string("pyproject.toml")
                .map(|content| content.contains("[tool.poetry]"))
                .unwrap_or(false)
        } } else { false }
}

impl ProjectType {
    pub fn is_uv(&self) -> bool {
        matches!(self, ProjectType::Uv)
    }

    pub fn is_poetry(&self) -> bool {
        matches!(self, ProjectType::Poetry)
    }

    pub fn is_cargo(&self) -> bool {
        matches!(self, ProjectType::Cargo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_project_type_methods() {
        assert!(ProjectType::Uv.is_uv());
        assert!(!ProjectType::Uv.is_poetry());
        assert!(!ProjectType::Uv.is_cargo());

        assert!(ProjectType::Poetry.is_poetry());
        assert!(!ProjectType::Poetry.is_uv());
        assert!(!ProjectType::Poetry.is_cargo());

        assert!(ProjectType::Cargo.is_cargo());
        assert!(!ProjectType::Cargo.is_uv());
        assert!(!ProjectType::Cargo.is_poetry());
    }

    #[test]
    fn test_has_project_section() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_dir_name = format!("test_project_detection_{}", timestamp);
        let test_dir = Path::new(&test_dir_name);
        
        // Clean up any existing test directory
        if test_dir.exists() {
            fs::remove_dir_all(test_dir).ok();
        }
        fs::create_dir_all(test_dir).unwrap();

        // Test with [project] section
        let pyproject_content = "[project]\nname = \"test\"\nversion = \"0.1.0\"";
        fs::write(test_dir.join("pyproject.toml"), pyproject_content).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        
        // Ensure we can change to the test directory
        assert!(std::env::set_current_dir(test_dir).is_ok(), "Failed to change to test directory");

        let result = has_project_section();
        assert!(result, "has_project_section() should return true when [project] section exists");

        // Always restore the original directory, even if assertion fails
        let restore_result = std::env::set_current_dir(&original_dir);
        assert!(restore_result.is_ok(), "Failed to restore original directory");
        
        // Clean up
        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_has_project_section_no_file() {
        let test_dir = Path::new("test_no_pyproject");
        if test_dir.exists() {
            fs::remove_dir_all(test_dir).ok();
        }
        fs::create_dir_all(test_dir).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(test_dir).unwrap();

        let result = has_project_section();
        assert!(!result);

        std::env::set_current_dir(original_dir).unwrap();
        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_has_project_section_no_project_section() {
        let test_dir = Path::new("test_no_project_section");
        if test_dir.exists() {
            fs::remove_dir_all(test_dir).ok();
        }
        fs::create_dir_all(test_dir).unwrap();

        let pyproject_content = "[tool.poetry]\nname = \"test\"";
        fs::write(test_dir.join("pyproject.toml"), pyproject_content).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(test_dir).unwrap();

        let result = has_project_section();
        assert!(!result);

        std::env::set_current_dir(original_dir).unwrap();
        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_has_poetry_section() {
        let test_dir = Path::new("test_poetry_detection");
        if test_dir.exists() {
            fs::remove_dir_all(test_dir).ok();
        }
        fs::create_dir_all(test_dir).unwrap();

        let pyproject_content = "[tool.poetry]\nname = \"test\"\nversion = \"0.1.0\"";
        fs::write(test_dir.join("pyproject.toml"), pyproject_content).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(test_dir).unwrap();

        let result = has_poetry_section();
        assert!(result);

        std::env::set_current_dir(original_dir).unwrap();
        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_has_poetry_section_no_file() {
        let test_dir = Path::new("test_no_poetry_file");
        if test_dir.exists() {
            fs::remove_dir_all(test_dir).ok();
        }
        fs::create_dir_all(test_dir).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(test_dir).unwrap();

        let result = has_poetry_section();
        assert!(!result);

        std::env::set_current_dir(original_dir).unwrap();
        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_has_poetry_section_no_poetry_section() {
        let test_dir = Path::new("test_no_poetry_section");
        if test_dir.exists() {
            fs::remove_dir_all(test_dir).ok();
        }
        fs::create_dir_all(test_dir).unwrap();

        let pyproject_content = "[project]\nname = \"test\"";
        fs::write(test_dir.join("pyproject.toml"), pyproject_content).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(test_dir).unwrap();

        let result = has_poetry_section();
        assert!(!result);

        std::env::set_current_dir(original_dir).unwrap();
        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_project_type_equality() {
        assert_eq!(ProjectType::Uv, ProjectType::Uv);
        assert_eq!(ProjectType::Poetry, ProjectType::Poetry);
        assert_eq!(ProjectType::Cargo, ProjectType::Cargo);
        assert_ne!(ProjectType::Uv, ProjectType::Poetry);
        assert_ne!(ProjectType::Uv, ProjectType::Cargo);
        assert_ne!(ProjectType::Poetry, ProjectType::Cargo);
    }

    #[test]
    fn test_project_type_debug() {
        // Test that Debug trait works
        let uv = ProjectType::Uv;
        let debug_str = format!("{:?}", uv);
        assert!(debug_str.contains("Uv"));
    }

    #[test]
    fn test_project_type_clone() {
        let original = ProjectType::Cargo;
        let cloned = original;
        assert_eq!(original, cloned);
    }
}
