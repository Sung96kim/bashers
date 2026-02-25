use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn run(
    dockerfile: Option<&Path>,
    tag: Option<&str>,
    no_cache: bool,
    context: Option<&Path>,
) -> Result<()> {
    let path = dockerfile.map(PathBuf::from).unwrap_or_else(|| {
        std::env::current_dir()
            .map(|cwd| cwd.join("Dockerfile"))
            .unwrap_or_else(|_| PathBuf::from("Dockerfile"))
    });
    let dockerfile_abs = path
        .canonicalize()
        .with_context(|| format!("Dockerfile path not found: {}", path.display()))?;
    eprintln!("Building: {}", dockerfile_abs.display());
    let context_path: PathBuf = context.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        dockerfile_abs
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    });
    let mut cmd = std::process::Command::new("docker");
    cmd.arg("build").arg("-f").arg(&dockerfile_abs);
    if let Some(t) = tag {
        cmd.arg("-t").arg(t);
    }
    if no_cache {
        cmd.arg("--no-cache");
    }
    cmd.arg(&context_path);
    let status = cmd.status().context("Failed to run docker build")?;
    if !status.success() {
        anyhow::bail!("docker build exited with {}", status);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_run_nonexistent_dockerfile_errors() {
        let result = run(
            Some(Path::new("/nonexistent/dockerfile")),
            None,
            false,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_run_nonexistent_with_tag_errors() {
        let result = run(
            Some(Path::new("/nonexistent/dockerfile")),
            Some("myimage:latest"),
            false,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_run_nonexistent_with_no_cache_errors() {
        let result = run(
            Some(Path::new("/nonexistent/dockerfile")),
            None,
            true,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_run_nonexistent_with_context_errors() {
        let result = run(
            Some(Path::new("/nonexistent/dockerfile")),
            None,
            false,
            Some(Path::new("/tmp")),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_run_nonexistent_with_all_options_errors() {
        let result = run(
            Some(Path::new("/nonexistent/dockerfile")),
            Some("myimage:v1"),
            true,
            Some(Path::new("/tmp")),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_error_message_includes_path() {
        let result = run(
            Some(Path::new("/nonexistent/my-docker-file")),
            None,
            false,
            None,
        );
        let err = result.unwrap_err().to_string();
        assert!(err.contains("my-docker-file"));
    }

    #[test]
    fn test_run_no_dockerfile_specified_defaults() {
        let result = run(None, None, false, None);
        assert!(result.is_err());
    }
}
