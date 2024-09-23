use nom::error::{ErrorKind, FromExternalError, ParseError};
use thiserror::Error;


// Allow the Nom variant to be large
#[allow(variant_size_differences)]

/// Enum for representing decoding errors
#[derive(Error, Debug, Clone, Eq, PartialEq)]
pub enum Error<'a> {
    /// A nom internal error
    #[error("Nom internal error")]
    Nom(&'a [u8], ErrorKind),

    /// An unknown IO error occured
    #[error("IO error: {0}")]
    IoError(String, std::io::ErrorKind),
}

impl<'a> ParseError<&'a [u8]> for Error<'a> {
    fn from_error_kind(input: &'a [u8], kind: ErrorKind) -> Self {
        Error::Nom(input, kind)
    }

    fn append(_: &[u8], _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a, E> FromExternalError<&'a [u8], E> for Error<'a> {
    fn from_external_error(input: &'a [u8], kind: ErrorKind, _e: E) -> Self {
        Error::Nom(input, kind)
    }
}