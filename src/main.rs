mod cgroups;
mod cli;
mod config;
mod filesystem;
mod namespaces;
mod runtime;
mod security;
mod state;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();
    runtime::dispatch(cli::Cli::parse())
}
