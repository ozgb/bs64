use crate::{CHARS, codecs::CodecBuf};
use super::{Codec, CodecError};

pub struct Vanilla {
}

impl Default for Vanilla {
    fn default() -> Self {
        Self {}
    }
}

impl Codec for Vanilla {
    fn encode_buf(&self, input: &mut CodecBuf, output: &mut CodecBuf) -> Result<(), CodecError> {
        let chunks = input.chunks(3);
        let chunks_len = chunks.len();
        let mut last_chunk: &[u8] = &[];
        for (i, chunk) in chunks.enumerate() {
            if i == chunks_len - 1 {
                last_chunk = chunk;
                break;
            }
            push_chars(chunk, output, 4)?;
        }

        // Final chunk
        let mut final_chunk = [0u8; 3];
        final_chunk[..last_chunk.len()].copy_from_slice(last_chunk);

        match last_chunk.len() {
            1 => push_chars(&final_chunk, output, 2)?,
            2 => push_chars(&final_chunk, output, 3)?,
            3 => push_chars(&final_chunk, output, 4)?,
            _ => unreachable!()
        }

        match input.len()%3 {
            0 => (),
            1 => {
                output.push(b'=')?;
                output.push(b'=')?;
            },
            2 => output.push(b'=')?,
            _ => unreachable!()
        }

        Ok(())
    }

    fn decode_buf<R: std::io::Read, W: std::io::Write>(&self, _input: &[u8], _output: &mut Vec<u8>) -> Result<(), CodecError> {
        todo!()
    }
}

#[inline(always)]
pub fn push_chars(chunk: &[u8], output: &mut CodecBuf, num_chars: usize) -> Result<(), CodecError> {
    let n: u32 = ((chunk[0] as u32) << 16) + ((chunk[1] as u32) << 8) + chunk[2] as u32;
    let n_split = [(n >> 18) & 0x3f, (n >> 12) & 0x3f, (n >> 6) & 0x3f, n & 0x3f];

    let o0 = CHARS[n_split[0] as usize];
    let o1 = CHARS[n_split[1] as usize];
    let o2 = CHARS[n_split[2] as usize];
    let o3 = CHARS[n_split[3] as usize];

    output.extend_by_slice(&[o0, o1, o2, o3][..num_chars])?;

    Ok(())
}
