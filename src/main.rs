use std::{
    env,
    io::{stdin, stdout, Write},
    path::PathBuf,
    process,
    time::Instant,
};

mod deletefiles;
mod hdiffmap;

use crossterm::{execute, terminal::SetTitle};
use deletefiles::DeleteFiles;
use hdiffmap::HDiffMap;

fn init_tracing() {
    #[cfg(target_os = "windows")]
    let _ = ansi_term::enable_ansi_support();

    tracing_subscriber::fmt()
        .without_time()
        .with_max_level(tracing::Level::DEBUG)
        .init();
}

fn wait_for_input() {
    print!("Press enter to exit");
    stdout().flush().unwrap();

    stdin().read_line(&mut String::new()).unwrap();
}

fn get_hpatchz_path() -> Result<PathBuf, &'static str> {
    let hpatchz_filename = "hpatchz.exe";

    // Find hpatchz in the current directory first
    let local_path = PathBuf::from(hpatchz_filename);
    if local_path.is_file() {
        return Ok(local_path);
    }

    // Find hpatchz in system PATH
    let path_var = env::var("PATH").map_err(|_| "Failed to read PATH environment variable")?;
    path_var
        .split(";")
        .map(PathBuf::from)
        .find(|p| p.join(hpatchz_filename).is_file())
        .ok_or("hpatchz not found in current directory or system PATH")
}

fn get_game_path(args: &[String]) -> Result<PathBuf, String> {
    if args.len() > 1 {
        return Ok(PathBuf::from(&args[1]));
    }

    let cur_dir = env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    let sr_exe: PathBuf = cur_dir.join("StarRail.exe");

    if sr_exe.is_file() {
        return Ok(cur_dir);
    } else {
        Err(format!("Usage: {} [game_folder]", args[0]))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    execute!(
        stdout(),
        SetTitle(format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")))
    )?;

    init_tracing();

    let args: Vec<String> = std::env::args().collect();

    let hpatchz_path = match get_hpatchz_path() {
        Ok(path) => path,
        Err(err) => {
            println!("{}", err);
            wait_for_input();
            process::exit(1)
        }
    };

    let game_path = match get_game_path(&args) {
        Ok(path) => path,
        Err(err) => {
            println!("{}", err);
            wait_for_input();
            process::exit(1)
        }
    };

    let mut delete_files = DeleteFiles::new(&game_path);
    let mut hdiff_map = HDiffMap::new(game_path, hpatchz_path);

    let now = Instant::now();

    if let Err(e) = delete_files.remove() {
        tracing::error!("{}", e);
    }

    if let Err(e) = hdiff_map.patch() {
        tracing::error!("{}", e);
    }

    (delete_files.items > 0).then(|| {
        tracing::info!("Deleted {} files listed in deletefiles.txt", delete_files.items)
    });

    let count = *hdiff_map.items.lock().unwrap();
    (count > 0).then(|| {
        tracing::info!("Patched {} files listed in hdiffmap.json", count)
    });

    tracing::info!("Finished in {:.2?}", now.elapsed());

    wait_for_input();

    Ok(())
}
