#![allow(unused_variables)]

mod ueplugingen; 

pub use ueplugingen::*;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    VarError(std::env::VarError),
    AskamaError(askama::Error),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
impl From<std::io::Error> for Error { fn from(value: std::io::Error) -> Self { Self::IoError(value) } }
impl From<std::env::VarError> for Error { fn from(value: std::env::VarError) -> Self { Self::VarError(value) } }
impl From<askama::Error> for Error { fn from(value: askama::Error) -> Self { Self::AskamaError(value) } }
