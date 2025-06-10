use std::{
    fs::File,
    io::{Seek, SeekFrom},
    path::Path,
};

use serde::Deserialize;
use serde_json::Value;

use crate::Error;

#[derive(Deserialize)]
struct DiffMap {
    source_file_name: String,
    source_file_size: u64,
}

pub struct Verifier<'a, 'b> {
    game_path: &'a Path,
    hdiff_map_path: &'b Path,
}

impl<'a, 'b> Verifier<'a, 'b> {
    pub fn new(game_path: &'a Path, hdiff_map_path: &'b Path) -> Self {
        Self {
            game_path,
            hdiff_map_path,
        }
    }

    fn load_diff_map(&self) -> Result<Vec<DiffMap>, Error> {
        let data = std::fs::read_to_string(&self.hdiff_map_path)?;
        let deserialized: Value = serde_json::from_str(&data).unwrap();

        let diff_map = deserialized.get("diff_map").unwrap();
        
        Ok(serde_json::from_value(diff_map.clone()).unwrap())
    }

    pub fn by_file_size(&self) -> Result<bool, Error> {
        let hdiff_map = self.load_diff_map()?;

        for diff_map in &hdiff_map {
            let expected_size = diff_map.source_file_size;
            let source_file_path = self.game_path.join(&diff_map.source_file_name);

            let mut source_file = File::open(source_file_path)?;
            let source_file_size = source_file.seek(SeekFrom::End(0))?;

            if source_file_size != expected_size {
                return Ok(false);
            }
        }

        Ok(true)
    }

    // Could also verify by MD5 but it will take a lot of time
}
