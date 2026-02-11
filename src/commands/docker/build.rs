use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn run(
    dockerfile: &Path,
    tag: Option<&str>,
    no_cache: bool,
    context: Option<&Path>,
) -> Result<()> {
    let dockerfile_abs = dockerfile
        .canonicalize()
        .with_context(|| format!("Dockerfile path not found: {}", dockerfile.display()))?;
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
