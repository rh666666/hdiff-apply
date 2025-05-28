use std::{
    env::{current_dir, temp_dir},
    fs::File,
    io::{stdin, stdout, Write},
    path::PathBuf,
};

use crate::hpatchz;

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

pub fn get_hpatchz_path() -> Result<PathBuf, &'static str> {
    let temp_path = temp_dir().join("hpatchz.exe");

    let mut file = File::create(&temp_path).map_err(|_| "Failed to create hpatchz file")?;
    file.write_all(hpatchz::HPATCHZ_EXE)
        .map_err(|_| "Failed to write binary")?;

    Ok(temp_path)
}

pub fn get_game_path(args: &[String]) -> Result<PathBuf, String> {
    if args.len() > 1 {
        return Ok(PathBuf::from(&args[1]));
    }

    let cwd = current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    let sr_exe = cwd.join("StarRail.exe");

    if sr_exe.is_file() {
        return Ok(cwd);
    } else {
        Err(format!("Usage: {} [game_folder]", args[0]))
    }
}
