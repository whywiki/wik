//! SHA-1 written from the specification (FIPS 180-4) without external crates

use crate::{HashAlgorithm, ObjectId};

/// The five starting numbers of the state fixed by the standard.
const INITIAL_STATE: [u32; 5] = [
    0x6745_2301,
    0xEFCD_AB89,
    0x98BA_DCFE,
    0x1032_5476,
    0xC3D2_E1F0,
];

/// SHA-1 written for wik from the specification, without external crates.
#[derive(Debug, Clone, Copy)]
pub struct Sha1Own;

impl HashAlgorithm for Sha1Own {
    fn hash(&self, data: &[u8]) -> ObjectId {
        let mut state = INITIAL_STATE;

        // 1: cut the message into 64-byte parts
        let padded = pad(data);
        let (blocks, _) = padded.as_chunks::<64>();

        // 2: blend each portion into the state
        for block in blocks {
            let boxes = rounds(&state, &schedule(block));
            for (number, mixed) in state.iter_mut().zip(boxes) {
                *number = number.wrapping_add(mixed);
            }
        }

        // 3: turn 5 numbers into 20 bytes
        let mut bytes = [0_u8; 20];
        let (pieces, _) = bytes.as_chunks_mut::<4>();
        for (four_bytes, number) in pieces.iter_mut().zip(state) {
            *four_bytes = number.to_be_bytes();
        }

        ObjectId::Sha1(bytes)
    }
}

/// Pads `data` to a whole number of 64-byte blocks:
/// - `data`
/// - the byte `0x80`
/// - zero bytes padding
/// - length of `data` in bits as a big-endian `u64`
fn pad(data: &[u8]) -> Vec<u8> {
    // Length of the original message in bits, as a u64
    // (SHA-1 stores it in 8 bytes)
    let bit_length = data.len() as u64 * 8;

    // copy the message so we can add bytes to it
    let mut bytes: Vec<u8> = data.to_vec();

    // end of message marker
    bytes.push(0x80);

    // Add zero bytes until exactly 8 bytes are missing
    // to the next multiple of 64. Those last 8 bytes
    // are reserved for the length
    while bytes.len() % 64 != 56 {
        bytes.push(0x00);
    }

    // Append the length as 8 big-endian bytes,
    // the total is now a multiple of 64
    bytes.extend_from_slice(&bit_length.to_be_bytes());

    bytes
}

/// Expands one 64-byte block into 80 number message schedule
/// (FIPS 180-4, section 6.1.2):
/// - the first 16 are the block itself (read 4 bytes at a time)
/// - later one is made from 4 earlier ones
fn schedule(block: &[u8; 64]) -> [u32; 80] {
    let mut w = [0_u32; 80];

    // numbers 0..15: block itself, 4 bytes each
    for t in 0..16 {
        w[t] = u32::from_be_bytes([
            block[4 * t],
            block[4 * t + 1],
            block[4 * t + 2],
            block[4 * t + 3],
        ]);
    }

    // numbers 16..79: each made from 4 earlier numbers
    for t in 16..80 {
        w[t] = (w[t - 3] ^ w[t - 8] ^ w[t - 14] ^ w[t - 16]).rotate_left(1);
    }

    w
}

/// Runs 80 rounds on copies of `state`. One round per number in `w`
/// and returns the five boxes at the end. `state` is not changed.
fn rounds(state: &[u32; 5], w: &[u32; 80]) -> [u32; 5] {
    let [mut a, mut b, mut c, mut d, mut e] = *state;

    for (t, &ingredient) in w.iter().enumerate() {
        // recipe changes every 20 rounds
        let (f, k) = match t {
            0..=19 => ((b & c) | (!b & d), 0x5A82_7999),
            20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
            40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
            _ => (b ^ c ^ d, 0xCA62_C1D6),
        };

        // new number
        let temp = a
            .rotate_left(5)
            .wrapping_add(f)
            .wrapping_add(e)
            .wrapping_add(k)
            .wrapping_add(ingredient);

        // Shift every box one to the right, new number first.
        (a, b, c, d, e) = (temp, a, b.rotate_left(30), c, d);
    }

    [a, b, c, d, e]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sha1Crate;

    /// Hashes `data` with `Sha1Own` and returns the id as hex
    fn sha1_hex(data: &[u8]) -> String {
        Sha1Own.hash(data).to_string()
    }

    #[test]
    fn padding_fills_whole_64_byte_blocks() {
        assert_eq!(pad(b"").len(), 64);
        assert_eq!(pad(&[0; 55]).len(), 64);
        assert_eq!(pad(&[0; 56]).len(), 128);
        assert_eq!(pad(&[0; 64]).len(), 128);
    }

    #[test]
    fn padding_appends_0x80_and_zeros_and_length() {
        let padded = pad(b"abc");

        assert_eq!(&padded[..3], b"abc");
        assert_eq!(padded[3], 0x80);
        assert!(padded[4..56].iter().all(|&byte| byte == 0));
        // 3 bytes == 24 bits as a big endian 64 bit number
        assert_eq!(&padded[56..], &24_u64.to_be_bytes());
    }

    #[test]
    fn schedule_reads_the_block_as_16_big_endian_numbers() {
        let block: [u8; 64] = pad(b"abc").try_into().unwrap();

        let w = schedule(&block);

        assert_eq!(w[0], 0x6162_6380);
        assert!(w[1..15].iter().all(|&number| number == 0));
        assert_eq!(w[15], 0x0000_0018);
    }

    #[test]
    fn schedule_builds_later_numbers_from_earlier_numbers() {
        let block: [u8; 64] = pad(b"abc").try_into().unwrap();

        let w = schedule(&block);

        assert_eq!(w[16], 0xC2C4_C700);
        assert_eq!(w[17], 0);
        assert_eq!(w[18], 0x0000_0030);
    }

    #[test]
    fn rounds_match_the_standards_abc_example() {
        let block: [u8; 64] = pad(b"abc").try_into().unwrap();

        let boxes = rounds(&INITIAL_STATE, &schedule(&block));

        assert_eq!(
            boxes,
            [
                0x4254_1B35,
                0x5738_D5E1,
                0x2183_4873,
                0x681E_6DF6,
                0xD8FD_F6AD
            ]
        );
    }

    #[test]
    fn matches_standard_test_vectors() {
        // From the SHA-1 standard (FIPS 180-4) and its examples
        assert_eq!(sha1_hex(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(sha1_hex(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(
            sha1_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
        );
        assert_eq!(
            sha1_hex(&vec![b'a'; 1_000_000]),
            "34aa973cd4c4daa4f61eeb2bdbad27316534016f"
        );
    }

    #[test]
    fn matches_the_crate_for_every_length_up_to_300() {
        let data: Vec<u8> = (0..300_u32).map(|i| (i * 7 + 3) as u8).collect();

        for len in 0..=data.len() {
            let input = &data[..len];
            assert_eq!(
                Sha1Own.hash(input),
                Sha1Crate.hash(input),
                "different hash for length {len}"
            );
        }
    }
}
