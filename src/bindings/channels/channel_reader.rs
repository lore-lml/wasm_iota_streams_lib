use crate::channels::ChannelReader as ChRd;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use crate::utils::set_panic_hook;
use crate::bindings::channels::{ResponseMessage, KeyNonce, ChannelInfo, EncryptedState};
use crate::payload::payload_serializers::{RawPacket, RawPacketBuilder};
use anyhow::{Result};


#[wasm_bindgen]
pub struct ChannelReader{
    channel: Rc<RefCell<ChRd>>
}


impl ChannelReader {

    pub fn new(channel: ChRd) -> ChannelReader{
        set_panic_hook();
        ChannelReader{
            channel: Rc::new(RefCell::new(channel)),
        }
    }
}

#[wasm_bindgen]
impl ChannelReader{

    pub fn clone(&self) -> ChannelReader{
        ChannelReader{
            channel: self.channel.clone(),
        }
    }

    ///
    /// Attach the Reader to Channel
    ///
    #[wasm_bindgen(catch)]
    pub async fn attach(self) -> Result<(), String> {
        match self.channel.borrow_mut().attach().await{
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string())
        }
    }

    ///
    /// Fetch all the remaining msgs
    ///
    /// # Return Value
    /// It returns a Vector of Tuple containing (msg_id, public_bytes, masked_bytes)
    ///
    pub async fn fetch_raw_msgs(self) -> u32 {
        self.channel.borrow_mut().fetch_raw_msgs().await
    }

    #[wasm_bindgen(catch)]
    pub fn pop_msg(&self, key_nonce: Option<KeyNonce>) -> Result<ResponseMessage, JsValue>{
        let (msg_id, public, masked) = match self.channel.borrow_mut().pop_next_msg(){
            None => return Err(JsValue::null()),
            Some(res) => res
        };

        match decode_response_message(&msg_id, &public, &masked, key_nonce){
            Ok(res) => Ok(res),
            Err(_) => Err(JsValue::null())
        }
    }

    pub fn has_next_msg(&self) -> bool{
        self.channel.borrow().has_next_msg()
    }

    pub fn channel_address(&self) -> ChannelInfo{
        let (channel_id, announce_id) = self.channel.borrow().channel_address();
        ChannelInfo::new(&channel_id, &announce_id)
    }

    ///
    /// Get the index of msg to find the transaction on the tangle
    ///
    pub fn msg_index(&self, msg_id: &str) -> Result<String, JsValue>{
        match self.channel.borrow().msg_index(msg_id){
            Ok(index) => Ok(index),
            Err(_) => Err(JsValue::null())
        }
    }

    pub fn export_to_bytes(&self, psw: &str) -> Result<EncryptedState, JsValue>{
        match self.channel.borrow().export_to_bytes(psw){
            Ok(state) => Ok(EncryptedState::new(state)),
            Err(e) => Err(JsValue::from_str(&e.to_string()))
        }
    }

    pub fn import_from_bytes(state: &EncryptedState, psw: &str, node_url: Option<String>) -> Result<ChannelReader, JsValue>{
        match ChRd::import_from_bytes(state.state(), psw, node_url.as_deref(), None){
            Ok(reader) => Ok(ChannelReader{
                channel: Rc::new(RefCell::new(reader))
            }),
            Err(e) => Err(JsValue::from_str(&e.to_string()))
        }
    }
}

fn decode_response_message(msg_id: &str, public: &[u8], masked: &[u8], key_nonce: Option<KeyNonce>) -> Result<ResponseMessage>{
    let key_nonce = match key_nonce{
        None => None,
        Some(kn) => Some((kn.key_ref().clone(), kn.nonce_ref().clone()))
    };
    let p_packet = RawPacket::from_streams_response(public, public, &None)?;
    let m_packet = match RawPacket::from_streams_response(public, masked, &key_nonce){
        Ok(m) => m,
        Err(_) => RawPacketBuilder::new().public(&"Encrypted".as_bytes().to_vec())?.build()
    };
    let p = p_packet.deserialize_public()?;
    let m = match m_packet.deserialize_masked(){
        Ok(m) => m,
        Err(_) => RawPacketBuilder::new().public(&"Encrypted".as_bytes().to_vec())?.build().deserialize_public()?
    };
    Ok(ResponseMessage::new(msg_id.to_string(), p, m))
}
