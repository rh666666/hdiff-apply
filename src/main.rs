use std::{
    path::{Path, PathBuf},
    time::Instant,
};

mod binary_version;
mod deletefiles;
mod error;
mod hdiffmap;
mod seven_util;
mod utils;
mod verifier;

use binary_version::BinaryVersion;
use clap::Parser;
use deletefiles::DeleteFiles;
use hdiffmap::HDiffMap;
use rand::{distr::Alphanumeric, Rng};
use seven_util::SevenUtil;
use verifier::Verifier;

type Error = error::Error;

pub const TEMP_DIR_NAME: &'static str = "hdiff-apply";

#[derive(Parser, Debug)]
struct Args {
    #[arg()]
    game_path: Option<String>,
    #[arg(long)]
    skip_version_check: bool, // TODO: impl this
}

fn run() -> Result<(), Error> {
    utils::init_tracing();

    utils::set_console_title()?;
    utils::clean_temp_hdiff_data()?;

    let args = Args::parse();

    let temp_dir_path = utils::get_and_create_temp_dir()?;
    let hpatchz_path = utils::get_hpatchz()?;
    let game_path = utils::determine_game_path(args.game_path)?;
    let update_archives_paths = utils::get_update_archives(&game_path)?;

    // Commented out because its not implemented yet
    //if args.skip_version_check {
    //    tracing::warn!("Bypassing version check. This may lead to issues.");
    //}

    tracing::info!("Preparing for update...");

    // <(hdiff_version, temp_path, archive_path)>
    let mut updates_big_vec: Vec<(BinaryVersion, PathBuf, PathBuf)> = vec![];

    // Prepare hdiffs by storing thier paths and versions
    for update_archive in &update_archives_paths {
        let rnd_name: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(5)
            .map(char::from)
            .collect();

        let temp_path = temp_dir_path.join(format!("hdiff_{}", rnd_name));

        SevenUtil::inst().extract_specific_files_to(
            &update_archive,
            &[
                "StarRail_Data\\StreamingAssets\\BinaryVersion.bytes",
                "hdiffmap.json",
                "deletefiles.txt",
            ],
            &temp_path,
        )?;

        let hdiff_version = BinaryVersion::parse(&temp_path.join("BinaryVersion.bytes"))?;

        updates_big_vec.push((hdiff_version, temp_path, update_archive.to_path_buf()));
    }
    updates_big_vec.sort_by(|a, b| a.0.cmp(&b.0));

    // Do some checks to make sure client doesn't brick :)
    let client_version = BinaryVersion::parse(
        &game_path.join("StarRail_Data\\StreamingAssets\\BinaryVersion.bytes"),
    )?;

    let mut prev_version = &client_version;
    let mut start_index = None;

    for (i, (hdiff_version, _, _)) in updates_big_vec.iter().enumerate() {
        if start_index.is_none() {
            if utils::verify_hdiff_version(prev_version, hdiff_version) {
                start_index = Some(i);
                prev_version = hdiff_version;
            }
        } else {
            if !utils::verify_hdiff_version(prev_version, hdiff_version) {
                return Err(Error::InvalidHdiffVersion(
                    prev_version.to_string(),
                    hdiff_version.to_string(),
                ));
            }
            prev_version = hdiff_version;
        }
    }

    if start_index.is_none() {
        let first_hdiff = updates_big_vec
            .last()
            .map(|(v, _, _)| v.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        return Err(Error::InvalidHdiffVersion(
            client_version.to_string(),
            first_hdiff.to_string(),
        ));
    }

    // Everything is correct proceeding further
    let mut sequence = String::new();
    let mut found_start = false;
    let mut prev_version = &client_version;

    for (version, _, _) in updates_big_vec.iter() {
        if !found_start {
            if utils::verify_hdiff_version(prev_version, version) {
                found_start = true;
                sequence.push_str(&format!("{}", prev_version.to_string()));
            } else {
                continue;
            }
        }

        sequence.push_str(&format!(" -> {}", version.patch_version));
        prev_version = version;
    }

    let update_choice = {
        print!(
            "Proceed with this update sequence: {} [Yes/No (default: Yes)]: ",
            sequence
        );
        utils::wait_for_confirmation(true)
    };

    let now = Instant::now();

    if update_choice {
        if let Some(index) = start_index {
            for (_, temp_path, archive_path) in updates_big_vec.iter().skip(index) {
                
                let hdiffmap_path = temp_path.join("hdiffmap.json");
                let deletefiles_path = temp_path.join("deletefiles.txt");
                
                let verifier = Verifier::new(game_path.as_path(), &hdiffmap_path);
                if !verifier.by_file_size()? {
                    tracing::error!("Size mismatch"); // TODO: return error with message
                }

                run_updater(
                    &game_path,
                    &hpatchz_path,
                    archive_path,
                    &hdiffmap_path,
                    &deletefiles_path,
                )?;
            }
        }
    }

    tracing::info!("Updated in {:.2?}", now.elapsed());
    utils::wait_for_input();
    Ok(())
}

fn run_updater(
    game_path: &PathBuf,
    hpatchz_path: &PathBuf,
    hdiff_archive: &PathBuf,
    hdiffmap_path: &Path,
    deletefiles_path: &Path,
) -> Result<(), Error> {
    let archive_str = &hdiff_archive.display().to_string();
    let archive_name = archive_str.split('\\').last().unwrap_or("hdiff");

    tracing::info!("Extracting {}", archive_name);
    SevenUtil::inst().extract_hdiff_to(&hdiff_archive, &game_path)?;

    let mut delete_files = DeleteFiles::new(&game_path);
    if let Err(e) = delete_files.remove(deletefiles_path) {
        tracing::error!("{}", e);
    }

    let mut hdiff_map = HDiffMap::new(&game_path, &hpatchz_path);
    if let Err(e) = hdiff_map.patch(hdiffmap_path) {
        tracing::error!("{}", e);
    }

    if delete_files.count() > 0 {
        tracing::info!(
            "Deleted {} files listed in deletefiles.txt",
            delete_files.count()
        )
    }

    if hdiff_map.count() > 0 {
        tracing::info!(
            "Patched {} files listed in hdiffmap.json",
            hdiff_map.count()
        )
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        tracing::error!("{}", e);
        utils::wait_for_input()
    }
}
