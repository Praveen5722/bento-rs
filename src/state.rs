use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};
const STATE_ROOT: &str = "/run/bento";

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerState {
    pub id: String, pub status: String, pub pid: Option<u32>, pub bundle: String, pub created_at: u64,
}
fn state_path(id: &str) -> PathBuf { Path::new(STATE_ROOT).join(format!("{id}.json")) }
pub fn save(id: &str, status: &str, pid: Option<u32>, bundle: &Path) -> Result<()> {
    fs::create_dir_all(STATE_ROOT).context("create state directory")?;
    let created_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let s = ContainerState { id:id.into(), status:status.into(), pid, bundle:bundle.display().to_string(), created_at };
    fs::write(state_path(id), serde_json::to_vec_pretty(&s)?).context("write state")
}
pub fn load(id: &str) -> Result<ContainerState> {
    Ok(serde_json::from_slice(&fs::read(state_path(id)).with_context(|| format!("no state for {id}"))?)?)
}
pub fn list() -> Result<Vec<ContainerState>> {
    let dir = Path::new(STATE_ROOT);
    if !dir.exists() { return Ok(Vec::new()); }
    let mut out = Vec::new();
    for e in fs::read_dir(dir)? {
        let e = e?;
        if e.path().extension().and_then(|s| s.to_str()) == Some("json") {
            if let Ok(b) = fs::read(e.path()) { if let Ok(s) = serde_json::from_slice(&b) { out.push(s); } }
        }
    }
    out.sort_by(|a: &ContainerState,b| a.id.cmp(&b.id));
    Ok(out)
}
pub fn delete(id: &str) -> Result<()> {
    let p = state_path(id);
    if p.exists() { fs::remove_file(p).context("delete state")?; }
    Ok(())
}
