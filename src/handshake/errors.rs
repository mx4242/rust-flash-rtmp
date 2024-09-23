use thiserror::Error;

// Allow the Nom variant to be large
#[allow(variant_size_differences)]

#[derive(Error, Debug, Clone, Eq, PartialEq)]
pub enum HandshakeError {
    #[error("No data provided")]
    NoData,

    #[error("Version mismatch: {0}")]
    VersionError(u8),

    #[error("Random Echo mismatch")]
    EchoMismatch {
        expected: [u8; 1528],
        got: [u8; 1528],
    },

    #[error("Handshake has already been done")]
    HandshakeAlreadyDone,
}
