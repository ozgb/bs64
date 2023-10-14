#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use super::simplesimd;

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
    let out = _mm256_add_epi8(input, _mm256_shuffle_epi8(lut, indices));
    out
}

pub fn encode_with_fallback(dest: &mut [u8], str: &[u8]) -> usize {
    if is_x86_feature_detected!("avx2") {
        unsafe { encode(dest, str) }
    } else {
        simplesimd::encode(str, dest)
    }
}

#[target_feature(enable = "avx2")]
pub unsafe fn encode(dest: &mut [u8], str: &[u8]) -> usize {
    let mut str_offset: isize = 0;
    let mut dest_offset = 0;

    let str_len = str.len() as isize;

    if str.len() >= 32 - 4 {
        let mask_vec = _mm256_set_epi32(
            0x8000000 as i32,
            0x8000000 as i32,
            0x8000000 as i32,
            0x8000000 as i32,
            0x8000000 as i32,
            0x8000000 as i32,
            0x8000000 as i32,
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

    dest_offset += simplesimd::encode(
        &str[str_offset as usize..],
        &mut dest[dest_offset as usize..],
    ) as isize;

    dest_offset as usize
}
