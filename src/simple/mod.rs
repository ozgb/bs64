use self::luts::{E0, E1, E2};

const CHARPAD: u8 = b'=';

use super::CodecError;

mod luts;

#[repr(packed(1))]
struct InputBytes {
    t1: u8,
    t2: u8,
    t3: u8,
}

#[repr(packed(1))]
struct OutputBytes {
    d1: u8,
    d2: u8,
    d3: u8,
    d4: u8,
}

#[inline(always)]
fn encode_any_inner(src: &[InputBytes], dest: &mut [OutputBytes]) -> usize {
    for (dest, src) in dest.iter_mut().zip(src.iter()) {
        let (t1, t2, t3) = (src.t1, src.t2, src.t3);
        dest.d1 = E0[t1 as usize];
        dest.d2 = E1[(((t1 & 0x03) << 4) | ((t2 >> 4) & 0x0F)) as usize];
        dest.d3 = E1[(((t2 & 0x0F) << 2) | ((t3 >> 6) & 0x03)) as usize];
        dest.d4 = E2[t3 as usize];
    }

    dest.len() * 4
}

#[inline(always)]
fn encode_32_inner(src: &[InputBytes], dest: &mut [OutputBytes]) -> usize {
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
pub unsafe fn encode_32(src: &[u8], dest: &mut [u8]) -> usize {
    let src: &[InputBytes] = std::slice::from_raw_parts(src.as_ptr() as *const InputBytes, 8);
    let dest: &mut [OutputBytes] =
        std::slice::from_raw_parts_mut(dest.as_mut_ptr() as *mut OutputBytes, 8);

    encode_32_inner(src, dest)
}

/// Encode any length src and destination slice
///
/// # Safety
/// - No checks are performed on the destination slice length
unsafe fn encode_any(src: &[u8], dest: &mut [u8]) -> usize {
    let src_len = src.len() / 3;
    let src: &[InputBytes] = std::slice::from_raw_parts(src.as_ptr() as *const InputBytes, src_len);
    let dest: &mut [OutputBytes] =
        std::slice::from_raw_parts_mut(dest.as_mut_ptr() as *mut OutputBytes, src_len);

    encode_any_inner(src, dest)
}

pub fn encode(src: &[u8], dest: &mut [u8]) -> usize {
    let len = src.len();
    let mut src_i = 0;
    let mut dest_i = 0;
    loop {
        if len - src_i < 24 {
            break;
        }
        unsafe {
            encode_32(&src[src_i..src_i + 24], &mut dest[dest_i..dest_i + 32]);
        }

        src_i += 24;
        dest_i += 32;
    }

    if len - src_i > 2 {
        unsafe { dest_i += encode_any(&src[src_i..], &mut dest[dest_i..]) }
        src_i = (dest_i / 4) * 3;
    }

    match len - src_i {
        0 => (),
        1 => {
            let t1 = src[src_i];
            dest[dest_i] = E0[t1 as usize];
            dest[dest_i + 1] = E1[((t1 & 0x03) << 4) as usize];
            dest[dest_i + 2] = CHARPAD;
            dest[dest_i + 3] = CHARPAD;
            dest_i += 4;
        }
        _ => {
            let (t1, t2) = (src[src_i], src[src_i + 1]);
            dest[dest_i] = E0[t1 as usize];
            dest[dest_i + 1] = E1[(((t1 & 0x03) << 4) | ((t2 >> 4) & 0x0F)) as usize];
            dest[dest_i + 2] = E1[((t2 & 0x0F) << 2) as usize];
            dest[dest_i + 3] = CHARPAD;
            dest_i += 4;
        }
    }

    dest_i
}
