use std::{
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
    let _ = ansi_term::enable_ansi_support();
    tracing_subscriber::fmt().without_time().init();
}

fn wait_for_input() {
    print!("Press enter to exit");
    stdout().flush().unwrap();

    stdin().read_line(&mut String::new()).unwrap();

    process::exit(1)
}

fn main() {
    init_tracing();

    let args: Vec<String> = std::env::args().collect();
    let mut path = PathBuf::new();

    if args.len() > 1 {
        path = PathBuf::from(&args[1]);
    } else {
        let mut executing_dir: Vec<&str> = args[0].split("\\").collect();
        executing_dir.pop();

        let game_path: PathBuf = PathBuf::from(executing_dir.join("\\"));
        let sr_exe = game_path.join("StarRail.exe");

        if sr_exe.exists() && sr_exe.is_file() {
            path = game_path
        } else {
            println!("Usage: {} [game_folder]", &args[0]);

            wait_for_input();
        }
    }

    let mut delete_files = DeleteFiles::new(&path);
    let mut hdiff_map = HDiffMap::new(&path);

    let mut log_msg: Vec<String> = Vec::new();

    let now = Instant::now();

    match delete_files.remove() {
        Ok(_) => log_msg.push(format!(
            "Deleted {} files listed in deletefiles.txt",
            delete_files.items
        )),
        Err(e) => tracing::error!("{}", e),
    }

    match hdiff_map.patch() {
        Ok(_) => log_msg.push(format!(
            "Patched {} files listed in hdiffmap.json",
            hdiff_map.items.lock().unwrap()
        )),
        Err(e) => tracing::error!("{}", e),
    }

    for msg in log_msg {
        tracing::info!("{msg}");
    }

    tracing::info!("Finished in {:.2?}", now.elapsed());

    wait_for_input();
}
