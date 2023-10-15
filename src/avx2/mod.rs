#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::CodecError;

use super::simple;

// Rust implementation of AVX2 fastbase64:
// https://github.com/lemire/fastbase64/blob/master/src/fastavxbase64.c

// LICENSE
//
// Copyright (c) 2015-2016, Wojciech Muła, Alfred Klomp,  Daniel Lemire
// (Unless otherwise stated in the source code)
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in the
//    documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
// IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED
// TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A
// PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED
// TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

unsafe fn enc_reshuffle(input: __m256i) -> __m256i {
    // translation from SSE into AVX2 of procedure
    // https://github.com/WojciechMula/base64simd/blob/master/encode/unpack_bigendian.cpp
    let input: __m256i = _mm256_shuffle_epi8(
        input,
        _mm256_set_epi8(
            10, 11, 9, 10, 7, 8, 6, 7, 4, 5, 3, 4, 1, 2, 0, 1, 14, 15, 13, 14, 11, 12, 10, 11, 8,
            9, 7, 8, 5, 6, 4, 5,
        ),
    );

    let t0: __m256i = _mm256_and_si256(input, _mm256_set1_epi32(0x0fc0fc00));
    let t1: __m256i = _mm256_mulhi_epu16(t0, _mm256_set1_epi32(0x04000040));

    let t2 = _mm256_and_si256(input, _mm256_set1_epi32(0x003f03f0));
    let t3 = _mm256_mullo_epi16(t2, _mm256_set1_epi32(0x01000010));

    _mm256_or_si256(t1, t3)
}

unsafe fn enc_translate(input: __m256i) -> __m256i {
    let lut: __m256i = _mm256_setr_epi8(
        65, 71, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -19, -16, 0, 0, 65, 71, -4, -4, -4, -4, -4,
        -4, -4, -4, -4, -4, -19, -16, 0, 0,
    );
    let mut indices = _mm256_subs_epu8(input, _mm256_set1_epi8(51));
    let mask = _mm256_cmpgt_epi8(input, _mm256_set1_epi8(25));
    indices = _mm256_sub_epi8(indices, mask);

    _mm256_add_epi8(input, _mm256_shuffle_epi8(lut, indices))
}

pub fn encode_with_fallback(dest: &mut [u8], str: &[u8]) -> usize {
    if is_x86_feature_detected!("avx2") {
        unsafe { encode(dest, str) }
    } else {
        simple::encode(str, dest)
    }
}

/// Encode a slice of bytes into base64 using avx2 instructions
///
/// # Safety
/// - Must only be executed on avx2 enabled cpus
#[target_feature(enable = "avx2")]
pub unsafe fn encode(dest: &mut [u8], str: &[u8]) -> usize {
    let mut str_offset: isize = 0;
    let mut dest_offset = 0;

    let str_len = str.len() as isize;

    if str.len() >= 32 - 4 {
        // i32::MIN means top bit is set
        let mask_vec = _mm256_set_epi32(
            i32::MIN,
            i32::MIN,
            i32::MIN,
            i32::MIN,
            i32::MIN,
            i32::MIN,
            i32::MIN,
            0, // we do not load the first 4 bytes
        );

        let mut inputvector: __m256i =
            _mm256_maskload_epi32(str.as_ptr().offset(-4) as *const i32, mask_vec);

        loop {
            inputvector = enc_reshuffle(inputvector);
            inputvector = enc_translate(inputvector);
            _mm256_storeu_si256(
                dest.as_ptr().offset(dest_offset) as *mut __m256i,
                inputvector,
            );
            str_offset += 24;
            dest_offset += 32;
            let remaining_len = str_len - str_offset;
            if remaining_len < 32 {
                break;
            }
            // no need for a mask here
            inputvector = _mm256_loadu_si256(str.as_ptr().offset(str_offset - 4) as *mut __m256i);
        }
    }

    if str_len - str_offset == 0 {
        return dest_offset as usize;
    }

    dest_offset += simple::encode(
        &str[str_offset as usize..],
        &mut dest[dest_offset as usize..],
    ) as isize;

    dest_offset as usize
}

unsafe fn dec_reshuffle(input: __m256i) -> __m256i {
    // inlined procedure pack_madd from https://github.com/WojciechMula/base64simd/blob/master/decode/pack.avx2.cpp
    // The only difference is that elements are reversed,
    // only the multiplication constants were changed.

    let merge_ab_and_bc: __m256i = _mm256_maddubs_epi16(input, _mm256_set1_epi32(0x01400140)); //_mm256_maddubs_epi16 is likely expensive
    let out: __m256i = _mm256_madd_epi16(merge_ab_and_bc, _mm256_set1_epi32(0x00011000));
    // end of inlined

    // Pack bytes together within 32-bit words, discarding words 3 and 7:
    let out = _mm256_shuffle_epi8(
        out,
        _mm256_setr_epi8(
            2, 1, 0, 6, 5, 4, 10, 9, 8, 14, 13, 12, -1, -1, -1, -1, 2, 1, 0, 6, 5, 4, 10, 9, 8, 14,
            13, 12, -1, -1, -1, -1,
        ),
    );
    // the call to _mm256_permutevar8x32_epi32 could be replaced by a call to _mm256_storeu2_m128i but it is doubtful that it would help
    _mm256_permutevar8x32_epi32(out, _mm256_setr_epi32(0, 1, 2, 4, 5, 6, -1, -1))
}

pub fn decode_with_fallback(dest: &mut [u8], str: &[u8]) -> Result<usize, CodecError> {
    if is_x86_feature_detected!("avx2") {
        unsafe { decode(dest, str) }
    } else {
        simple::decode(str, dest)
    }
}

/// Decode a slice of bytes into base64 using avx2 instructions
///
/// # Safety
/// - Must only be executed on avx2 enabled cpus
#[target_feature(enable = "avx2")]
pub unsafe fn decode(out: &mut [u8], src: &[u8]) -> Result<usize, CodecError> {
    let mut src_i: isize = 0;
    let mut dest_i: isize = 0;

    while src.len() - src_i as usize >= 45 {
        // The input consists of six character sets in the Base64 alphabet,
        // which we need to map back to the 6-bit values they represent.
        // There are three ranges, two singles, and then there's the rest.
        //
        //  #  From       To        Add  Characters
        //  1  [43]       [62]      +19  +
        //  2  [47]       [63]      +16  /
        //  3  [48..57]   [52..61]   +4  0..9
        //  4  [65..90]   [0..25]   -65  A..Z
        //  5  [97..122]  [26..51]  -71  a..z
        // (6) Everything else => invalid input

        let str = _mm256_loadu_si256(src.as_ptr().offset(src_i) as *const __m256i);

        // code by @aqrit from
        // https://github.com/WojciechMula/base64simd/issues/3#issuecomment-271137490
        // transated into AVX2
        let lut_lo: __m256i = _mm256_setr_epi8(
            0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x13, 0x1A, 0x1B, 0x1B,
            0x1B, 0x1A, 0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x13, 0x1A,
            0x1B, 0x1B, 0x1B, 0x1A,
        );
        let lut_hi: __m256i = _mm256_setr_epi8(
            0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
            0x10, 0x10, 0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10,
            0x10, 0x10, 0x10, 0x10,
        );
        let lut_roll: __m256i = _mm256_setr_epi8(
            0, 16, 19, 4, -65, -65, -71, -71, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 19, 4, -65, -65, -71,
            -71, 0, 0, 0, 0, 0, 0, 0, 0,
        );

        let mask_2f: __m256i = _mm256_set1_epi8(0x2f);

        // lookup
        let hi_nibbles: __m256i = _mm256_srli_epi32(str, 4);
        let lo_nibbles: __m256i = _mm256_and_si256(str, mask_2f);

        let lo: __m256i = _mm256_shuffle_epi8(lut_lo, lo_nibbles);
        let eq_2f: __m256i = _mm256_cmpeq_epi8(str, mask_2f);

        let hi_nibbles = _mm256_and_si256(hi_nibbles, mask_2f);
        let hi: __m256i = _mm256_shuffle_epi8(lut_hi, hi_nibbles);
        let roll: __m256i = _mm256_shuffle_epi8(lut_roll, _mm256_add_epi8(eq_2f, hi_nibbles));

        if _mm256_testz_si256(lo, hi) == 0 {
            break;
        }

        let str = _mm256_add_epi8(str, roll);
        // end of copied function

        src_i += 32;

        // end of inlined function

        // Reshuffle the input to packed 12-byte output format:
        let str = dec_reshuffle(str);
        _mm256_storeu_si256(out.as_mut_ptr().offset(dest_i) as *mut __m256i, str);
        dest_i += 24;
    }

    let end_decode_len = simple::decode(&src[src_i as usize..], &mut out[dest_i as usize..])?;
    Ok(dest_i as usize + end_decode_len)
}
