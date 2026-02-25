//! Точка входа в приложение Semaphore CLI

use clap::Parser;
use semaphore::cli::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.run()
}
