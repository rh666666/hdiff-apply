use std::{io::stdout, time::Instant};

mod deletefiles;
mod error;
mod hdiffmap;
mod hpatchz;
mod utils;

use clap::Parser;
use crossterm::{terminal::SetTitle, QueueableCommand};
use deletefiles::DeleteFiles;
use hdiffmap::HDiffMap;

type Error = error::Error;

#[derive(Parser, Debug)]
struct Args {
    #[arg()]
    game_path: Option<String>,
}

fn run() -> Result<(), Error> {
    stdout().queue(SetTitle(format!(
        "{} v{} ",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )))?;

    utils::init_tracing();

    let args = Args::parse();

    let hpatchz_path = utils::get_hpatchz()?;
    let game_path = utils::determine_game_path(args.game_path)?;

    let now = Instant::now();

    let mut delete_files = DeleteFiles::new(&game_path);
    if let Err(e) = delete_files.remove() {
        tracing::error!("{}", e);
    }

    let mut hdiff_map = HDiffMap::new(&game_path, &hpatchz_path);
    if let Err(e) = hdiff_map.patch() {
        tracing::error!("{}", e);
    }

    if delete_files.items > 0 {
        tracing::info!(
            "Deleted {} files listed in deletefiles.txt",
            delete_files.items
        )
    }

    let patch_items = *hdiff_map.items.lock().unwrap();
    if patch_items > 0 {
        tracing::info!("Patched {} files listed in hdiffmap.json", patch_items)
    }

    tracing::info!("Program finished executing in {:.2?}", now.elapsed());

    utils::wait_for_input();
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        tracing::error!("{}", e);
        utils::wait_for_input()
    }
}
