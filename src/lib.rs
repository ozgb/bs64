use thiserror::Error;

use data_encoding::BASE64;

pub mod avx2;
pub mod simple;

/// The error type for encoding and decoding.
#[derive(Error, Debug)]
pub enum CodecError {
    #[error("codec error")]
    CodecError(#[from] std::io::Error),
    #[error("output length {0} is < expected length {1}")]
    OutputLengthTooShort(usize, usize),
    #[error("input length {0} != 0 % 4")]
    InputModError(usize),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("unknown codec error")]
    Unknown,
}

#[derive(Default)]
pub struct EncodeOptions {}

pub fn encode_len(input: &[u8]) -> usize {
    match input.len() % 3 {
        0 => input.len() / 3 * 4,
        _ => input.len() / 3 * 4 + 4,
    }
}

pub fn decode_len(input: &[u8]) -> usize {
    (input.len() / 4) * 3
}

impl EncodeOptions {
    pub fn encode(self, input: &[u8]) -> String {
        let mut output = vec![0u8; encode_len(input)];
        avx2::encode_with_fallback(&mut output, input);
        unsafe { String::from_utf8_unchecked(output) }
    }

    pub fn encode_mut(self, input: &[u8], output: &mut [u8]) -> Result<usize, CodecError> {
        if output.len() < encode_len(input) {
            Err(CodecError::OutputLengthTooShort(
                output.len(),
                encode_len(input),
            ))
        } else {
            Ok(avx2::encode_with_fallback(output, input))
        }
    }
}

#[derive(Default)]
pub struct DecodeOptions {}

impl DecodeOptions {
    pub fn decode(self, input: &[u8]) -> Result<Vec<u8>, CodecError> {
        let mut output = vec![0u8; decode_len(input)];
        avx2::decode_with_fallback(&mut output, &input)?;
        Ok(output)
    }

    pub fn decode_mut(self, input: &[u8], output: &mut [u8]) -> Result<usize, CodecError> {
        if output.len() < decode_len(input) {
            Err(CodecError::OutputLengthTooShort(
                output.len(),
                decode_len(input),
            ))
        } else {
            avx2::decode_with_fallback(output, input)
        }
    }
}

pub fn encode(input: &[u8]) -> String {
    EncodeOptions::default().encode(input)
}

pub fn encode_mut(input: &[u8], output: &mut [u8]) -> Result<usize, CodecError> {
    EncodeOptions::default().encode_mut(input, output)
}

pub fn decode(input: &[u8]) -> Result<Vec<u8>, CodecError> {
    DecodeOptions::default().decode(input)
}

pub fn decode_mut(input: &[u8], output: &mut [u8]) -> Result<usize, CodecError> {
    DecodeOptions::default().decode_mut(input, output)
}
