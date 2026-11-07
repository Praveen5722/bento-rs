use anyhow::{bail, Context, Result};
use crate::config::Spec;

pub fn unshare_for_container(spec: &Spec, no_rootfs: bool) -> Result<()> {
    let mut flags: libc::c_int = 0;
    if let Some(linux) = &spec.linux {
        for ns in &linux.namespaces {
            flags |= match ns.kind.as_str() {
                "pid" => libc::CLONE_NEWPID,
                "mount" => libc::CLONE_NEWNS,
                "uts" => libc::CLONE_NEWUTS,
                "ipc" => libc::CLONE_NEWIPC,
                "network" => libc::CLONE_NEWNET,
                "user" => libc::CLONE_NEWUSER,
                "cgroup" => libc::CLONE_NEWCGROUP,
                "time" => libc::CLONE_NEWTIME,
                other => bail!("unsupported namespace: {other}"),
            };
        }
    }
    if no_rootfs { flags &= !libc::CLONE_NEWNS; }
    if flags != 0 && unsafe { libc::unshare(flags) } != 0 {
        return Err(std::io::Error::last_os_error()).context("unshare failed; check kernel support and privileges");
    }
    if flags & libc::CLONE_NEWNS != 0 {
        let rc = unsafe { libc::mount(std::ptr::null(), b"/\0".as_ptr() as *const libc::c_char,
            std::ptr::null(), (libc::MS_REC|libc::MS_PRIVATE) as libc::c_ulong, std::ptr::null()) };
        if rc != 0 { return Err(std::io::Error::last_os_error()).context("make mount propagation private"); }
    }
    Ok(())
}
