use rayon::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use std::{
    io::{self},
    path::{Path, PathBuf},
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

        let data = std::fs::read_to_string(&path)?;
        let deserialized: Value = serde_json::from_str(&data).unwrap();

        let diff_map = deserialized
            .get("diff_map")
            .ok_or(PatchError::JsonError())?;

        Ok(serde_json::from_value(diff_map.clone()).unwrap())
    }

    fn remove_file<P: AsRef<Path>>(&self, path: P) {
        match std::fs::remove_file(&path) {
            Ok(_) => tracing::info!("Removed old hdiff file: {}", path.as_ref().display()),
            Err(e) => tracing::error!("Failed to remove {}: {}", path.as_ref().display(), e),
        }
    }

    pub fn patch(&mut self) -> Result<(), PatchError> {
        let path = &self.game_path;
        let hdiff = &self.load_diff_map()?;

        hdiff.into_par_iter().for_each(|entry| {
            let source_file_name = &entry.source_file_name;
            let patch_file_name = &entry.patch_file_name;
            let target_file_name = &entry.target_file_name;

            let output = Command::new(&self.hpatchz_path)
                .arg(path.join(source_file_name))
                .arg(path.join(patch_file_name))
                .arg(path.join(target_file_name))
                .output()
                .unwrap();

            let mut items = self.items.lock().unwrap();

            if !output.stdout.is_empty() {
                tracing::info!("{}", String::from_utf8_lossy(&output.stdout).trim());

                *items += 1;
            }

            if !output.stderr.is_empty() {
                tracing::error!("{}", String::from_utf8_lossy(&output.stderr).trim());
            }

            if *items > 0 {
                self.remove_file(source_file_name);
                self.remove_file(patch_file_name);
            }
        });

        Ok(())
    }
}
