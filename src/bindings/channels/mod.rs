mod channel_reader;
pub use channel_reader::ChannelReader;


use wasm_bindgen::prelude::*;
use crate::utility::iota_utility::{create_encryption_key, create_encryption_nonce};
use std::convert::TryInto;

#[wasm_bindgen]
#[derive(Clone)]
pub struct ResponseMessage{
    msg_id: String,
    public: Vec<u8>,
    masked: Vec<u8>
}

impl ResponseMessage{
    pub fn new(msg_id: String, public: Vec<u8>, masked: Vec<u8>) -> Self {
        ResponseMessage { msg_id, public, masked }
    }
}

#[wasm_bindgen]
impl ResponseMessage{
    #[wasm_bindgen(getter)]
    pub fn msg_id(&self) -> String {
        self.msg_id.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn public(&self) -> Vec<u8> {
        self.public.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn masked(&self) -> Vec<u8>{
        self.masked.clone()
    }
}

#[wasm_bindgen]
pub struct ChannelInfo{
    channel_id: String,
    announce_id: String
}

#[wasm_bindgen]
impl ChannelInfo{
    #[wasm_bindgen(constructor)]
    pub fn new(channel_id: &str, announce_id: &str) -> ChannelInfo{
        ChannelInfo{
            channel_id: channel_id.to_string(),
            announce_id: announce_id.to_string()
        }
    }
    pub fn channel_id(&self) -> String {
        self.channel_id.clone()
    }
    pub fn announce_id(&self) -> String {
        self.announce_id.clone()
    }
}

#[wasm_bindgen]
pub struct KeyNonce{
    key: [u8; 32],
    nonce: [u8; 24]
}

#[wasm_bindgen]
impl KeyNonce{
    #[wasm_bindgen(constructor)]
    pub fn new(key: &str, nonce: &str) -> KeyNonce{
        KeyNonce{
            key: create_encryption_key(key),
            nonce: create_encryption_nonce(nonce)
        }
    }

    #[wasm_bindgen(getter)]
    pub fn key(&self) -> Vec<u8> {
        self.key.to_vec()
    }

    #[wasm_bindgen(getter)]
    pub fn nonce(&self) -> Vec<u8> {
        self.nonce.to_vec()
    }

    pub fn clone(&self) -> KeyNonce{
        KeyNonce{
            key: self.key.to_vec().try_into().unwrap(),
            nonce: self.nonce.to_vec().try_into().unwrap()
        }
    }
}

impl KeyNonce{
    pub fn key_ref(&self) -> &[u8; 32] {
        &self.key
    }

    pub fn nonce_ref(&self) -> &[u8; 24] {
        &self.nonce
    }
}
