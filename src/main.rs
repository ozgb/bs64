use base64::{engine::general_purpose, Engine as _};
use bs64::codecs::sponge::Sponge;
use data_encoding::BASE64;
use std::time::{Duration, Instant};

use bs64::codecs::fairy::Fairy;
use bs64::codecs::vanilla::Vanilla;

use bs64::codecs::Codec;

fn eval<T: Codec>(c: &T) {
    let output = c.encode(b"hello\n");
    assert_eq!(output, "aGVsbG8K");

    let output = c.encode(b"helllo\n");
    assert_eq!(output, "aGVsbGxvCg==");
}

fn eval_chromium() {
    let num_bytes = 100;
    let mut output = vec![0u8; (num_bytes * 4) / 3 + 4];
    let mut bytes = Vec::with_capacity(num_bytes);
    for i in 0..num_bytes {
        bytes.push(i as u8);
    }
    bs64::codecs::safesimd::encode(&bytes, output.as_mut_slice());
    println!("{:?}", String::from_utf8(output.to_vec()).unwrap());
    println!("{}", BASE64.encode(&bytes));
}

fn print_performance(name: &str, time: Duration, iterations: usize) {
    let its_per_sec = iterations as f64 / time.as_secs_f64();
    let ns_per_it = time.as_nanos() / iterations as u128;
    println!("{name: <10} | {its_per_sec: <15.2} | {ns_per_it: <10}");
}

/// main function
fn main() {
    let fairy = Fairy::default();
    let vanilla = Vanilla::default();
    let sponge = Sponge::default();

    eval(&fairy);
    eval(&vanilla);

    let num_bytes = 100000;
    let iterations = 1000;

    let mut bytes = Vec::with_capacity(num_bytes);
    for i in 0..num_bytes {
        bytes.push(i as u8);
    }
    println!(
        "{0: <10} | {1: <15} | {2: <10}",
        "name", "its_per_sec", "ns_per_it"
    );

    //let start = Instant::now();
    //for _ in 0..iterations {
    //    vanilla.encode(&bytes);
    //}
    //let total = start.elapsed();
    //print_performance("vanilla", total, iterations);

    let mut output = vec![0u8; (num_bytes * 4) / 3 + 4];
    let start = Instant::now();
    for _ in 0..iterations {
        unsafe {
            bs64::codecs::avx2::encode(output.as_mut_slice(), &bytes);
        }
    }
    let total = start.elapsed();
    print_performance("avx2", total, iterations);

    let mut output = vec![0u8; (num_bytes * 4) / 3 + 4];
    let start = Instant::now();
    for _ in 0..iterations {
        bs64::codecs::safesimd::encode(&bytes, output.as_mut_slice());
    }
    let total = start.elapsed();
    print_performance("safesimd", total, iterations);

    //let start = Instant::now();
    //for _ in 0..iterations {
    //    fairy.encode(&bytes);
    //}
    //let total = start.elapsed();
    //print_performance("fairy", total, iterations);

    //let start = Instant::now();
    //for _ in 0..iterations {
    //    sponge.encode(&bytes);
    //}
    //let total = start.elapsed();
    //print_performance("sponge", total, iterations);

    let start = Instant::now();
    for _ in 0..iterations {
        BASE64.encode(&bytes);
    }
    let total = start.elapsed();
    print_performance("data_enc", total, iterations);

    let start = Instant::now();
    for _ in 0..iterations {
        let _s: String = general_purpose::STANDARD_NO_PAD.encode(&bytes);
    }
    let total = start.elapsed();
    print_performance("base64", total, iterations);
}
