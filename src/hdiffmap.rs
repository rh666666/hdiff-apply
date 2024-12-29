use serde::Deserialize;
use serde_json::Value;
use std::{
    fs::{self},
    io::{self},
    path::PathBuf,
    process::Command,
};

use thiserror::Error;

pub struct HDiffMap {
    game_path: PathBuf,
    pub items: u32,
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
pub struct DiffMap {
    source_file_name: String,
    target_file_name: String,
    patch_file_name: String,
}

impl HDiffMap {
    pub fn new(game_path: &PathBuf) -> Self {
        Self {
            game_path: game_path.clone(),
            items: 0,
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
        let hdiff = self.load_diff_map()?;

        for entry in &hdiff {
            let output = Command::new("hpatchz")
                .arg(path.join(&entry.source_file_name))
                .arg(path.join(&entry.patch_file_name))
                .arg(path.join(&entry.target_file_name))
                .output()
                .expect("Failed to execute hpatchz");

            if !output.stdout.is_empty() {
                tracing::info!("{}", String::from_utf8_lossy(&output.stdout).trim());
                self.items += 1;
            }

            if !output.stderr.is_empty() {
                tracing::error!("{}", String::from_utf8_lossy(&output.stderr).trim());
            }
        }

        Ok(())
    }
}
