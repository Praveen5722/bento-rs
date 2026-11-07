use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spec {
    pub oci_version: String,
    pub hostname: Option<String>,
    pub root: Root,
    pub process: Process,
    pub linux: Option<Linux>,
}
#[derive(Debug, Deserialize)]
pub struct Root { pub path: PathBuf, #[serde(default)] pub readonly: bool }
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Process {
    pub args: Vec<String>,
    #[serde(default)] pub env: Vec<String>,
    #[serde(default = "default_cwd")] pub cwd: String,
    pub user: Option<User>,
}
fn default_cwd() -> String { "/".into() }
#[derive(Debug, Deserialize)]
pub struct User { #[serde(default)] pub uid: u32, #[serde(default)] pub gid: u32 }
#[derive(Debug, Deserialize)]
pub struct Linux { #[serde(default)] pub namespaces: Vec<Namespace>, pub resources: Option<Resources> }
#[derive(Debug, Deserialize)]
pub struct Namespace { #[serde(rename="type")] pub kind: String, pub path: Option<PathBuf> }
#[derive(Debug, Deserialize)]
pub struct Resources { pub memory: Option<Memory>, pub pids: Option<Pids> }
#[derive(Debug, Deserialize)]
pub struct Memory { pub limit: Option<i64> }
#[derive(Debug, Deserialize)]
pub struct Pids { pub limit: Option<i64> }

impl Spec {
    pub fn load(bundle: &Path) -> Result<Self> {
        let path = bundle.join("config.json");
        let raw = fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
        let spec: Spec = serde_json::from_str(&raw).context("invalid config.json")?;
        spec.validate()?;
        Ok(spec)
    }
    pub fn validate(&self) -> Result<()> {
        if self.oci_version.is_empty() { bail!("ociVersion must not be empty"); }
        if self.process.args.is_empty() { bail!("process.args must contain an executable"); }
        if !self.process.cwd.starts_with('/') { bail!("process.cwd must be absolute"); }
        if self.root.path.as_os_str().is_empty() { bail!("root.path must not be empty"); }
        if let Some(linux) = &self.linux {
            let mut seen = std::collections::HashSet::new();
            for ns in &linux.namespaces {
                if !seen.insert(ns.kind.as_str()) { bail!("duplicate namespace type: {}", ns.kind); }
                if !matches!(ns.kind.as_str(), "pid"|"mount"|"uts"|"ipc"|"network"|"user"|"cgroup"|"time") {
                    bail!("unsupported namespace type: {}", ns.kind);
                }
                if ns.path.is_some() { bail!("joining an existing namespace is not implemented"); }
            }
        }
        Ok(())
    }
    pub fn rootfs(&self, bundle: &Path) -> Result<PathBuf> {
        let path = if self.root.path.is_absolute() { self.root.path.clone() } else { bundle.join(&self.root.path) };
        let canonical = path.canonicalize().with_context(|| format!("rootfs does not exist: {}", path.display()))?;
        if !canonical.is_dir() { bail!("rootfs must be a directory"); }
        Ok(canonical)
    }
}
