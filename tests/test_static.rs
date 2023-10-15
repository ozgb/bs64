use bs64;
use data_encoding::BASE64;
use rand::prelude::*;

const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const NON_VALID_CHARS: &[u8; 30] = b"!@#$%^&*()_{}|:<>?~`-=[]\\;',.\"";

#[test]
fn zero_len() {
    let input = b"";
    let expected = b"";
    let mut output = vec![0u8; bs64::encode_len(input.len())];
    let len = bs64::encode_mut(input, &mut output).unwrap();
    assert_eq!(output, expected);
    assert_eq!(len, 0);

    let input = b"";
    let expected = "";
    let output = bs64::encode(input);
    assert_eq!(output, expected);
}

#[test]
fn length_to_1000_simple() {
    let mut rng = SmallRng::seed_from_u64(20);
    for i in 0..1000 {
        let mut input = vec![0u8; i];
        rng.fill(&mut input[..]);

        let expected = BASE64.encode(&input);
        let mut output = vec![0u8; bs64::encode_len(input.len())];
        bs64::simple::encode(&input, &mut output);
        assert_eq!(output, expected.as_bytes());
    }
}

#[test]
fn length_to_1000() {
    let mut rng = SmallRng::seed_from_u64(20);
    for i in 64..1000 {
        println!("{i}");
        let mut input = vec![0u8; i];
        rng.fill(&mut input[..]);

        let expected = BASE64.encode(&input);
        let output = bs64::encode(&input);
        assert_eq!(output, expected);
    }
}

#[test]
fn output_size() {
    let mut rng = SmallRng::seed_from_u64(20);
    for i in 0..1000 {
        let mut input = vec![0u8; i];
        rng.fill(&mut input[..]);
        let mut output = vec![0u8; bs64::encode_len(input.len())];

        let output_len = bs64::encode_mut(&input, &mut output).unwrap();
        assert_eq!(output_len, output.len());
    }
}

#[test]
fn garbage_decode_results_in_error() {
    let mut rng = SmallRng::seed_from_u64(20);
    for i in 10..1000 {
        let mut input: Vec<u8> = vec![0u8; i]
            .iter()
            .map(|_| *CHARS.choose(&mut rng).unwrap())
            .collect();
        assert_eq!(input.len(), i);
        let mut output = vec![0u8; bs64::encode_len(input.len())];

        // Insert invalid char
        let j = rng.gen_range(0..i);
        input[j] = *NON_VALID_CHARS.choose(&mut rng).unwrap();

        let is_err = bs64::decode_mut(&input, &mut output).is_err();
        assert!(is_err);
    }
}
