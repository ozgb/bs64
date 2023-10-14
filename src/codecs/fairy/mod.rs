mod lut;

use crate::codecs::CodecBuf;
use lut::{C0_LUT, C1_LUT, C2_LUT, C3_LUT};

use super::{Codec, CodecError};

pub struct Fairy {}

impl Default for Fairy {
    fn default() -> Self {
        Self {}
    }
}

impl Codec for Fairy {
    fn encode_buf(&self, input: &mut CodecBuf, output: &mut CodecBuf) -> Result<(), CodecError> {
        let chunks = input.chunks(3);
        let chunks_len = chunks.len();
        let mut last_chunk: &[u8] = &[];
        for (i, chunk) in chunks.enumerate() {
            if i == chunks_len - 1 {
                last_chunk = chunk;
                break;
            }
            push_chars_lut(chunk, output)?;
        }

        // Last chunk
        match last_chunk.len() {
            1 => push_chars_lut_n(&last_chunk, output)?,
            2 => push_chars_lut_n(&last_chunk, output)?,
            3 => push_chars_lut_n(&last_chunk, output)?,
            _ => unreachable!(),
        }

        match input.len() % 3 {
            0 => (),
            1 => {
                output.push(b'=')?;
                output.push(b'=')?;
            }
            2 => output.push(b'=')?,
            _ => unreachable!(),
        }

        Ok(())
    }

    fn decode_buf<R: std::io::Read, W: std::io::Write>(
        &self,
        _input: &[u8],
        _output: &mut Vec<u8>,
    ) -> Result<(), CodecError> {
        todo!()
    }
}

#[inline(always)]
pub fn push_chars_lut(chunk: &[u8], output: &mut CodecBuf) -> Result<(), CodecError> {
    output.push(C0_LUT[chunk[0] as usize])?;
    {
        let i: u16 = ((chunk[1] as u16) << 8) + chunk[0] as u16;
        output.push(C1_LUT[i as usize])?;
    }
    {
        let i: u16 = ((chunk[2] as u16) << 8) + chunk[1] as u16;
        output.push(C2_LUT[i as usize])?;
    }
    output.push(C3_LUT[chunk[2] as usize])?;
    Ok(())
}

#[inline(always)]
pub fn push_chars_lut_n(chunk: &[u8], output: &mut CodecBuf) -> Result<(), CodecError> {
    output.push(C0_LUT[chunk[0] as usize])?;
    {
        let i: u16 = ((*chunk.get(1).unwrap_or(&0) as u16) << 8) + chunk[0] as u16;
        output.push(C1_LUT[i as usize])?;
    }
    if chunk.len() < 2 {
        return Ok(());
    }
    {
        let i: u16 = ((*chunk.get(2).unwrap_or(&0) as u16) << 8) + chunk[1] as u16;
        output.push(C2_LUT[i as usize])?;
    }
    if chunk.len() < 3 {
        return Ok(());
    }
    output.push(C3_LUT[chunk[2] as usize])?;
    Ok(())
}
