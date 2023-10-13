use codecs::Codec;
use data_encoding::BASE64;

pub mod codecs;

const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub struct EncodeOptions {

}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self {  }
    }
}

impl EncodeOptions {
    pub fn encode(self, input: &[u8]) -> String {
        codecs::vanilla::Vanilla::default().encode(input)
    }
}

pub struct DecodeOptions {

}

impl Default for DecodeOptions {
    fn default() -> Self {
        Self {  }
    }
}

impl DecodeOptions {
    pub fn decode(self, input: &[u8]) -> Result<Vec<u8>, data_encoding::DecodeError> {
        BASE64.decode(input)
    }
}

pub fn encode(input: &[u8]) -> String {
    EncodeOptions::default().encode(input)
}

pub fn decode(input: &[u8]) -> Result<Vec<u8>, data_encoding::DecodeError> {
    DecodeOptions::default().decode(input)
}
