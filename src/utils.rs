use std::{
    env::{current_dir, temp_dir},
    fs::{create_dir, File},
    io::{stdin, stdout, Write},
    path::PathBuf,
};

use crate::{binary_version::BinaryVersion, hpatchz, Error, TEMP_DIR_NAME};

pub fn init_tracing() {
    #[cfg(target_os = "windows")]
    let _ = ansi_term::enable_ansi_support();

    tracing_subscriber::fmt()
        .without_time()
        .with_max_level(tracing::Level::DEBUG)
        .init();
}

pub fn wait_for_input() {
    print!("Press enter to exit");
    stdout().flush().unwrap();

    stdin().read_line(&mut String::new()).unwrap();
}

pub fn get_hpatchz() -> Result<PathBuf, Error> {
    let temp_path = temp_dir().join(TEMP_DIR_NAME).join("hpatchz.exe");

    let mut file = File::create(&temp_path)?;
    file.write_all(hpatchz::HPATCHZ_EXE)?;

    Ok(temp_path)
}

pub fn determine_game_path(game_path: Option<String>) -> Result<PathBuf, Error> {
    match game_path {
        Some(path) => Ok(PathBuf::from(path)),
        None => {
            let cwd = current_dir()?;
            let sr_exe = cwd.join("StarRail.exe");

            if sr_exe.is_file() {
                Ok(cwd)
            } else {
                Err(Error::PathNotFound(cwd.display().to_string()))
            }
        }
    }
}

pub fn wait_for_confirmation(default_choice: bool) -> bool {
    stdout().flush().unwrap();

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => return true,
        "n" | "no" => return false,
        _ => return default_choice,
    }
}

pub fn get_update_archive(game_path: &PathBuf) -> Result<PathBuf, Error> {
    for entry in game_path.read_dir()? {
        let entry = entry?;
        let path = entry.path();

        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            if name.to_lowercase().ends_with(".7z") {
                return Ok(path);
            }
        }
    }

    Err(Error::ArchiveNotFound())
}

pub fn create_temp_dir(path: &str) -> Result<PathBuf, Error> {
    let path = temp_dir().join(path);
    if !path.exists() {
        create_dir(&path)?;
    }
    Ok(path)
}

pub fn verify_hdiff_version(client_version: &BinaryVersion, hdiff_version: &BinaryVersion) -> bool {
    client_version.major_version == hdiff_version.major_version
        && client_version.minor_version == hdiff_version.minor_version
        && hdiff_version.patch_version == client_version.patch_version + 1
}
