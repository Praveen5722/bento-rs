use anyhow::{Context, Result};
use std::{ffi::CString, os::unix::ffi::OsStrExt, path::Path};

pub fn prepare_rootfs(rootfs: &Path, readonly: bool) -> Result<()> {
    let root = CString::new(rootfs.as_os_str().as_bytes())?;
    if unsafe { libc::chroot(root.as_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("chroot failed");
    }
    let slash = CString::new("/")?;
    if unsafe { libc::chdir(slash.as_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("chdir after chroot failed");
    }
    let proc = CString::new("proc")?;
    let target = CString::new("/proc")?;
    let fstype = CString::new("proc")?;
    let flags = (libc::MS_NOSUID|libc::MS_NODEV|libc::MS_NOEXEC) as libc::c_ulong;
    if unsafe { libc::mount(proc.as_ptr(), target.as_ptr(), fstype.as_ptr(), flags, std::ptr::null()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("mount /proc failed");
    }
    if readonly && unsafe { libc::mount(std::ptr::null(), slash.as_ptr(), std::ptr::null(),
        (libc::MS_BIND|libc::MS_REMOUNT|libc::MS_RDONLY) as libc::c_ulong, std::ptr::null()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("remount rootfs read-only failed");
    }
    Ok(())
}
pub fn enter_workdir(cwd: &str) -> Result<()> {
    let p = CString::new(cwd)?;
    if unsafe { libc::chdir(p.as_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("container working directory does not exist");
    }
    Ok(())
}
