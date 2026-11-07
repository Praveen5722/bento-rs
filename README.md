# Bento.rs

Bento.rs is an educational Linux container-runtime prototype written in Rust. It demonstrates how a runtime can read an OCI-style bundle, create Linux namespaces, apply cgroup v2 limits, enter a root filesystem, and execute a process.

> **Security status:** this is a learning prototype, not a production container runtime. It is not yet OCI-conformant and must not be used to execute untrusted workloads. It currently uses chroot rather than a hardened pivot_root flow, does not install seccomp filters, does not comprehensively drop Linux capabilities, and does not implement user-namespace ID mapping, mount allowlists, networking setup, hooks, or the full OCI lifecycle.

## Features
- Rust CLI using clap
- OCI-style config.json parsing and basic validation
- Linux PID, mount, UTS, IPC, network, user, cgroup and time namespaces
- cgroup v2 memory and PID limits
- Root filesystem entry using chroot
- /proc mount inside the private mount namespace
- Runtime state files and basic run, state, list, delete and spec commands

## Requirements
- Linux kernel with namespace support
- cgroup v2 mounted at /sys/fs/cgroup
- Rust stable toolchain
- A Linux root filesystem containing the executable and its shared libraries
- Root privileges or correctly delegated namespace and cgroup permissions

This project is Linux-only. Use an Ubuntu VM for development.

## Build
```bash
cargo fmt --all
cargo check
cargo test
cargo build --release
```

## Prepare a root filesystem
The rootfs directory is intentionally empty. In a disposable Ubuntu/Debian lab VM:

```bash
sudo apt update
sudo apt install -y debootstrap
sudo debootstrap --variant=minbase jammy ./rootfs http://archive.ubuntu.com/ubuntu
sudo mkdir -p ./rootfs/proc ./rootfs/dev ./rootfs/tmp
```

Never use your host / as the container rootfs.

## Run
Copy config/config.json into a bundle directory next to rootfs and ensure root.path resolves correctly:

```bash
mkdir -p bundle
cp config/config.json bundle/config.json
sudo cargo run -- run --id demo --bundle ./bundle
```

Namespace-only experiment (not a secure container):

```bash
sudo cargo run -- run --id ns-demo --bundle ./bundle --no-cgroups --no-rootfs
```

## Inspect
```bash
sudo cargo run -- state demo
sudo cargo run -- list
sudo cargo run -- delete demo
```

## Architecture
```text
bento CLI
  └── runtime
      ├── config       OCI-style config parsing / validation
      ├── namespaces  unshare(2), mount propagation
      ├── cgroups     cgroup v2 memory.max / pids.max
      ├── filesystem  chroot, /proc mount, working directory
      ├── security    explicit security roadmap/status
      └── state       runtime metadata
Linux kernel
```

## Limitations
- run is synchronous; create, start, exec, kill and events are not implemented.
- Existing namespace paths are rejected.
- OverlayFS image/layer management is not implemented.
- Seccomp and capability dropping are not implemented.
- No device nodes, /dev, /sys, network interfaces, DNS or port mapping setup.
- cgroup delegation/controller enabling is environment-dependent.
- chroot alone is not a security boundary.

## Roadmap
1. Harden filesystem setup with pivot_root and mount allowlists.
2. Implement full OCI lifecycle commands.
3. Implement user namespaces and UID/GID mappings.
4. Add capability dropping and seccomp profiles.
5. Add OverlayFS layers.
6. Add network namespace and veth setup.
7. Validate against OCI runtime-spec schemas and tests.
8. Add integration tests in a disposable Linux VM and CI.

## References
- https://github.com/opencontainers/runtime-spec
- https://github.com/opencontainers/runtime-tools
- https://man7.org/linux/man-pages/man7/namespaces.7.html
- https://docs.kernel.org/admin-guide/cgroup-v2.html

## License
MIT
