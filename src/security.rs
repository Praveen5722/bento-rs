use anyhow::Result;

/// Security roadmap placeholder. No seccomp filter or capability dropping is active yet.
pub fn security_status() -> Result<&'static str> {
    Ok("seccomp and capability dropping are not implemented; do not run untrusted workloads")
}
