# 🚀 Base 64 

[![](https://img.shields.io/crates/v/bs64.svg)](https://crates.io/crates/bs64) [![Docs](https://docs.rs/bs64/badge.svg)](https://docs.rs/bs64)

✨ SIMD-accelerated Base64 for Rust ✨

## 🌟 Features
- 💡 Uses AVX2 instructions for super-fast encoding and decoding
- 🔄 Fallback when AVX2 is unavailable uses any available SIMD

## 🎯 Project goals
- 🔧 Simple, idiomatic API
- 📦 Sensible defaults
- ⚡ Fast

## Installation

```bash
cargo add bs64
```

## Usage

```rust
use bs64;

fn main() {
  // Encode
  let input = vec![2, 3, 4, 5];
  let output: String = bs64::encode(&input);

  // Decode
  let decoded_output = bs64::decode(output.as_bytes());
}
```

## Benchmarks

Ran using 100k inputs, 10000 iterations on an Intel® Core™ i7-1065G7. Comparisons are made against [base64](https://crates.io/crates/base64) and [data-encoding](https://crates.io/crates/data-encoding) crates.
```
cargo run --features "cli" --release -- -b 100000 -i 10000
```

### Encode

| name                  | MB/s
|----------------------|--------
|🚀 **bs64::encode()**    | 4813.70        
|🚀 **bs64::encode_mut()**| 6579.17        
|🚀 **bs64 fallback**         | 944.18         
|data_encoding         | 858.51         
|data_encoding mut     | 873.28         
|base64                | 748.02         
|base64 mut            | 870.99 

## Decode

| name                   | MB/s          |
|------------------------|---------------|
| 🚀 **bs64::decode()**     | 3899.26       |
| 🚀 **bs64::decode_mut()** | 3965.25       |
| 🚀 **bs64 fallback**          | 837.17        |
| data_encoding          | 647.33        |
| data_encoding mut      | 684.01        |
| base64                 | 761.68        |
| base64 mut             | 805.60        |

## Implementation Details

Code was initially ported from https://github.com/lemire/fastbase64

The `simple` fallback implementation is based on the `chromium` implementation from the fastbase64 repo. The use of iterators and chunking the input in the Rust implementation makes it easy for the compiler to vectorise the processing.

The AVX2 implementation is largely untouched compared with the original `fastbase64` implementation.

The code is optimised for x86_64, and therefore assumes large-ish caches are available for storing lookup tables. I created a naive implementation that indexed a static array of valid base64 chars - the performance there was only slightly worse than the chromium LUT implementation, so I may add this as an option for low-memory targets (i.e. embedded).

Useful links:
- https://github.com/lemire/fastbase64
- https://www.nickwilcox.com/blog/autovec/

## TODO

- [x] Integration tests
- [x] Benchmarking suite
- [ ] Comply with MIME, UTF-7, and other Base64 standards
- [ ] Regression tests + benchmark in Github Actions
- [ ] Change default implementation with feature flags
- [ ] Builders for custom configs at runtime
