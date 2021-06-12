use std::string::ToString;

use anyhow::Result;
use iota_streams::{
    app::transport::tangle::client::Client as StreamsClient,
    app_channels::api::tangle::Subscriber
};
use iota_streams::app::message::HasLink;
use iota_streams::app_channels::api::tangle::MessageContent;

use crate::payload::payload_types::{StreamsPacket, StreamsPacketSerializer};
use crate::utility::iota_utility::{create_link, msg_index, hash_string};
use crate::payload::payload_serializers::RawPacket;
use crate::channels::channel_state::ChannelState;
use iota_streams::app::transport::tangle::client::SendOptions;
use crate::user_builders::subscriber_builder::SubscriberBuilder;
use crate::channels::builders::channel_builders::ChannelReaderBuilder;

///
/// Channel Reader
///
pub struct ChannelReader {
    subscriber: Subscriber<StreamsClient>,
    channel_address: String,
    announcement_id: String,
    unread_msgs: Vec<(String, Vec<u8>, Vec<u8>)>,
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
            unread_msgs: Vec::new()
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

        if !self.fetch_next_msgs().await{
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
    /// Send a subscription to a channels for future private communications
    ///
    pub async fn send_subscription(&mut self) -> Result<String>{
        let link = create_link(&self.channel_address, &self.announcement_id)?;
        let addr = self.subscriber.send_subscribe(&link).await?.msgid.to_string();
        Ok(addr)
    }

    ///
    /// Receive a signed packet and return it in a StreamsPacket struct that is able to parse its content to your own types
    ///
    pub async fn receive_parsed_packet<T>(&mut self, msg_id: &str, key_nonce: Option<([u8;32], [u8;24])>) -> Result<StreamsPacket<T>>
        where
            T: StreamsPacketSerializer,
    {
        let msg_link = create_link(&self.channel_address, msg_id)?;
        let (_, public_payload, masked_payload) = self.subscriber.receive_signed_packet(&msg_link).await?;
        let (p_data, m_data) = (&public_payload.0, &masked_payload.0);

        StreamsPacket::from_streams_response(&p_data, &m_data, &key_nonce)
    }

    ///
    /// Receive a signed packet in raw format. It returns a tuple (pub_bytes, masked_bytes)
    ///
    pub async fn receive_raw_packet<T>(&mut self, msg_id: &str) -> Result<(Vec<u8>, Vec<u8>)>
        where
            T: StreamsPacketSerializer,
    {
        let msg_link = create_link(&self.channel_address, msg_id)?;
        let (_, public_payload, masked_payload) = self.subscriber.receive_signed_packet(&msg_link).await?;
        Ok((public_payload.0.clone(), masked_payload.0.clone()))
    }

    ///
    /// Fetch all the remaining msgs
    ///
    /// # Return Value
    /// It returns a Vector of Tuple containing (msg_id, public_bytes, masked_bytes)
    ///
    pub async fn fetch_raw_msgs(&mut self) -> Vec<(String, Vec<u8>, Vec<u8>)> {
        self.fetch_next_msgs().await;
        let res = self.unread_msgs.clone();
        self.unread_msgs = Vec::new();
        res
    }

    ///
    /// Fetch all the remaining msgs
    ///
    /// # Return Value
    /// It returns a Vector of StreamsPacket that can parse its content
    ///
    pub async fn fetch_parsed_msgs<T>(&mut self, key_nonce: &Option<([u8;32], [u8;24])>) -> Result<Vec<(String, StreamsPacket<T>)>>
    where
        T: StreamsPacketSerializer
    {
        self.fetch_next_msgs().await;

        let mut res = vec![];
        for (id, p, m) in &self.unread_msgs {
            res.push((id.clone(), StreamsPacket::from_streams_response(p, m, key_nonce)?));
        }

        self.unread_msgs = Vec::new();
        Ok(res)
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
            unread_msgs: Vec::new(),
        })
    }

    fn export(&self, psw: &str) -> Result<ChannelState>{
        let psw_hash = hash_string(psw);
        let author_state = self.subscriber.export(&psw_hash)?;
        Ok(ChannelState::new(&author_state, &self.channel_address, &self.announcement_id, ""))
    }

    async fn fetch_next_msgs(&mut self) -> bool{
        let msgs = self.subscriber.fetch_all_next_msgs().await;
        let mut found = false;
        for msg in msgs {
            let link = msg.link.rel();
            match msg.body{
                MessageContent::SignedPacket {pk: _, public_payload, masked_payload } => {
                    let p = public_payload.0;
                    let m = masked_payload.0;

                    if !p.is_empty() || !m.is_empty(){
                        self.unread_msgs.push((link.to_string(), p, m));
                        found = true;
                    }
                }
                _ => {println!("{}", link.to_string());}
            }
        }
        found
    }
}
