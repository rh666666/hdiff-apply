use std::{
    fs::{remove_file, File},
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use thiserror::Error;

pub struct DeleteFiles {
    game_path: PathBuf,
    pub items: u32,
}

#[derive(Debug, Error)]
pub enum FileError {
    #[error("{0} doesn't exist, skipping")]
    NotFound(String),
    #[error("IO error occurred: {0}")]
    IoError(#[from] io::Error),
}

impl DeleteFiles {
    pub fn new(game_path: &PathBuf) -> Self {
        Self {
            game_path: game_path.clone(),
            items: 0,
        }
    }

    pub fn remove(&mut self) -> Result<(), FileError> {
        let mut path = self.game_path.clone();
        path.push("deletefiles.txt");

        if !path.exists() {
            return Err(FileError::NotFound(format!("{}", path.display())));
        }

        let deletefile_content = File::open(&path)?;
        let reader = BufReader::new(deletefile_content);

        for line in reader.lines() {
            let line = line?;
            let file = Path::new(&line);

            let full_path = &self.game_path.join(file);

            match remove_file(&full_path) {
                Ok(_) => {
                    self.items += 1;
                }
                Err(e) => tracing::error!("Failed to delete {}: {}", full_path.display(), e),
            }
        }

        Ok(())
    }
}
