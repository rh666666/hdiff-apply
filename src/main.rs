use std::{
    io::{stdin, stdout, Write},
    path::PathBuf,
    process,
};

mod deletefiles;
mod hdiffmap;

use deletefiles::DeleteFiles;
use hdiffmap::HDiffMap;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} [game_folder]", &args[0]);

        print!("Press enter to exit");
        stdout().flush().unwrap();

        stdin().read_line(&mut String::new()).unwrap();

        process::exit(1)
    }

    let path = PathBuf::from(&args[1]);

    let mut delete_files = DeleteFiles::new(&path);
    let mut hdiff_map = HDiffMap::new(&path);

    match delete_files.remove() {
        Ok(_) => println!("Deleted {} files listed in deletefiles.txt", delete_files.items),
        Err(e) => eprintln!("Error: {}", e),
    }

    match hdiff_map.patch() {
        Ok(_) => println!("Patched {} files listed in hdiffmap.json", hdiff_map.items),
        Err(e) => eprintln!("Error: {}", e),
    }
}
