use base64::{engine::general_purpose, Engine as _};
use data_encoding::BASE64;
use std::time::{Duration, Instant};

fn eval() {
    for num_bytes in 100..104 {
        let mut output = vec![0u8; (num_bytes * 4) / 3 + 4];
        let mut bytes = Vec::with_capacity(num_bytes);
        for i in 0..num_bytes {
            bytes.push(i as u8);
        }
        bs64::simple::encode(&bytes, output.as_mut_slice());
        println!("{:?}", String::from_utf8(output.to_vec()).unwrap());
        println!("{}", BASE64.encode(&bytes));
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

    let mut bytes = Vec::with_capacity(num_bytes);
    for i in 0..num_bytes {
        bytes.push(i as u8);
    }
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
        general_purpose::STANDARD_NO_PAD
            .encode_slice(&bytes, &mut output)
            .unwrap();
    }
    let total = start.elapsed();
    print_performance("base64 mut", total, iterations);
}
