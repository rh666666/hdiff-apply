use std::{
    io::stdout,
    process,
    time::Instant,
};

mod deletefiles;
mod hdiffmap;
mod hpatchz;
mod utils;

use crossterm::{execute, terminal::SetTitle};
use deletefiles::DeleteFiles;
use hdiffmap::HDiffMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    execute!(
        stdout(),
        SetTitle(format!(
            "{} v{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ))
    )?;

    utils::init_tracing();

    let args: Vec<String> = std::env::args().collect();

    let hpatchz_path = utils::get_hpatchz_path().unwrap_or_else(|err| {
        println!("{}", err);
        utils::wait_for_input();
        process::exit(1);
    });

    let game_path = utils::get_game_path(&args).unwrap_or_else(|err| {
        println!("{}", err);
        utils::wait_for_input();
        process::exit(1);
    });

    let mut delete_files = DeleteFiles::new(&game_path);
    let mut hdiff_map = HDiffMap::new(&game_path, &hpatchz_path);

    let now = Instant::now();

    tracing::info!("Deleting old files...");
    if let Err(e) = delete_files.remove() {
        tracing::error!("{}", e);
    }

    tracing::info!("Patching files...");
    if let Err(e) = hdiff_map.patch() {
        tracing::error!("{}", e);
    }

    (delete_files.items > 0).then(|| {
        tracing::info!(
            "Deleted {} files listed in deletefiles.txt",
            delete_files.items
        )
    });

    let count = *hdiff_map.items.lock().unwrap();
    (count > 0).then(|| tracing::info!("Patched {} files listed in hdiffmap.json", count));

    tracing::info!("Program finished executing in {:.2?}", now.elapsed());
    utils::wait_for_input();

    Ok(())
}
