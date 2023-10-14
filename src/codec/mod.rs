/// Encoders and decoders for various formats.
use thiserror::Error;

pub mod avx2;
pub mod simplesimd;

/// The error type for encoding and decoding.
#[derive(Error, Debug)]
pub enum CodecError {
    #[error("codec error")]
    CodecError(#[from] std::io::Error),
    #[error("buffer overflow")]
    BufferOverflow,
    #[error("unknown codec error")]
    Unknown,
}
