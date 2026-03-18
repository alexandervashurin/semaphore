//! Точка входа в приложение Velum CLI

use clap::Parser;
use semaphore_ffi::cli::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.run()
}
