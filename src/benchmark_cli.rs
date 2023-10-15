#![cfg(feature = "cli")]

use base64::{engine::general_purpose, Engine as _};
use clap::Parser;
use data_encoding::BASE64;
use rand::prelude::*;

use std::time::{Duration, Instant};

/// Benchmark CLI for bs64
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of bytes for input
    #[arg(short, long, default_value_t = 100)]
    bytes: usize,

    /// Number of iterations
    #[arg(short, long, default_value_t = 100)]
    iterations: usize,
}

fn eval() {
    for num_bytes in 100..104 {
        let mut output = vec![0u8; bs64::encode_len(num_bytes)];
        let mut bytes = Vec::with_capacity(num_bytes);
        for i in 0..num_bytes {
            bytes.push(i as u8);
        }
        bs64::simple::encode(&bytes, output.as_mut_slice());
        let mut decoded_output = vec![0u8; bytes.len()];
        unsafe {
            bs64::avx2::decode(decoded_output.as_mut_slice(), &output).unwrap();
        }
        println!("{:?}", String::from_utf8(output.to_vec()).unwrap());
        println!("{}", BASE64.encode(&bytes));
        println!("{:?}", decoded_output);
    }
}

/// main function
pub fn main() {
    let args = Args::parse();

    println!("## Bytes per iteration: {}", args.bytes);
    println!("## Iterations: {}", args.iterations);

    benchmark_encode(args.bytes, args.iterations);
    benchmark_decode(args.bytes, args.iterations);
}

fn print_performance(name: &str, time: Duration, iterations: usize, num_bytes: usize) {
    let mb_per_s = ((num_bytes * iterations) as f64 / (1 << 20) as f64) / time.as_secs_f64();
    println!("{name: <20} | {mb_per_s: <15.2}");
}

fn benchmark_decode(num_bytes: usize, iterations: usize) {
    let rng = SmallRng::from_seed(20);
    let mut bytes = Vec::with_capacity(num_bytes);
    for i in 0..num_bytes {
        bytes.push(i as u8);
    }
    let encoded = bs64::encode(&bytes);
    let encoded = encoded.as_bytes();

    println!("# Decode");
    println!("{0: <20} | {1: <15}", "name", "MB/s");

    let start = Instant::now();
    for _ in 0..iterations {
        bs64::decode(&encoded).unwrap();
    }
    let total = start.elapsed();
    print_performance("bs64::decode()", total, iterations, num_bytes);

    let mut output = vec![0u8; (num_bytes * 4) / 3 + 4];
    let start = Instant::now();
    for _ in 0..iterations {
        bs64::decode_mut(&encoded, &mut output).unwrap();
    }
    let total = start.elapsed();
    print_performance("bs64::decode_mut()", total, iterations, num_bytes);

    let mut output = vec![0u8; (num_bytes * 4) / 3 + 4];
    let start = Instant::now();
    for _ in 0..iterations {
        bs64::simple::decode(&encoded, output.as_mut_slice()).unwrap();
    }
    let total = start.elapsed();
    print_performance("bs64 fallback", total, iterations, num_bytes);

    let start = Instant::now();
    for _ in 0..iterations {
        BASE64.decode(&encoded).unwrap();
    }
    let total = start.elapsed();
    print_performance("data_enc", total, iterations, num_bytes);

    let mut output = vec![0u8; BASE64.decode_len(encoded.len()).unwrap()];
    let start = Instant::now();
    for _ in 0..iterations {
        BASE64.decode_mut(&encoded, &mut output).unwrap();
    }
    let total = start.elapsed();
    print_performance("data_enc mut", total, iterations, num_bytes);

    let start = Instant::now();
    for _ in 0..iterations {
        general_purpose::STANDARD.decode(&encoded).unwrap();
    }
    let total = start.elapsed();
    print_performance("base64", total, iterations, num_bytes);

    let mut output = vec![0u8; BASE64.decode_len(encoded.len()).unwrap()];
    let start = Instant::now();
    for _ in 0..iterations {
        general_purpose::STANDARD
            .decode_slice(&encoded, &mut output)
            .unwrap();
    }
    let total = start.elapsed();
    print_performance("base64 mut", total, iterations, num_bytes);
}

fn benchmark_encode(num_bytes: usize, iterations: usize) {
    let mut bytes = Vec::with_capacity(num_bytes);
    for i in 0..num_bytes {
        bytes.push(i as u8);
    }
    println!("# Encode");
    println!("{0: <20} | {1: <15}", "name", "MB/s");

    let start = Instant::now();
    for _ in 0..iterations {
        bs64::encode(&bytes);
    }
    let total = start.elapsed();
    print_performance("bs64::encode()", total, iterations, num_bytes);

    let mut output = vec![0u8; (num_bytes * 4) / 3 + 4];
    let start = Instant::now();
    for _ in 0..iterations {
        bs64::encode_mut(&bytes, &mut output).unwrap();
    }
    let total = start.elapsed();
    print_performance("bs64::encode_mut()", total, iterations, num_bytes);

    let mut output = vec![0u8; (num_bytes * 4) / 3 + 4];
    let start = Instant::now();
    for _ in 0..iterations {
        bs64::simple::encode(&bytes, output.as_mut_slice());
    }
    let total = start.elapsed();
    print_performance("bs64 fallback", total, iterations, num_bytes);

    let start = Instant::now();
    for _ in 0..iterations {
        BASE64.encode(&bytes);
    }
    let total = start.elapsed();
    print_performance("data_enc", total, iterations, num_bytes);

    let mut output = vec![0u8; BASE64.encode_len(bytes.len())];
    let start = Instant::now();
    for _ in 0..iterations {
        BASE64.encode_mut(&bytes, &mut output);
    }
    let total = start.elapsed();
    print_performance("data_enc mut", total, iterations, num_bytes);

    let start = Instant::now();
    for _ in 0..iterations {
        let _s: String = general_purpose::STANDARD_NO_PAD.encode(&bytes);
    }
    let total = start.elapsed();
    print_performance("base64", total, iterations, num_bytes);

    let mut output = vec![0u8; BASE64.encode_len(bytes.len())];
    let start = Instant::now();
    for _ in 0..iterations {
        general_purpose::STANDARD
            .encode_slice(&bytes, &mut output)
            .unwrap();
    }
    let total = start.elapsed();
    print_performance("base64 mut", total, iterations, num_bytes);
}
