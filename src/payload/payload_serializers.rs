use anyhow::Result;
use iota_streams::core::prelude::hex;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::payload::payload_types::{StreamsPacket, StreamsPacketBuilder, StreamsPacketSerializer};

pub struct RawSerializer;

impl StreamsPacketSerializer for RawSerializer{
    fn serialize<T: Serialize>(data: &T) -> Result<String> {
        let bytes = bincode::serialize(data)?;
        Ok(hex::encode(bytes))
    }

    fn deserialize<T: DeserializeOwned>(data: &[u8]) -> Result<T> {
        let data = hex::decode(data)?;
        Ok(bincode::deserialize(&data)?)
    }
}

pub struct JsonSerializer;

impl StreamsPacketSerializer for JsonSerializer{
    fn serialize<T: Serialize>(data: &T) -> Result<String> {
        let bytes = serde_json::to_string(data)?.as_bytes().to_vec();
        let bytes = bincode::serialize(&bytes)?;
        Ok(hex::encode(bytes))
    }

    fn deserialize<T: DeserializeOwned>(data: &[u8]) -> Result<T> {
        let data = hex::decode(data)?;
        let data: Vec<u8> = bincode::deserialize(&data)?;
        Ok(serde_json::from_slice(&data)?)
    }
}

pub type RawPacket = StreamsPacket<RawSerializer>;
pub type RawPacketBuilder = StreamsPacketBuilder<RawSerializer>;

pub type JsonPacket = StreamsPacket<JsonSerializer>;
pub type JsonPacketBuilder = StreamsPacketBuilder<JsonSerializer>;
