use thiserror::Error;

use crate::{deletefiles, hdiffmap};

#[derive(Error, Debug)]
pub enum Error {
    #[error[transparent]]
    DeleteFileError(#[from] deletefiles::DeleteFileError),
    #[error[transparent]]
    PatchError(#[from] hdiffmap::PatchError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("StarRail.exe not found in the current directory: {0}\nTip: Pass the game path as the first argument if it's not in the current directory or move this .exe")]
    Path(String),
}
