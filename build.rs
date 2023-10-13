//build.rs
use std::io::{Result, Write} ;
use std::path::Path ;
use std::fs::File ;
use std::env ;

const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn swap_endianess(x: usize) -> usize {
    (x << 8) | (x >> 8)
}

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap() ;
    let dest_path = Path::new(&out_dir).join("luts.rs") ;
    let mut f = File::create(&dest_path).unwrap() ;


    write!(f, "pub const C0_LUT: [u8; 256] = [\n")? ;
    for i in 0..256 {
        write!(f, "{}, ", CHARS[i >> 2])? ;
        if i % 16 == 0 {
            write!(f, "\n")? ;
        }
    }
    write!(f, "];\n")? ;

    write!(f, "pub const C1_LUT: [u8; 65536] = [\n")? ;
    for i in 0..65536 {
        write!(f, "{}, ", CHARS[(swap_endianess(i) >> 4) & 0x3f])? ;
        if i % 16 == 0 {
            write!(f, "\n")? ;
        }
    }
    write!(f, "];\n")? ;

    write!(f, "pub const C2_LUT: [u8; 65536] = [\n")? ;
    for i in 0..65536 {
        write!(f, "{}, ", CHARS[(swap_endianess(i) >> 6) & 0x3f])? ;
        if i % 16 == 0 {
            write!(f, "\n")? ;
        }
    }
    write!(f, "];\n")? ;

    write!(f, "pub const C3_LUT: [u8; 65536] = [\n")? ;
    for i in 0..65536 {
        write!(f, "{}, ", CHARS[i & 0x3f])? ;
        if i % 16 == 0 {
            write!(f, "\n")? ;
        }
    }
    write!(f, "];\n")? ;

    Ok(())
}
