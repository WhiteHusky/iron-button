use std::process::{ExitCode, Termination};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("XDG related error: {0}")]
    XDGError(#[from] ashpd::Error),
    #[error("Could not parse config: {0}")]
    ConfigParseError(serde_yaml_ng::Error),
    #[error("Could not read config: {0}")]
    ConfigReadError(std::io::Error),
    #[error("Unexpectedly stopped receiving keys")]
    UnexpectedEndOfKeys,
}

impl Termination for Error {
    fn report(self) -> std::process::ExitCode {
        eprintln!("{self}");
        ExitCode::FAILURE
    }
}
