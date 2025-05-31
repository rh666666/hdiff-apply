// I didnt find any good 7z crates so this will have to do for now

use std::{path::PathBuf, process::Command};

use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use crate::Error;

pub struct SevenUtil {
    executable: PathBuf,
}

impl SevenUtil {
    pub fn new() -> Result<Self, Error> {
        let executable = Self::resolve_sevenz_path()?;
        Ok(Self { executable })
    }

    fn resolve_sevenz_path() -> Result<PathBuf, Error> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let seven_zip = hklm.open_subkey("SOFTWARE\\7-Zip")?;
        let path: String = seven_zip.get_value("Path")?;

        let exe_path: PathBuf = PathBuf::from(path).join("7z.exe");
        Ok(exe_path)
    }

    pub fn extract_specific_file_to(
        &self,
        archive: &PathBuf,
        file_in_archive: &str,
        dst: &PathBuf,
    ) -> Result<(), Error> {
        let output = Command::new(&self.executable)
            .arg("e")
            .arg(archive)
            .arg(file_in_archive)
            .arg(format!("-o{}", dst.display().to_string()))
            .arg("-aoa")
            .output()?;

        if !output.status.success() {
            let stderr_msg = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Extraction(stderr_msg.to_string()));
        }

        Ok(())
    }

     pub fn extract_to(
        &self,
        archive: &PathBuf,
        dst: &PathBuf,
    ) -> Result<(), Error> {
        let output = Command::new(&self.executable)
            .arg("x")
            .arg(archive)
            .arg(format!("-o{}", dst.display().to_string()))
            .arg("-aoa")
            .output()?;

        if !output.status.success() {
            let stderr_msg = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Extraction(stderr_msg.to_string()));
        }

        Ok(())
    }
}
