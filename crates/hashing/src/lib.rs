use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HashingError {
    #[error("I/O error during hashing: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to decode image for perceptual hashing: {0}")]
    ImageDecode(String),
}

/// Compute hex BLAKE3 hash of in-memory byte slice.
pub fn hash_bytes(data: &[u8]) -> String {
    let hash = blake3::hash(data);
    hash.to_hex().to_string()
}

/// Compute hex BLAKE3 hash of a file by streaming in 64KB chunks.
/// Avoids loading large files into memory.
pub fn hash_file<P: AsRef<Path>>(path: P) -> Result<String, HashingError> {
    let file = File::open(path)?;
    let reader = BufReader::with_capacity(64 * 1024, file);
    hash_reader(reader)
}

/// Stream bytes from any reader into the BLAKE3 hasher.
pub fn hash_reader<R: Read>(mut reader: R) -> Result<String, HashingError> {
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Compute a 64-bit perceptual difference hash (dHash) for an image.
/// Resizes image to 9x8 grayscale, then compares adjacent pixel intensities.
pub fn perceptual_hash_image(image_bytes: &[u8]) -> Result<u64, HashingError> {
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| HashingError::ImageDecode(e.to_string()))?;

    // Convert to grayscale and resize to 9 width x 8 height
    let gray = img.to_luma8();
    let resized = image::imageops::resize(&gray, 9, 8, image::imageops::FilterType::Triangle);

    let mut hash: u64 = 0;
    let mut bit_pos = 0;

    for y in 0..8 {
        for x in 0..8 {
            let left_pixel = resized.get_pixel(x, y)[0];
            let right_pixel = resized.get_pixel(x + 1, y)[0];
            if left_pixel > right_pixel {
                hash |= 1 << bit_pos;
            }
            bit_pos += 1;
        }
    }

    Ok(hash)
}

/// Calculate similarity percentage between two perceptual hashes.
/// 1.0 (100%) = visually identical.
pub fn perceptual_similarity(hash_a: u64, hash_b: u64) -> f64 {
    let hamming_distance = (hash_a ^ hash_b).count_ones();
    let similarity = 1.0 - (hamming_distance as f64 / 64.0);
    (similarity * 10000.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_bytes() {
        let h1 = hash_bytes(b"hello world");
        let h2 = hash_bytes(b"hello world");
        let h3 = hash_bytes(b"hello world!");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert_eq!(h1.len(), 64); // BLAKE3 hex representation is 64 characters
    }

    #[test]
    fn test_perceptual_similarity() {
        let h1: u64 = 0b10101010;
        let h2: u64 = 0b10101010;
        assert_eq!(perceptual_similarity(h1, h2), 100.0);

        let h3: u64 = 0b10101011; // 1 bit flipped
        assert!(perceptual_similarity(h1, h3) > 98.0);
    }
}
