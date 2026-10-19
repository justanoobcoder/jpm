use clap::Parser;

#[derive(Parser)]
#[command(
    version,
    about,
    long_about = "A package manager designed for Linux From Scratch (LFS) systems. It automates \
        the process of downloading source code, extracting archives, configuring build \
        environments, compiling software, and installing packages from source.",
    max_term_width = 100
)]
pub struct Cli {}
