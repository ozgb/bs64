use crate::CHARS;

use self::luts::{CHARPAD, E0, E1, E2};

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
fn encode_32_inner(src: &[u8], dest: &mut [u8]) -> usize {
    for v in 0..8 {
        let i = v * 3;
        let dest_i = v * 4;
        let (t1, t2, t3) = (src[i], src[i + 1], src[i + 2]);
        dest[dest_i] = E0[t1 as usize];
        dest[dest_i + 1] = E1[(((t1 & 0x03) << 4) | ((t2 >> 4) & 0x0F)) as usize];
        dest[dest_i + 2] = E1[(((t2 & 0x0F) << 2) | ((t3 >> 6) & 0x03)) as usize];
        dest[dest_i + 3] = E2[t3 as usize];
    }

    32
}

#[inline(always)]
fn encode_any_inner(src: &[InputBytes], dest: &mut [OutputBytes]) -> usize {
    for (dest, src) in dest.iter_mut().zip(src.iter()) {
        let n: u32 = ((src.t1 as u32) << 16) + ((src.t2 as u32) << 8) + src.t3 as u32;
        let n_split = [
            (n >> 18) & 0x3f,
            (n >> 12) & 0x3f,
            (n >> 6) & 0x3f,
            n & 0x3f,
        ];

        dest.d1 = CHARS[n_split[0] as usize];
        dest.d2 = CHARS[n_split[1] as usize];
        dest.d3 = CHARS[n_split[2] as usize];
        dest.d4 = CHARS[n_split[3] as usize];
    }

    dest.len() * 4
}

#[inline(always)]
fn encode_32_inner_alt(src: &[InputBytes], dest: &mut [OutputBytes]) -> usize {
    for (dest, src) in dest.iter_mut().zip(src.iter()) {
        let n: u32 = ((src.t1 as u32) << 16) + ((src.t2 as u32) << 8) + src.t3 as u32;
        let n_split = [
            (n >> 18) & 0x3f,
            (n >> 12) & 0x3f,
            (n >> 6) & 0x3f,
            n & 0x3f,
        ];

        dest.d1 = CHARS[n_split[0] as usize];
        dest.d2 = CHARS[n_split[1] as usize];
        dest.d3 = CHARS[n_split[2] as usize];
        dest.d4 = CHARS[n_split[3] as usize];
    }

    32
}

#[target_feature(enable = "avx")]
pub unsafe fn encode_32_avx(src: &[u8], dest: &mut [u8]) -> usize {
    unsafe {
        let src: &[InputBytes] = std::slice::from_raw_parts(src.as_ptr() as *const InputBytes, 8);
        let dest: &mut [OutputBytes] =
            std::slice::from_raw_parts_mut(dest.as_mut_ptr() as *mut OutputBytes, 8);

        encode_32_inner_alt(src, dest)
    }
}

#[target_feature(enable = "avx2")]
pub unsafe fn encode_32_avx2(src: &[u8], dest: &mut [u8]) -> usize {
    unsafe {
        let src: &[InputBytes] = std::slice::from_raw_parts(src.as_ptr() as *const InputBytes, 8);
        let dest: &mut [OutputBytes] =
            std::slice::from_raw_parts_mut(dest.as_mut_ptr() as *mut OutputBytes, 8);

        encode_32_inner_alt(src, dest)
    }
}

/// Encode 24 bytes from src slice to 32 destination slice
/// Unsafe because no checks are performed on the slice lengths
pub unsafe fn encode_32(src: &[u8], dest: &mut [u8]) -> usize {
    let src: &[InputBytes] = std::slice::from_raw_parts(src.as_ptr() as *const InputBytes, 8);
    let dest: &mut [OutputBytes] =
        std::slice::from_raw_parts_mut(dest.as_mut_ptr() as *mut OutputBytes, 8);

    encode_32_inner_alt(src, dest)
}

/// Encode any length src and destination slice
/// Unsafe because no checks are performed on the destination slice length
unsafe fn encode_any(src: &[u8], dest: &mut [u8]) -> usize {
    let src_len = src.len() / 3;
    let src: &[InputBytes] = std::slice::from_raw_parts(src.as_ptr() as *const InputBytes, src_len);
    let dest: &mut [OutputBytes] =
        std::slice::from_raw_parts_mut(dest.as_mut_ptr() as *mut OutputBytes, src_len);

    encode_any_inner(src, dest)
}

pub fn encode_alt(src: &[u8], dest: &mut [u8]) -> usize {
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
            let t1 = src[src_i as usize];
            let n: u32 = (t1 as u32) << 16;
            let n_split = [
                ((n >> 18) & 0x3f) as usize,
                ((n >> 12) & 0x3f) as usize,
                ((n >> 6) & 0x3f) as usize,
                (n & 0x3f) as usize,
            ];

            dest[dest_i] = CHARS[n_split[0]];
            dest[dest_i + 1] = CHARS[n_split[1]];
            dest[dest_i + 2] = CHARPAD;
            dest[dest_i + 3] = CHARPAD;
            dest_i += 4;
        }
        _ => {
            let (t1, t2) = (src[src_i], src[src_i + 1]);
            let n: u32 = ((t1 as u32) << 16) + ((t2 as u32) << 8);
            let n_split = [
                ((n >> 18) & 0x3f) as usize,
                ((n >> 12) & 0x3f) as usize,
                ((n >> 6) & 0x3f) as usize,
                (n & 0x3f) as usize,
            ];

            dest[dest_i] = CHARS[n_split[0]];
            dest[dest_i + 1] = CHARS[n_split[1]];
            dest[dest_i + 2] = CHARS[n_split[2]];
            dest[dest_i + 3] = CHARPAD;
            dest_i += 4;
        }
    }

    dest_i
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

    let mut t1: u8;
    let mut t2: u8;
    let mut t3: u8;

    if len - src_i > 2 {
        for i in (src_i..len - 2).step_by(3) {
            (t1, t2, t3) = (src[src_i], src[src_i + 1], src[src_i + 2]);
            dest[dest_i] = E0[t1 as usize];
            dest[dest_i + 1] = E1[(((t1 & 0x03) << 4) | ((t2 >> 4) & 0x0F)) as usize];
            dest[dest_i + 2] = E1[(((t2 & 0x0F) << 2) | ((t3 >> 6) & 0x03)) as usize];
            dest[dest_i + 3] = E2[t3 as usize];
            dest_i += 4;
            src_i = i + 3;
        }
    }

    match len - src_i {
        0 => (),
        1 => {
            t1 = src[src_i as usize];
            dest[dest_i] = E0[t1 as usize];
            dest[dest_i + 1] = E1[((t1 & 0x03) << 4) as usize];
            dest[dest_i + 2] = CHARPAD;
            dest[dest_i + 3] = CHARPAD;
            dest_i += 4;
        }
        _ => {
            (t1, t2) = (src[src_i], src[src_i + 1]);
            dest[dest_i] = E0[t1 as usize];
            dest[dest_i + 1] = E1[(((t1 & 0x03) << 4) | ((t2 >> 4) & 0x0F)) as usize];
            dest[dest_i + 2] = E2[((t2 & 0x0F) << 2) as usize];
            dest[dest_i + 3] = CHARPAD;
            dest_i += 4;
        }
    }

    dest[dest_i] = b'\0';
    dest_i + 1
}
