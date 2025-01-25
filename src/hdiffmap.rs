use rayon::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use std::{
    fs::{self, remove_file},
    io::{self},
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex},
};
use thiserror::Error;

pub struct HDiffMap {
    game_path: PathBuf,
    hpatchz_path: PathBuf,
    pub items: Arc<Mutex<u32>>,
}

#[derive(Debug, Error)]
pub enum PatchError {
    #[error("hdiffmap.json structure changed!")]
    JsonError(),
    #[error("{0} doesn't exist, skipping")]
    NotFound(String),
    #[error("IO error occurred: {0}")]
    IoError(#[from] io::Error),
}

#[derive(Deserialize)]
struct DiffMap {
    source_file_name: String,
    target_file_name: String,
    patch_file_name: String,
}

impl HDiffMap {
    pub fn new(game_path: PathBuf, hpatchz_path: PathBuf) -> Self {
        Self {
            game_path,
            hpatchz_path,
            items: Arc::new(Mutex::new(0)),
        }
    }

    fn load_diff_map(&self) -> Result<Vec<DiffMap>, PatchError> {
        let mut path = self.game_path.clone();
        path.push("hdiffmap.json");

        if !path.exists() {
            return Err(PatchError::NotFound(format!("{}", path.display())));
        }

        let data = fs::read_to_string(&path)?;
        let deserialized: Value = serde_json::from_str(&data).unwrap();

        let diff_map = deserialized
            .get("diff_map")
            .ok_or(PatchError::JsonError())?;

        Ok(serde_json::from_value(diff_map.clone()).unwrap())
    }

    pub fn patch(&mut self) -> Result<(), PatchError> {
        let path = &self.game_path;
        let hdiff = &self.load_diff_map()?;

        hdiff.into_par_iter().for_each(|entry| {
            let output = Command::new(&self.hpatchz_path)
                .arg(path.join(&entry.source_file_name))
                .arg(path.join(&entry.patch_file_name))
                .arg(path.join(&entry.target_file_name))
                .output()
                .expect("Failed to execute hpatchz");

            if !output.stdout.is_empty() {
                tracing::info!("{}", String::from_utf8_lossy(&output.stdout).trim());

                let mut items = self.items.lock().unwrap();
                *items += 1;
            }

            if !output.stderr.is_empty() {
                tracing::error!("{}", String::from_utf8_lossy(&output.stderr).trim());
            }
        });

        // Delete old hdiff files (source_file_name, patch_file_name)
        // Should be safe to delete now since we make checks to make sure hpatchz exists
        let files_to_delete: Vec<PathBuf> = hdiff
            .iter()
            .flat_map(|entry| {
                vec![
                    path.join(&entry.source_file_name),
                    path.join(&entry.patch_file_name),
                ]
            })
            .collect();

        for file in files_to_delete {
            if let Err(e) = remove_file(&file) {
                tracing::error!("Failed to remove {}: {}", file.display(), e)
            } else {
                tracing::info!("Removed old hdiff file: {}", file.display())
            }
        }

        Ok(())
    }
}
