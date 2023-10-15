# 🚀 Base 64 

✨ SIMD-accelerated Base64 for Rust ✨

## 🌟 Features:
- 💡 Uses AVX2 instructions for super-fast encoding and decoding
- 🔄 Fallback when AVX2 is unavailable uses any available SIMD

## 🎯 Project goals:
- 🔧 Simple, idiomatic API
- 📦 Sensible defaults
- ⚡ Fast

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


## TODO

- [x] Integration tests
- [x] Benchmarking suite
- [ ] Regression tests + benchmark in Github Actions
- [ ] Change default implementation with feature flags
- [ ] Builders for custom configs at runtime
