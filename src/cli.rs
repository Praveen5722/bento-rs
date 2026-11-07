use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "bento", version, about = "A small Linux container runtime prototype")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Run {
        #[arg(long)] id: String,
        #[arg(long, default_value = ".")] bundle: PathBuf,
        #[arg(long)] no_cgroups: bool,
        #[arg(long)] no_rootfs: bool,
    },
    State { id: String },
    List,
    Delete { id: String },
    Spec,
}
