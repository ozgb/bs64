use bs64::codecs::sponge::Sponge;
use data_encoding::BASE64;
use base64::{Engine as _, engine::general_purpose};
use rand::prelude::*;
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

    let num_bytes = 10000;
    let iterations = 1000;

    let mut bytes = Vec::with_capacity(1000);
    for _ in 0..num_bytes {
        bytes.push(random::<u8>());
    }
    println!("{0: <10} | {1: <15} | {2: <10}", "name", "its_per_sec", "ns_per_it");

    let mut times = Vec::new();
    for _ in 0..iterations {
        let start = Instant::now();
        vanilla.encode(&bytes);
        times.push(start.elapsed());
    }

    let total: Duration = times.iter().sum();
    print_performance("vanilla", total, iterations);

    let mut times = Vec::new();
    for _ in 0..iterations {
        let start = Instant::now();
        fairy.encode(&bytes);
        times.push(start.elapsed());
    }

    let total: Duration = times.iter().sum();
    print_performance("fairy", total, iterations);

    let mut times = Vec::new();
    for _ in 0..iterations {
        let start = Instant::now();
        sponge.encode(&bytes);
        times.push(start.elapsed());
    }

    let total: Duration = times.iter().sum();
    print_performance("sponge", total, iterations);

    let mut times = Vec::new();
    for _ in 0..iterations {
        let start = Instant::now();
        BASE64.encode(&bytes);
        times.push(start.elapsed());
    }

    let total: Duration = times.iter().sum();
    print_performance("data_enc", total, iterations);

    let mut times = Vec::new();
    for _ in 0..iterations {
        let start = Instant::now();
        let _s: String = general_purpose::STANDARD_NO_PAD.encode(&bytes);
        times.push(start.elapsed());
    }

    let total: Duration = times.iter().sum();
    print_performance("base64", total, iterations);
}
