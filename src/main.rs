use base64::{engine::general_purpose, Engine as _};
use data_encoding::BASE64;
use std::time::{Duration, Instant};

fn eval() {
    for num_bytes in 100..104 {
        let mut bytes = Vec::with_capacity(num_bytes);
        for i in 0..num_bytes {
            bytes.push(i as u8);
        }
        let mut output = vec![0u8; bs64::encode_len(&bytes)];
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

fn print_performance(name: &str, time: Duration, iterations: usize) {
    let its_per_sec = iterations as f64 / time.as_secs_f64();
    let ns_per_it = time.as_nanos() / iterations as u128;
    println!("{name: <20} | {its_per_sec: <15.2} | {ns_per_it: <10}");
}

/// main function
fn main() {
    let num_bytes = 100000;
    let iterations = 1000;

    benchmark_encode(num_bytes, iterations);
    benchmark_decode(num_bytes, iterations);
}

fn benchmark_decode(num_bytes: usize, iterations: usize) {
    let mut bytes = Vec::with_capacity(num_bytes);
    for i in 0..num_bytes {
        bytes.push(i as u8);
    }
    let encoded = bs64::encode(&bytes);
    let encoded = encoded.as_bytes();

    println!("Decode:");
    println!(
        "{0: <20} | {1: <15} | {2: <10}",
        "name", "its_per_sec", "ns_per_it"
    );

    let start = Instant::now();
    for _ in 0..iterations {
        bs64::decode(&encoded).unwrap();
    }
    let total = start.elapsed();
    print_performance("bs64::decode()", total, iterations);

    let mut output = vec![0u8; (num_bytes * 4) / 3 + 4];
    let start = Instant::now();
    for _ in 0..iterations {
        bs64::decode_mut(&encoded, &mut output).unwrap();
    }
    let total = start.elapsed();
    print_performance("bs64::decode_mut()", total, iterations);

    let start = Instant::now();
    for _ in 0..iterations {
        BASE64.decode(&encoded).unwrap();
    }
    let total = start.elapsed();
    print_performance("data_enc", total, iterations);

    let mut output = vec![0u8; BASE64.decode_len(encoded.len()).unwrap()];
    let start = Instant::now();
    for _ in 0..iterations {
        BASE64.decode_mut(&encoded, &mut output).unwrap();
    }
    let total = start.elapsed();
    print_performance("data_enc mut", total, iterations);

    let start = Instant::now();
    for _ in 0..iterations {
        general_purpose::STANDARD.decode(&encoded).unwrap();
    }
    let total = start.elapsed();
    print_performance("base64", total, iterations);

    let mut output = vec![0u8; BASE64.decode_len(encoded.len()).unwrap()];
    let start = Instant::now();
    for _ in 0..iterations {
        general_purpose::STANDARD
            .decode_slice(&encoded, &mut output)
            .unwrap();
    }
    let total = start.elapsed();
    print_performance("base64 mut", total, iterations);
}

fn benchmark_encode(num_bytes: usize, iterations: usize) {
    let mut bytes = Vec::with_capacity(num_bytes);
    for i in 0..num_bytes {
        bytes.push(i as u8);
    }
    println!("Encode:");
    println!(
        "{0: <20} | {1: <15} | {2: <10}",
        "name", "its_per_sec", "ns_per_it"
    );

    let start = Instant::now();
    for _ in 0..iterations {
        bs64::encode(&bytes);
    }
    let total = start.elapsed();
    print_performance("bs64::encode()", total, iterations);

    let mut output = vec![0u8; (num_bytes * 4) / 3 + 4];
    let start = Instant::now();
    for _ in 0..iterations {
        bs64::encode_mut(&bytes, &mut output).unwrap();
    }
    let total = start.elapsed();
    print_performance("bs64::encode_mut()", total, iterations);

    let mut output = vec![0u8; (num_bytes * 4) / 3 + 4];
    let start = Instant::now();
    for _ in 0..iterations {
        unsafe {
            bs64::avx2::encode(output.as_mut_slice(), &bytes);
        }
    }
    let total = start.elapsed();
    print_performance("avx2", total, iterations);

    let mut output = vec![0u8; (num_bytes * 4) / 3 + 4];
    let start = Instant::now();
    for _ in 0..iterations {
        bs64::simple::encode(&bytes, output.as_mut_slice());
    }
    let total = start.elapsed();
    print_performance("simd", total, iterations);

    let start = Instant::now();
    for _ in 0..iterations {
        BASE64.encode(&bytes);
    }
    let total = start.elapsed();
    print_performance("data_enc", total, iterations);

    let mut output = vec![0u8; BASE64.encode_len(bytes.len())];
    let start = Instant::now();
    for _ in 0..iterations {
        BASE64.encode_mut(&bytes, &mut output);
    }
    let total = start.elapsed();
    print_performance("data_enc mut", total, iterations);

    let start = Instant::now();
    for _ in 0..iterations {
        let _s: String = general_purpose::STANDARD_NO_PAD.encode(&bytes);
    }
    let total = start.elapsed();
    print_performance("base64", total, iterations);

    let mut output = vec![0u8; BASE64.encode_len(bytes.len())];
    let start = Instant::now();
    for _ in 0..iterations {
        general_purpose::STANDARD
            .encode_slice(&bytes, &mut output)
            .unwrap();
    }
    let total = start.elapsed();
    print_performance("base64 mut", total, iterations);
}
