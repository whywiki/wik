//! SHA-1 written from the specification (FIPS 180-4) without external crates

/// Pads `data` to a whole number of 64-byte blocks:
/// - `data`
/// - the byte `0x80`
/// - zero bytes padding
/// - length of `data` in bits as a big-endian `u64`
fn pad(data: &[u8]) -> Vec<u8> {
    // Length of the original message in bits, as a u64
    // (SHA-1 stores it in 8 bytes)
    let bit_length = data.len() as u64 * 8;

    // convert data to bytes
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
