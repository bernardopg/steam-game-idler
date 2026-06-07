//! Build-time API key obfuscation.
//!
//! IMPORTANT: this is **obfuscation, not encryption**. The key is XOR-masked
//! with a deterministic, source-visible mask and embedded in the binary at
//! compile time. Anyone with the binary can recover the original value with a
//! few lines of code, since both the SEED/SALT and the derivation are public.
//!
//! This exists only to keep the Steam Web API key out of plaintext `strings`
//! output — it is a speed bump, not a security boundary. The Steam Web API key
//! is semi-public, rate-limited and has no destructive scope, so this trade-off
//! is acceptable. Do not rely on this for protecting real secrets.

use std::env;

const SEED: [u8; 32] = [
    0x4a, 0x9b, 0x2c, 0xfd, 0x8e, 0x1f, 0x7a, 0x3b, 0xcc, 0x5d, 0x9e, 0x6f, 0x0a, 0xbb, 0x4c, 0xdd,
    0x2e, 0x8f, 0x10, 0xa1, 0x52, 0xe3, 0x74, 0x05, 0xb6, 0x47, 0xd8, 0x69, 0xfa, 0x2b, 0x9c, 0x1d,
];

const SALT: [u8; 16] = [
    0x73, 0x61, 0x6c, 0x74, 0x5f, 0x66, 0x6f, 0x72, 0x5f, 0x61, 0x70, 0x69, 0x5f, 0x6b, 0x65, 0x79,
];

/// Derive the deterministic XOR mask used to obfuscate the embedded key.
const fn derive_obfuscation_mask() -> [u8; 32] {
    let mut key = [0u8; 32];
    let mut i = 0;

    while i < 32 {
        let salt_byte = SALT[i % 16];
        let mut temp = SEED[i] ^ salt_byte;

        let mut round = 0;
        while round < 100 {
            temp = temp.wrapping_add(SEED[(i + round) % 32]);
            temp = temp ^ (temp >> 3);
            temp = temp.wrapping_mul(0x9e);
            temp = temp ^ salt_byte;
            round += 1;
        }

        key[i] = temp;
        i += 1;
    }

    key
}

/// Obfuscate an API key at compile time into a fixed-size buffer.
const fn obfuscate_api_key_const(api_key: &str) -> ([u8; 64], usize) {
    let mask = derive_obfuscation_mask();
    let api_bytes = api_key.as_bytes();
    let api_len = api_bytes.len();

    let mut obfuscated = [0u8; 64];
    let mut i = 0;
    while i < api_len && i < 64 {
        obfuscated[i] = api_bytes[i] ^ mask[i % 32] ^ ((i as u8).wrapping_mul(7));
        i += 1;
    }

    (obfuscated, api_len)
}

/// Recover the embedded API key, or an empty string when none was baked in.
pub fn deobfuscate_api_key() -> String {
    match option_env!("STEAM_API_KEY") {
        Some(_compile_time_key) => {
            const OBFUSCATED_DATA: ([u8; 64], usize) = {
                match option_env!("STEAM_API_KEY") {
                    Some(key) => obfuscate_api_key_const(key),
                    None => ([0u8; 64], 0),
                }
            };

            let (obfuscated_data, original_len) = OBFUSCATED_DATA;

            let mask = derive_obfuscation_mask();

            let mut recovered = Vec::with_capacity(original_len);
            for i in 0..original_len {
                let byte = obfuscated_data[i] ^ mask[i % 32] ^ ((i as u8).wrapping_mul(7));
                recovered.push(byte);
            }

            String::from_utf8_lossy(&recovered).to_string()
        }
        None => String::new(),
    }
}

/// Read the API key from the environment (dev / user-provided path).
pub fn get_api_key_from_env() -> Result<String, String> {
    env::var("KEY")
        .or_else(|_| env::var("STEAM_API_KEY"))
        .map_err(|_| "No API key found in environment variables".to_string())
}
