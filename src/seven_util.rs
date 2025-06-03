// I didnt find any good 7z crates so this will have to do for now

use std::{path::PathBuf, process::Command, sync::OnceLock};

use thiserror::Error;
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use crate::utils;

static INST: OnceLock<SevenUtil> = OnceLock::new();

#[derive(Error, Debug)]
pub enum SevenError {
    #[error("Failed to open 7-zip registry subkey. Make sure 7-zip is installed")]
    RegOpenFailed(#[source] std::io::Error),
    #[error("Failed to read 7-zip path value. Make sure 7-zip is installed")]
    RegGetValueFailed(#[source] std::io::Error),
    #[error("7z.exe not found at expected path: '{0}'. Make sure 7-zip is installed")]
    SevenZipNotFound(String),
    #[error("7-zip failed to run using Command")]
    CommandError(#[source] std::io::Error),
    #[error("File '{file}' not found in archive '{archive}'")]
    FileNotFoundInArchive { file: String, archive: String },
    #[error("7-zip extraction failed: '{0}'")]
    ExtractionFailed(String),
}

#[derive(Default)]
pub struct SevenUtil {
    executable: PathBuf,
}

impl SevenUtil {
    pub fn new() -> Result<Self, SevenError> {
        let executable = Self::resolve_sevenz_path()?;
        Ok(Self { executable })
    }

    fn resolve_sevenz_path() -> Result<PathBuf, SevenError> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

        let seven_zip = hklm
            .open_subkey("SOFTWARE\\7-Zip")
            .map_err(SevenError::RegOpenFailed)?;

        let path: String = seven_zip
            .get_value("Path")
            .map_err(SevenError::RegGetValueFailed)?;

        let exe_path: PathBuf = PathBuf::from(path).join("7z.exe");
        if !exe_path.exists() || !exe_path.is_file() {
            return Err(SevenError::SevenZipNotFound(exe_path.display().to_string()));
        }
        Ok(exe_path)
    }

    pub fn inst() -> &'static SevenUtil {
        INST.get_or_init(|| match SevenUtil::new() {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("{}", e);
                utils::wait_for_input();
                std::process::exit(0);
            }
        })
    }

    pub fn extract_specific_file_to(
        &self,
        archive: &PathBuf,
        file_in_archive: &str,
        dst: &PathBuf,
    ) -> Result<(), SevenError> {
        let output = Command::new(&self.executable)
            .arg("e")
            .arg(archive)
            .arg(file_in_archive)
            .arg(format!("-o{}", dst.display().to_string()))
            .arg("-aoa")
            .output()
            .map_err(SevenError::CommandError)?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("No files to process") {
            return Err(SevenError::FileNotFoundInArchive {
                file: file_in_archive.to_string(),
                archive: archive.display().to_string(),
            });
        }

        if !output.status.success() {
            let stderr_msg = String::from_utf8_lossy(&output.stderr);
            return Err(SevenError::ExtractionFailed(stderr_msg.to_string()));
        }

        Ok(())
    }

    pub fn extract_to(&self, archive: &PathBuf, dst: &PathBuf) -> Result<(), SevenError> {
        let output = Command::new(&self.executable)
            .arg("x")
            .arg(archive)
            .arg(format!("-o{}", dst.display().to_string()))
            .arg("-aoa")
            .output()
            .map_err(SevenError::CommandError)?;

        if !output.status.success() {
            let stderr_msg = String::from_utf8_lossy(&output.stderr);
            return Err(SevenError::ExtractionFailed(stderr_msg.to_string()));
        }

        Ok(())
    }
}
