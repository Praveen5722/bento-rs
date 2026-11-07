use anyhow::{Context, Result};
use std::{fs, path::{Path, PathBuf}};
const CGROUP_ROOT: &str = "/sys/fs/cgroup";

pub struct Cgroup { pub path: PathBuf }
impl Cgroup {
    pub fn create(id: &str, spec: &crate::config::Spec) -> Result<Self> {
        validate_id(id)?;
        let path = Path::new(CGROUP_ROOT).join("bento").join(id);
        fs::create_dir_all(&path).with_context(|| format!("cannot create cgroup {}", path.display()))?;
        if let Some(r) = spec.linux.as_ref().and_then(|l| l.resources.as_ref()) {
            if let Some(n) = r.memory.as_ref().and_then(|m| m.limit) {
                if n <= 0 { anyhow::bail!("memory limit must be positive"); }
                fs::write(path.join("memory.max"), n.to_string()).context("write memory.max")?;
            }
            if let Some(n) = r.pids.as_ref().and_then(|p| p.limit) {
                if n < 0 { anyhow::bail!("pids limit must be non-negative"); }
                fs::write(path.join("pids.max"), n.to_string()).context("write pids.max")?;
            }
        }
        Ok(Self { path })
    }
    pub fn attach_pid(&self, pid: u32) -> Result<()> {
        fs::write(self.path.join("cgroup.procs"), pid.to_string()).context("attach process")
    }
    pub fn remove(&self) -> Result<()> {
        if self.path.exists() { fs::remove_dir(&self.path).context("remove cgroup")?; }
        Ok(())
    }
}
pub fn validate_id(id: &str) -> Result<()> {
    if id.is_empty() || id.len() > 64 || !id.chars().all(|c| c.is_ascii_alphanumeric() || c=='-' || c=='_') {
        anyhow::bail!("container ID must be 1-64 characters: letters, digits, '-' or '_'");
    }
    Ok(())
}
