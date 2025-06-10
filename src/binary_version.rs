use std::{fs::File, io::Read, path::PathBuf};

use crate::Error;

#[derive(Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct BinaryVersion {
    pub major_version: u32,
    pub minor_version: u32,
    pub patch_version: u32,
}

impl BinaryVersion {
    pub fn parse(binary_version_path: &PathBuf) -> Result<Self, Error> {
        let mut file = File::open(binary_version_path)?;
        let mut buf = [0; 266];
        let n = file.read(&mut buf)?;

        let content = String::from_utf8_lossy(&buf[..n]);

        let dash_pos = content.rfind('-').unwrap();
        let start_pos = dash_pos.saturating_sub(6);
        let version_slice: &str = &content[start_pos..];

        let version_end = version_slice.find('-').unwrap_or(version_slice.len());
        let version_str = &version_slice[..version_end];

        let parts: Vec<u32> = version_str
            .split('.')
            .take(3)
            .filter_map(|s| s.parse::<u32>().ok())
            .collect();

        if parts.len() != 3 {
            return Err(Error::VersionParse());
        }

        Ok(Self {
            major_version: parts[0],
            minor_version: parts[1],
            patch_version: parts[2],
        })
    }
}

impl ToString for BinaryVersion {
    fn to_string(&self) -> String {
        format!(
            "{}.{}.{}",
            self.major_version, self.minor_version, self.patch_version
        )
    }
}
