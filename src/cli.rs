use clap::{ArgGroup, Parser};

#[derive(Parser)]
#[command(
    version,
    about,
    long_about = "A package manager designed for Linux From Scratch (LFS) systems. It automates \
        the process of downloading source code, extracting archives, configuring build \
        environments, compiling software, and installing packages from source.",
    max_term_width = 100,
    group(ArgGroup::new("op").required(true).args(["install", "remove"]))
)]
pub struct Cli {
    #[arg(required = true, value_name = "PKG_NAME")]
    pub pkgs: Vec<String>,

    #[arg(short = 'I', long)]
    pub install: bool,

    #[arg(short = 'R', long)]
    pub remove: bool,
}
