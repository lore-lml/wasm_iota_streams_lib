use crate::channels::ChannelReader as ChRd;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use crate::utils::set_panic_hook;
use crate::bindings::channels::{ResponseMessage, KeyNonce, ChannelInfo};
use crate::payload::payload_serializers::{RawPacket, RawPacketBuilder};
use anyhow::Result;
use std::collections::VecDeque;


#[wasm_bindgen]
pub struct ChannelReader{
    channel: Rc<RefCell<ChRd>>,
    unread_msgs: VecDeque<(String, Vec<u8>, Vec<u8>)>,
}


impl ChannelReader {

    pub fn new(channel: ChRd) -> ChannelReader{
        set_panic_hook();
        ChannelReader{
            channel: Rc::new(RefCell::new(channel)),
            unread_msgs: VecDeque::new(),
        }
    }
}

#[wasm_bindgen]
impl ChannelReader{

    pub fn clone(&self) -> ChannelReader{
        ChannelReader{
            channel: self.channel.clone(),
            unread_msgs: self.unread_msgs.clone(),
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
    pub async fn fetch_raw_msgs(self) -> bool {
        let msgs = self.channel.borrow_mut().fetch_raw_msgs().await;
        if msgs.len() <= 0{
            return false
        }



        true
    }

    #[wasm_bindgen(catch)]
    pub fn pop_msg(&mut self, key_nonce: Option<KeyNonce>) -> Result<ResponseMessage, JsValue>{
        let (msg_id, public, masked) = match self.unread_msgs.pop_front(){
            None => return Err(JsValue::null()),
            Some(res) => res
        };

        match decode_response_message(&msg_id, &public, &masked, key_nonce){
            Ok(res) => Ok(res),
            Err(_) => Err(JsValue::null())
        }
    }

    pub fn has_next_msg(&self) -> bool{
        !self.unread_msgs.is_empty()
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
