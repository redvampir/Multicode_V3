mod app;
mod modal;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    file: Option<PathBuf>,
    #[arg(long)]
    project: Option<PathBuf>,
}

pub fn main() -> iced::Result {
    let args = Args::parse();
    let path = if let Some(file) = args.file {
        file.parent().map(|p| p.to_path_buf())
    } else {
        args.project
    };
    app::run(path)
}
