use data_encoding::BASE64;
use super::Codec;

pub struct Sponge {
}

impl Default for Sponge {
    fn default() -> Self {
        Self {}
    }
}

impl Codec for Sponge {
    fn encode_buf(&self, input: &mut super::CodecBuf, output: &mut super::CodecBuf) -> Result<(), super::CodecError> {
        let encode_len = BASE64.encode_len(input.len());
        for _ in 0..encode_len {
            output.push(0)?;
        }
        BASE64.encode_mut(input.as_ref(), &mut output.buf[..encode_len]);
        output.buf_pos = output.buf.len();
        Ok(())
    }

    fn decode_buf<R: std::io::Read, W: std::io::Write>(&self, input: &[u8], output: &mut Vec<u8>) -> Result<(), super::CodecError> {
        let decode_len = BASE64.decode_len(input.len()).unwrap();
        for _ in 0..decode_len {
            output.push(0);
        }
        BASE64.decode_mut(input.as_ref(), &mut output.as_mut_slice()).unwrap();
        Ok(())
    }
}
