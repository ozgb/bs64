use codec::{avx2, CodecError};
use data_encoding::BASE64;

pub mod codec;

const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

#[derive(Default)]
pub struct EncodeOptions {}

pub fn encode_len(input: &[u8]) -> usize {
    match input.len() % 3 {
        0 => input.len() / 3 * 4,
        _ => input.len() / 3 * 4 + 4,
    }
}

impl EncodeOptions {
    pub fn encode(self, input: &[u8]) -> String {
        let mut output = vec![0u8; encode_len(input)];
        avx2::encode_with_fallback(&mut output, input);
        unsafe { String::from_utf8_unchecked(output) }
    }

    pub fn encode_mut(self, input: &[u8], output: &mut [u8]) -> Result<usize, CodecError> {
        if output.len() < encode_len(input) {
            Err(CodecError::BufferOverflow)
        } else {
            Ok(avx2::encode_with_fallback(output, input))
        }
    }
}

#[derive(Default)]
pub struct DecodeOptions {}

impl DecodeOptions {
    pub fn decode(self, input: &[u8]) -> Result<Vec<u8>, data_encoding::DecodeError> {
        BASE64.decode(input)
    }
}

pub fn encode(input: &[u8]) -> String {
    EncodeOptions::default().encode(input)
}

pub fn encode_mut(input: &[u8], output: &mut [u8]) -> Result<usize, CodecError> {
    EncodeOptions::default().encode_mut(input, output)
}

pub fn decode(input: &[u8]) -> Result<Vec<u8>, data_encoding::DecodeError> {
    DecodeOptions::default().decode(input)
}
