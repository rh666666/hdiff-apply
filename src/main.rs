use std::{
    env::{self, temp_dir},
    fs::File,
    io::{stdin, stdout, Write},
    path::PathBuf,
    process,
    time::Instant,
};

mod deletefiles;
mod hdiffmap;
mod hpatchz;

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
    let temp_path = temp_dir().join(hpatchz_filename);

    let mut file = File::create(&temp_path).map_err(|_| "Failed to create hpatchz file")?;
    file.write_all(hpatchz::EMBEDDED_BINARY).map_err(|_| "Failed to write binary")?;

    Ok(temp_path)
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

    tracing::info!("Deleting old files...");
    if let Err(e) = delete_files.remove() {
        tracing::error!("{}", e);
    }

    tracing::info!("Patching files...");
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
