use std::string::ToString;

use anyhow::Result;
use iota_streams::{
    app::transport::tangle::client::Client as StreamsClient,
    app_channels::api::tangle::Subscriber
};
use iota_streams::app::message::HasLink;
use iota_streams::app_channels::api::tangle::MessageContent;

use crate::utility::iota_utility::{create_link, msg_index, hash_string};
use crate::payload::payload_serializers::RawPacket;
use crate::channels::channel_state::ChannelState;
use iota_streams::app::transport::tangle::client::SendOptions;
use crate::user_builders::subscriber_builder::SubscriberBuilder;
use crate::channels::builders::channel_builders::ChannelReaderBuilder;
use std::collections::VecDeque;

///
/// Channel Reader
///
pub struct ChannelReader {
    subscriber: Subscriber<StreamsClient>,
    channel_address: String,
    announcement_id: String,
    unread_msgs: VecDeque<(String, Vec<u8>, Vec<u8>)>,
}

impl ChannelReader {

    ///
    /// Gets the builder of the ChannelReader
    ///
    pub fn builder() -> ChannelReaderBuilder{
        ChannelReaderBuilder::new()
    }

    ///
    /// Initialize the Channel Reader
    ///
    pub fn new(subscriber: Subscriber<StreamsClient>, channel_address: &str, announcement_id: &str) -> ChannelReader {
        ChannelReader {
            subscriber,
            channel_address: channel_address.to_string(),
            announcement_id: announcement_id.to_string(),
            unread_msgs: VecDeque::new()
        }
    }

    ///
    /// Restore the channels from a previously stored byte array state
    ///
    pub fn import_from_bytes(state: &[u8], psw: &str, node_url: Option<&str>, send_options: Option<SendOptions>) -> Result<ChannelReader>{
        let channel_state = ChannelState::decrypt(&state, &psw)?;
        let channel = ChannelReader::import(&channel_state, psw, node_url, send_options)?;
        Ok(channel)
    }

    ///
    /// Export the channels state into an encrypted byte array.
    ///
    pub fn export_to_bytes(&self, psw: &str)-> Result<Vec<u8>>{
        let channel_state = self.export(psw)?;
        channel_state.encrypt(psw)
    }

    ///
    /// Attach the Reader to Channel
    ///
    pub async fn attach(&mut self) -> Result<()> {
        let link = create_link(&self.channel_address, &self.announcement_id)?;
        self.subscriber.receive_announcement(&link).await?;

        if !self.fetch_all_msgs().await{
            return Ok(());
        }

        let comp = format!("{}:{}.state", self.channel_address, self.announcement_id);
        let msg = &self.unread_msgs[0];
        let packet = match RawPacket::from_streams_response(&msg.1, &msg.2, &None){
            Ok(packet) => packet,
            Err(_) => return Ok(())
        };

        match packet.deserialize_public::<String>(){
            Ok(state_msg) => {
                if state_msg == comp{
                    self.unread_msgs.remove(0);
                }
            }
            Err(_) => {}
        };
        Ok(())
    }

    ///
    /// Fetch all the remaining msgs
    ///
    /// # Return Value
    /// It returns a Vector of Tuple containing (msg_id, public_bytes, masked_bytes)
    ///
    pub async fn fetch_raw_msgs(&mut self) -> u32 {
        self.fetch_all_msgs().await;
        self.unread_msgs.len() as u32
    }

    pub fn has_next_msg(&self) -> bool{
        !self.unread_msgs.is_empty()
    }

    pub fn pop_next_msg(&mut self) -> Option<(String, Vec<u8>, Vec<u8>)>{
        self.unread_msgs.pop_front()
    }

    ///
    /// Get the channels address and the announcement id
    ///
    pub fn channel_address(&self) -> (String, String){
        (self.channel_address.clone(), self.announcement_id.clone())
    }

    ///
    /// Get the index of msg to find the transaction on the tangle
    ///
    pub fn msg_index(&self, msg_id: &str) -> Result<String>{
        let addr = create_link(&self.channel_address, msg_id)?;
        Ok(msg_index(&addr))
    }
}

impl ChannelReader{

    fn import(channel_state: &ChannelState, psw: &str, node_url: Option<&str>, send_options: Option<SendOptions>) -> Result<ChannelReader>{
        let subscriber = SubscriberBuilder::build_from_state(
            &channel_state.user_state(),
            psw,
            node_url,
            send_options
        )?;
        let channel_address = subscriber.channel_address().unwrap().to_string();

        Ok(ChannelReader {
            subscriber,
            channel_address,
            announcement_id: channel_state.announcement_id(),
            unread_msgs: VecDeque::new(),
        })
    }

    fn export(&self, psw: &str) -> Result<ChannelState>{
        let psw_hash = hash_string(psw);
        let author_state = self.subscriber.export(&psw_hash)?;
        Ok(ChannelState::new(&author_state, &self.channel_address, &self.announcement_id, ""))
    }

    async fn fetch_all_msgs(&mut self) -> bool{
        let msgs = self.subscriber.fetch_all_next_msgs().await;
        let mut found = false;
        for msg in msgs {
            let link = msg.link.rel();
            match msg.body{
                MessageContent::SignedPacket {pk: _, public_payload, masked_payload } => {
                    let p = public_payload.0;
                    let m = masked_payload.0;

                    if !p.is_empty() || !m.is_empty(){
                        self.unread_msgs.push_back((link.to_string(), p, m));
                        found = true;
                    }
                }
                _ => {println!("{}", link.to_string());}
            }
        }
        found
    }
}
