use anyhow::Result;
use iota_streams::core::prelude::hex;
use rand::Rng;
use iota_streams::app_channels::api::tangle::Address;
use std::convert::TryInto;
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::XChaCha20Poly1305;
use crypto::hashes::{
    Digest,
    blake2b::Blake2b256
};

///
/// Generates a new random String of 81 Chars of A..Z and 9
///
pub fn random_seed() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ9";
    const SEED_LEN: usize = 81;
    let mut rng = rand::thread_rng();

    let seed: String = (0..SEED_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    seed
}

///
/// Generates SendOptions struct with the specified mwm and pow
///

pub fn hash_string(string: &str) -> String{
    let hash = Blake2b256::digest(&string.as_bytes());
    hex::encode(&hash)
}

pub fn create_link(channel_address: &str, msg_id: &str) -> Result<Address>{
    match Address::from_str(channel_address, msg_id) {
        Ok(link) => Ok(link),
        Err(e) => Err(anyhow::anyhow!(e))
    }
}

pub fn create_encryption_key(string_key: &str) -> [u8; 32]{
    hash_string(string_key).as_bytes()[..32].try_into().unwrap()

}

pub fn create_encryption_nonce(string_nonce: &str) -> [u8;24]{
    hash_string(string_nonce).as_bytes()[..24].try_into().unwrap()
}

pub fn decrypt_data(data: &[u8], key: &[u8; 32], nonce: &[u8; 24]) -> Result<Vec<u8>>{
    let key_arr = GenericArray::from_slice(key);
    let nonce_arr = GenericArray::from_slice(nonce);
    let chacha = XChaCha20Poly1305::new(key_arr);
    match chacha.decrypt(nonce_arr, data.as_ref()){
        Ok(dec) => Ok(dec),
        Err(_) => return Err(anyhow::Error::msg("Error during data decryption"))
    }
}

pub fn encrypt_data(data: &[u8], key: &[u8; 32], nonce: &[u8; 24]) -> Result<Vec<u8>>{
    let key_arr = GenericArray::from_slice(key);
    let nonce_arr = GenericArray::from_slice(nonce);
    let chacha = XChaCha20Poly1305::new(key_arr);
    match chacha.encrypt(nonce_arr, data.as_ref()){
        Ok(enc) => Ok(enc),
        Err(_) => return Err(anyhow::Error::msg("Error during data encryption"))
    }
}

pub fn msg_index(address: &Address) -> String{
    let total = [address.appinst.as_ref(), address.msgid.as_ref()].concat();
    let hash = Blake2b256::digest(&total);
    hex::encode(&hash)
}
