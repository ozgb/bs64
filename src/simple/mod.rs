use self::luts::{BADCHAR, D0, D1, D2, D3, E0, E1, E2};

// Inner loop code ported from here:
// https://github.com/lemire/fastbase64/blob/master/src/chromiumbase64.c
// Iterators used to guide the compiler to vectorize the loop + produce simd instructions

use super::CodecError;

mod luts;

#[repr(packed(1))]
struct Bytes {
    t1: u8,
    t2: u8,
    t3: u8,
}

#[repr(packed(1))]
struct Chars {
    d1: u8,
    d2: u8,
    d3: u8,
    d4: u8,
}

fn encode_any_inner(src: &[Bytes], dest: &mut [Chars]) -> usize {
    // We use a iterator loop here to avoid bounds checks
    // This allows the compiler to vectorize the loop
    // and generate SIMD instructions
    // See this blog post for more details:
    // https://www.nickwilcox.com/blog/autovec/
    for (dest, src) in dest.iter_mut().zip(src.iter()) {
        let (t1, t2, t3) = (src.t1, src.t2, src.t3);
        dest.d1 = E0[t1 as usize];
        dest.d2 = E1[(((t1 & 0x03) << 4) | ((t2 >> 4) & 0x0F)) as usize];
        dest.d3 = E1[(((t2 & 0x0F) << 2) | ((t3 >> 6) & 0x03)) as usize];
        dest.d4 = E2[t3 as usize];
    }

    dest.len() * 4
}

fn encode_32_inner(src: &[Bytes], dest: &mut [Chars]) -> usize {
    for (dest, src) in dest.iter_mut().zip(src.iter()) {
        let (t1, t2, t3) = (src.t1, src.t2, src.t3);
        dest.d1 = E0[t1 as usize];
        dest.d2 = E1[(((t1 & 0x03) << 4) | ((t2 >> 4) & 0x0F)) as usize];
        dest.d3 = E1[(((t2 & 0x0F) << 2) | ((t3 >> 6) & 0x03)) as usize];
        dest.d4 = E2[t3 as usize];
    }

    32
}

/// Encode 24 bytes from src slice to 32 destination slice
///
/// # Safety
/// - No checks are performed on the slice lengths
unsafe fn encode_32(src: &[u8], dest: &mut [u8]) -> usize {
    let src = std::slice::from_raw_parts(src.as_ptr() as *const Bytes, 8);
    let dest = std::slice::from_raw_parts_mut(dest.as_mut_ptr() as *mut Chars, 8);

    encode_32_inner(src, dest)
}

/// Encode any length src and destination slice
///
/// # Safety
/// - No checks are performed on the destination slice length
unsafe fn encode_any(src: &[u8], dest: &mut [u8]) -> usize {
    let src_len = src.len() / 3;
    let src = std::slice::from_raw_parts(src.as_ptr() as *const Bytes, src_len);
    let dest = std::slice::from_raw_parts_mut(dest.as_mut_ptr() as *mut Chars, src_len);

    encode_any_inner(src, dest)
}

/// Encode src slice to dest slice
/// Returns the number of bytes written to dest
/// Panics if dest is not large enough
pub fn encode(src: &[u8], dest: &mut [u8]) -> usize {
    let src_iter = src.chunks(24);
    let dest_iter = dest.chunks_mut(32);
    let data_iter = src_iter.zip(dest_iter);

    let mut src_i = 0;
    let mut dest_i = 0;
    let mut num_chunks = 0;
    for (src, dest) in data_iter {
        // Final chunk
        if src.len() < 24 {
            if src.len() > 2 {
                unsafe { dest_i += encode_any(src, dest) }
                src_i = (dest_i / 4) * 3;
            }

            match src.len() - src_i {
                0 => (),
                1 => {
                    let t1 = src[src_i];
                    dest[dest_i] = E0[t1 as usize];
                    dest[dest_i + 1] = E1[((t1 & 0x03) << 4) as usize];
                    dest[dest_i + 2] = b'=';
                    dest[dest_i + 3] = b'=';
                    dest_i += 4;
                }
                _ => {
                    let (t1, t2) = (src[src_i], src[src_i + 1]);
                    dest[dest_i] = E0[t1 as usize];
                    dest[dest_i + 1] = E1[(((t1 & 0x03) << 4) | ((t2 >> 4) & 0x0F)) as usize];
                    dest[dest_i + 2] = E1[((t2 & 0x0F) << 2) as usize];
                    dest[dest_i + 3] = b'=';
                    dest_i += 4;
                }
            }
            break;
        }

        unsafe {
            encode_32(src, dest);
        }
        num_chunks += 1;
    }

    dest_i + (num_chunks * 32)
}

fn decode_any_inner(src: &[Chars], dest: &mut [Bytes]) -> Result<usize, CodecError> {
    for (dst, src) in dest.iter_mut().zip(src.iter()) {
        let x =
            D0[src.d1 as usize] | D1[src.d2 as usize] | D2[src.d3 as usize] | D3[src.d4 as usize];

        let x0: *const u8 = &x as *const u32 as *const u8;
        let x1 = unsafe { x0.offset(1) };
        let x2 = unsafe { x0.offset(2) };

        if x >= BADCHAR {
            let str = format!("{} {} {} {}", src.d1, src.d2, src.d3, src.d4);
            return Err(CodecError::InvalidInput(str));
        }
        unsafe {
            dst.t1 = *x0;
            dst.t2 = *x1;
            dst.t3 = *x2;
        }
    }

    Ok(dest.len() * 3)
}

unsafe fn decode_any(src: &[u8], dest: &mut [u8]) -> Result<usize, CodecError> {
    let src_len = src.len() / 4;
    let src = std::slice::from_raw_parts(src.as_ptr() as *const Chars, src_len);
    let dest = std::slice::from_raw_parts_mut(dest.as_mut_ptr() as *mut Bytes, src_len);

    decode_any_inner(src, dest)
}

fn decode_32_inner(src: &[Chars], dest: &mut [Bytes]) -> Result<usize, CodecError> {
    for (dst, src) in dest.iter_mut().zip(src.iter()) {
        let x: u32 =
            D0[src.d1 as usize] | D1[src.d2 as usize] | D2[src.d3 as usize] | D3[src.d4 as usize];

        let x0: *const u8 = &x as *const u32 as *const u8;
        let x1 = unsafe { x0.offset(1) };
        let x2 = unsafe { x0.offset(2) };

        if x >= BADCHAR {
            let str = format!("{} {} {} {}", src.d1, src.d2, src.d3, src.d4);
            return Err(CodecError::InvalidInput(str));
        }
        unsafe {
            dst.t1 = *x0;
            dst.t2 = *x1;
            dst.t3 = *x2;
        }
    }

    Ok(24)
}

/// Decodes 32 bytes from src slice to 24 byte destination slice
///
/// # Safety
/// - No checks are performed on the slice lengths
unsafe fn decode_32(src: &[u8], dest: &mut [u8]) -> Result<usize, CodecError> {
    let src = std::slice::from_raw_parts(src.as_ptr() as *const Chars, 8);
    let dest = std::slice::from_raw_parts_mut(dest.as_mut_ptr() as *mut Bytes, 8);

    decode_32_inner(src, dest)
}

/// Decode src slice to dest slice
/// Returns the number of bytes written to dest, or an error if input is invalid
pub fn decode(src: &[u8], dest: &mut [u8]) -> Result<usize, CodecError> {
    if src.len() == 0 {
        return Ok(0);
    }

    /*
     * if padding is used, then the message must be at least
     * 4 chars and be a multiple of 4
     */
    if src.len() < 4 || (src.len() % 4 != 0) {
        return Err(CodecError::InputModError(src.len())); /* error */
    }

    /* there can be at most 2 pad chars at the end */
    let src = match src {
        [rest @ .., b'=', b'='] => rest,
        [rest @ .., b'='] => rest,
        _ => src,
    };

    let len = src.len();
    let mut src_i = 0;
    let mut dest_i = 0;
    loop {
        // Use -4 to allow for padding bytes
        if len - 4 - src_i < 32 {
            break;
        }
        unsafe {
            decode_32(&src[src_i..src_i + 32], &mut dest[dest_i..dest_i + 24])?;
        }

        src_i += 32;
        dest_i += 24;
    }

    let leftover = len % 4;
    let len_nopad = match leftover {
        0 => len,
        _ => (len / 4) * 4,
    };

    if len_nopad.saturating_sub(src_i) > 0 {
        unsafe { dest_i += decode_any(&src[src_i..len_nopad], &mut dest[dest_i..])? }
        src_i = len_nopad;
    }

    match leftover {
        0 => (),
        2 => {
            let x = D0[src[src_i] as usize] | D1[src[src_i + 1] as usize];
            dest[dest_i] = x as u8; // i.e. second char
            dest_i += 1;
        }
        3 => {
            let y = &src[src_i..src_i + 3];
            let x: u32 = D0[y[0] as usize] | D1[y[1] as usize] | D2[y[2] as usize]; /* 0x3c */

            let x0: *const u8 = &x as *const u32 as *const u8;
            let x1 = unsafe { x0.offset(1) };

            unsafe {
                dest[dest_i] = *x0;
                dest[dest_i + 1] = *x1;
            }
            dest_i += 2;
        }
        _ => unreachable!(),
    }

    Ok(dest_i)
}
