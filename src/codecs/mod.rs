/// Encoders and decoders for various formats.

use std::io::{Read, Write};
use thiserror::Error;

pub mod fairy;
pub mod vanilla;
pub mod sponge;

/// BUF_LEN must be divisible by 3.
const BUF_LEN: usize = 1024;

pub type Default = vanilla::Vanilla;

pub struct CodecBuf {
    buf: [u8; BUF_LEN],
    buf_pos: usize,
}

impl CodecBuf {
    pub fn new() -> Self {
        Self {
            buf: [0u8; BUF_LEN],
            buf_pos: 0,
        }
    }

    pub fn push(&mut self, val: u8) -> Result<(), CodecError> {
        if self.buf_pos >= BUF_LEN {
            return Err(CodecError::BufferOverflow);
        }
        self.buf[self.buf_pos] = val;
        self.buf_pos += 1;
        Ok(())
    }

    pub fn extend_by_slice(&mut self, slice: &[u8]) -> Result<(), CodecError> {
        if self.buf_pos + slice.len() >= BUF_LEN {
            return Err(CodecError::BufferOverflow);
        }
        self.buf[self.buf_pos..self.buf_pos + slice.len()].copy_from_slice(slice);
        self.buf_pos += slice.len();
        Ok(())
    }

    pub fn chunks(&self, chunk_size: usize) -> std::slice::Chunks<'_, u8> {
        self.buf[..self.buf_pos].chunks(chunk_size)
    }

    pub fn copy_from_slice(&mut self, slice: &[u8]) {
        self.buf[..slice.len()].copy_from_slice(slice);
        self.buf_pos = slice.len();
    }

    pub fn read_from<R: std::io::Read>(&mut self, input: &mut R, n: usize) -> Result<usize, CodecError> {
        let bytes_read = input.read(&mut self.buf[..n])?;
        self.buf_pos = bytes_read;
        Ok(bytes_read)
    }

    pub fn clear(&mut self) {
        self.buf_pos = 0;
    }

    pub fn len(&self) -> usize {
        self.buf_pos
    }
}

impl AsRef<[u8]> for CodecBuf {
    fn as_ref(&self) -> &[u8] {
        &self.buf[..self.buf_pos]
    }
}

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

/// Trait for encoding and decoding data.
pub trait Codec {

    /// Encodes the given input.
    fn encode(&self, input: &[u8]) -> String {
        let mut output = Vec::with_capacity(input.len() * 4 / 3);

        // Create an input stream
        let mut input_stream = std::io::Cursor::new(input);
        match self.encode_stream(&mut input_stream, &mut output) {
            Ok(_) => (),
            Err(e) => {println!("err: {:?}", e); panic!(":("); }
        }

        String::from_utf8(output).unwrap()
    }

    /// Encodes the given input.
    fn encode_stream<R: Read, W: Write>(&self, input: &mut R, output: &mut W) -> Result<(), CodecError> {
        let mut input_buf = CodecBuf::new();
        let mut output_buf = CodecBuf::new();

        loop {
            let bytes_read = input_buf.read_from(input, (BUF_LEN / 4) * 3)?;
            if bytes_read == 0 {
                break;
            }
            self.encode_buf(&mut input_buf, &mut output_buf)?;
            let buf_slice = output_buf.as_ref();
            output.write_all(buf_slice)?;
            output_buf.clear();
        }

        Ok(())
    }

    /// Encodes the given input.
    fn encode_buf(&self, input: &mut CodecBuf, output: &mut CodecBuf) -> Result<(), CodecError>;

    /// Decodes the given input.
    fn decode_stream<R: Read, W: Write>(&self, _input: R, _output: W) -> Result<(), CodecError> {
        todo!()
    }
    
    /// Decodes the given input.
    fn decode_buf<R: Read, W: Write>(&self, input: &[u8], output: &mut Vec<u8>) -> Result<(), CodecError>;
}
