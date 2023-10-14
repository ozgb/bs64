use self::luts::{CHARPAD, E0, E1, E2};

mod luts;

pub fn encode_32(src: &[u8], dest: &mut [u8]) {}

pub fn encode(src: &[u8], dest: &mut [u8]) -> usize {
    let len = src.len();
    let mut src_i = 0;
    let mut dest_i = 0;
    //loop {
    //    break;
    //    if len - src_i < 24 {
    //        break;
    //    }
    //    encode_32(&src[src_i..src_i+24], &mut dest[dest_i..dest_i+32]);

    //    src_i += 24;
    //    dest_i += 32;
    //}

    let mut t1: u8;
    let mut t2: u8;
    let mut t3: u8;

    if len - src_i > 2 {
        for i in (src_i..len - 2).step_by(3) {
            t1 = src[i];
            t2 = src[i + 1];
            t3 = src[i + 2];
            dest[dest_i] = E0[t1 as usize];
            dest_i += 1;
            dest[dest_i] = E1[(((t1 & 0x03) << 4) | ((t2 >> 4) & 0x0F)) as usize];
            dest_i += 1;
            dest[dest_i] = E1[(((t2 & 0x0F) << 2) | ((t3 >> 6) & 0x03)) as usize];
            dest_i += 1;
            dest[dest_i] = E2[t3 as usize];
            dest_i += 1;
            src_i = i + 3;
        }
    }

    match len - src_i {
        0 => (),
        1 => {
            t1 = src[src_i as usize];
            dest[dest_i] = E0[t1 as usize];
            dest_i += 1;
            dest[dest_i] = E1[((t1 & 0x03) << 4) as usize];
            dest_i += 1;
            dest[dest_i] = CHARPAD;
            dest_i += 1;
            dest[dest_i] = CHARPAD;
            dest_i += 1;
        }
        _ => {
            t1 = src[src_i];
            t2 = src[src_i + 1];
            dest[dest_i] = E0[t1 as usize];
            dest_i += 1;
            dest[dest_i] = E1[(((t1 & 0x03) << 4) | ((t2 >> 4) & 0x0F)) as usize];
            dest_i += 1;
            dest[dest_i] = E2[((t2 & 0x0F) << 2) as usize];
            dest_i += 1;
            dest[dest_i] = CHARPAD;
            dest_i += 1;
        }
    }

    dest[dest_i] = b'\0';
    dest_i + 1
}
