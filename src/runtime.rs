use anyhow::{bail, Context, Result};
use std::{os::unix::process::CommandExt, path::PathBuf, process::Command};
use tracing::{info, warn};
use crate::{cgroups::{self, Cgroup}, cli::{Cli, Commands}, config::Spec, filesystem, namespaces, state};

pub fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Run{id,bundle,no_cgroups,no_rootfs} => run(&id,&bundle,no_cgroups,no_rootfs),
        Commands::State{id} => { println!("{}", serde_json::to_string_pretty(&state::load(&id)?)?); Ok(()) }
        Commands::List => {
            for s in state::list()? { println!("{:<24} {:<12} pid={}",s.id,s.status,s.pid.map(|p|p.to_string()).unwrap_or_else(||"-".into())); }
            Ok(())
        }
        Commands::Delete{id} => {
            cgroups::validate_id(&id)?;
            if let Ok(s)=state::load(&id) { if s.status=="running" { bail!("container is marked running; stop it first"); } }
            state::delete(&id)?;
            let p=std::path::Path::new("/sys/fs/cgroup/bento").join(&id);
            if p.exists() { std::fs::remove_dir(p).context("remove cgroup")?; }
            println!("deleted {id}"); Ok(())
        }
        Commands::Spec => { println!("{}",include_str!("../config/config.json")); Ok(()) }
    }
}
fn run(id:&str,bundle:&PathBuf,no_cgroups:bool,no_rootfs:bool)->Result<()> {
    cgroups::validate_id(id)?;
    let bundle=bundle.canonicalize().context("bundle directory does not exist")?;
    let spec=Spec::load(&bundle)?;
    let rootfs=if no_rootfs {None} else {Some(spec.rootfs(&bundle)?)};
    if state::load(id).is_ok() { bail!("container ID already exists: {id}"); }
    let cgroup=if no_cgroups {None} else {Some(Cgroup::create(id,&spec)?)};
    namespaces::unshare_for_container(&spec,no_rootfs)?;
    if let Some(h)=&spec.hostname {
        let n=std::ffi::CString::new(h.as_str())?;
        if unsafe {libc::sethostname(n.as_ptr(),h.len())}!=0 {return Err(std::io::Error::last_os_error()).context("sethostname failed");}
    }
    let mut cmd=Command::new(&spec.process.args[0]);
    cmd.args(spec.process.args.iter().skip(1)).env_clear();
    for e in &spec.process.env { if let Some((k,v))=e.split_once('=') {cmd.env(k,v);} }
    let root=rootfs.clone();
    let cwd=spec.process.cwd.clone();
    let cg=cgroup.as_ref().map(|c|c.path.clone());
    let readonly=spec.root.readonly;
    let uid=spec.process.user.as_ref().map(|u|u.uid).unwrap_or(0);
    let gid=spec.process.user.as_ref().map(|u|u.gid).unwrap_or(0);
    unsafe {
        cmd.pre_exec(move || {
            if let Some(p)=&cg { std::fs::write(p.join("cgroup.procs"),std::process::id().to_string())?; }
            if let Some(r)=&root {
                filesystem::prepare_rootfs(r,readonly).map_err(|e|std::io::Error::other(e.to_string()))?;
                filesystem::enter_workdir(&cwd).map_err(|e|std::io::Error::other(e.to_string()))?;
            }
            if gid!=0 && libc::setgid(gid)!=0 {return Err(std::io::Error::last_os_error());}
            if uid!=0 && libc::setuid(uid)!=0 {return Err(std::io::Error::last_os_error());}
            Ok(())
        });
    }
    info!(container=id,"starting container process");
    let mut child=cmd.spawn().context("spawn container process; check executable in rootfs")?;
    state::save(id,"running",Some(child.id()),&bundle)?;
    let status=child.wait().context("wait for container")?;
    state::save(id,"stopped",Some(child.id()),&bundle)?;
    if let Some(c)=cgroup {if let Err(e)=c.remove(){warn!(error=%e,"could not remove cgroup");}}
    info!(container=id,exit_status=%status,"container exited");
    Ok(())
}
