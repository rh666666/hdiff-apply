use std::{io::stdout, path::PathBuf, time::Instant};

mod binary_version;
mod deletefiles;
mod error;
mod hdiffmap;
mod seven_util;
mod utils;

use binary_version::BinaryVersion;
use clap::Parser;
use crossterm::{terminal::SetTitle, QueueableCommand};
use deletefiles::DeleteFiles;
use hdiffmap::HDiffMap;
use seven_util::SevenUtil;

type Error = error::Error;

#[derive(Default, Debug)]
enum HdiffProcedure {
    #[default]
    Update,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg()]
    game_path: Option<String>,
    #[arg(long)]
    skip_version_check: bool,
}

pub const TEMP_DIR_NAME: &'static str = "hdiff-apply";

fn run() -> Result<(), Error> {
    utils::init_tracing();

    stdout().queue(SetTitle(format!(
        "{} v{} | Made by nie",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )))?;

    let args = Args::parse();
    let mut procedures = Vec::new();

    let temp_dir_path = utils::create_temp_dir(TEMP_DIR_NAME)?;
    let hpatchz_path = utils::get_hpatchz()?;
    let game_path = utils::determine_game_path(args.game_path)?;
    let update_archive_path = utils::get_update_archive(&game_path)?;

    if args.skip_version_check {
        tracing::warn!("Bypassing version check. This may lead to issues.");
    }

    SevenUtil::inst().extract_specific_file_to(
        &update_archive_path,
        "StarRail_Data\\StreamingAssets\\BinaryVersion.bytes",
        &temp_dir_path,
    )?;

    let client_version = BinaryVersion::parse(
        &game_path.join("StarRail_Data\\StreamingAssets\\BinaryVersion.bytes"),
    )?;
    let hdiff_version = BinaryVersion::parse(&temp_dir_path.join("BinaryVersion.bytes"))?;

    let versions_match = utils::verify_hdiff_version(&client_version, &hdiff_version);
    if !versions_match && !args.skip_version_check {
        return Err(Error::InvalidHdiffVersion(
            client_version.to_string(),
            hdiff_version.to_string(),
        ));
    }

    let update_choice = {
        print!(
            "Update client from {} to {} [Yes/No (default: Yes)]: ",
            client_version.to_string(),
            hdiff_version.to_string()
        );
        utils::wait_for_confirmation(true)
    };

    if update_choice {
        procedures.push(HdiffProcedure::Update);
    }

    let now = Instant::now();

    run_procedures(&procedures, &game_path, &hpatchz_path)?;

    tracing::info!("Updated in {:.2?}", now.elapsed());
    utils::wait_for_input();
    Ok(())
}

fn run_procedures(
    procedures: &[HdiffProcedure],
    game_path: &PathBuf,
    hpatchz_path: &PathBuf,
) -> Result<(), Error> {
    for proc in procedures {
        match proc {
            HdiffProcedure::Update => {
                let update_archive_path = utils::get_update_archive(&game_path)?;

                let archive_str = &update_archive_path.display().to_string();
                let archive_name = archive_str.split('\\').last().unwrap_or("hdiff");

                tracing::info!("Extracting {}", archive_name);
                SevenUtil::inst().extract_to(&update_archive_path, &game_path)?;

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
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        tracing::error!("{}", e);
        utils::wait_for_input()
    }
}
