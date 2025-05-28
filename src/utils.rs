use std::{
    env::{current_dir, temp_dir},
    fs::File,
    io::{stdin, stdout, Write},
    path::PathBuf,
};

use crate::{hpatchz, Error};

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
    let temp_path = temp_dir().join("hpatchz.exe");

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
                Err(Error::Path(cwd.display().to_string()))
            }
        }
    }
}
