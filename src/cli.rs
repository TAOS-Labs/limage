use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "limage")]
#[command(about = "A tool for building and running kernels", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Build,

    Run {
        #[arg(value_name = "KERNEL")]
        kernel: Option<PathBuf>,
    },

    Clean,
}