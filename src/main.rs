use std::{
    env,
    io::{stdin, stdout, Write},
    path::PathBuf,
    process,
    time::Instant,
};

mod deletefiles;
mod hdiffmap;

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

    process::exit(1)
}

fn get_hpatchz_from_env() -> PathBuf {
    if let Ok(path_var) = env::var("PATH") {
        let delimiter = if cfg!(windows) { ";" } else { ":" };

        if let Some(hpatchz_path) = path_var.split(delimiter).map(PathBuf::from).find(|p| {
            let hpatchz = p.join("hpatchz.exe");
            hpatchz.is_file()
        }) {
            return hpatchz_path.join("hpatchz.exe");
        }
    }

    println!("Hpatchz not found in path!");
    wait_for_input();

    // Download if not found

    PathBuf::new()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let args: Vec<String> = std::env::args().collect();
    let hpatchz_path = get_hpatchz_from_env();

    let game_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        let cur_dir = env::current_dir()?;
        let sr_exe: PathBuf = cur_dir.join("StarRail.exe");

        if sr_exe.is_file() {
            cur_dir
        } else {
            println!("Usage: {} [game_folder]", &args[0]);
            wait_for_input();

            PathBuf::new()
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

    tracing::info!(
        "Deleted {} files listed in deletefiles.txt",
        delete_files.items
    );
    tracing::info!(
        "Patched {} files listed in hdiffmap.json",
        hdiff_map.items.lock().unwrap()
    );

    tracing::info!("Finished in {:.2?}", now.elapsed());

    wait_for_input();

    Ok(())
}
