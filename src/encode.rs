use super::CHARS;

#[inline(always)]
pub fn push_chars(chunk: &[u8], output: &mut Vec<u8>, num_chars: usize) {
    let n: u32 = ((chunk[0] as u32) << 16) + ((chunk[1] as u32) << 8) + chunk[2] as u32;
    let n_split = [(n >> 18) & 0x3f, (n >> 12) & 0x3f, (n >> 6) & 0x3f, n & 0x3f];
    for i in 0..num_chars {
        output.push(CHARS[n_split[i] as usize]);
    }
}

pub fn encode(input: &[u8]) -> String {
    let chunks = input.chunks(3);
    let chunks_len = chunks.len();
    let mut output = Vec::with_capacity(chunks_len * 4);
    for chunk in chunks.take(chunks_len - 1) {
        push_chars(chunk, &mut output, 4);
    }

    // Final chunk
    let last_chunk = if input.len() % 3 == 0 {
        &input[(input.len() - 3)..]
    } else {
        &input[(input.len() / 3) * 3..]
    };
    let mut final_chunk = [0u8; 3];
    final_chunk[..last_chunk.len()].copy_from_slice(last_chunk);

    match last_chunk.len() {
        1 => push_chars(&final_chunk, &mut output, 2),
        2 => push_chars(&final_chunk, &mut output, 3),
        3 => push_chars(&final_chunk, &mut output, 4),
        _ => unreachable!()
    }

    match input.len()%3 {
        0 => (),
        1 => {
            output.push(b'=');
            output.push(b'=');
        },
        2 => output.push(b'='),
        _ => unreachable!()
    }

    String::from_utf8(output).unwrap()
}

#[cfg(test)]
mod tests {
    use super::encode;


    #[test]
    fn simple() {
        let output = encode(b"hello\n");
        assert_eq!(output, "aGVsbG8K");

        let output = encode(b"helllo\n");
        assert_eq!(output, "aGVsbGxvCg==");
    }

}
