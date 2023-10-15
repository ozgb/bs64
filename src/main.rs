#[cfg(feature = "cli")]
mod benchmark_cli;

#[cfg(feature = "cli")]
fn main() {
    benchmark_cli::main();
}

#[cfg(not(feature = "cli"))]
fn main() {
    println!("cli missing");
}
