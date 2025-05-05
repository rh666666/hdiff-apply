use std::{
    fs::{remove_file, File},
    io::{BufRead, BufReader},
    path::Path,
};

use thiserror::Error;

pub struct DeleteFiles<'a> {
    game_path: &'a Path,
    pub items: u32,
}

#[derive(Debug, Error)]
pub enum DeleteFileError {
    #[error("{0} doesn't exist, skipping")]
    NotFound(String),
    #[error("IO error occurred: {0}")]
    IoError(#[from] std::io::Error),
}

impl<'a> DeleteFiles<'a> {
    pub fn new(game_path: &'a Path) -> Self {
        Self {
            game_path,
            items: 0,
        }
    }

    pub fn remove(&mut self) -> Result<(), DeleteFileError> {
        let path = self.game_path.join("deletefiles.txt");

        if !path.exists() {
            return Err(DeleteFileError::NotFound(format!("{}", path.display())));
        }

        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let path = Path::new(&line);

            let full_path = &self.game_path.join(path);

            match remove_file(&full_path) {
                Ok(_) => {
                    tracing::info!("Deleted {}", full_path.display());
                    self.items += 1;
                },
                Err(e) => tracing::error!("Failed to delete {}: {}", full_path.display(), e),
            }
        }

        Ok(())
    }
}
