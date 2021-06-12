use std::marker::PhantomData;

use anyhow::Result;
use base64::{decode_config, encode_config, URL_SAFE_NO_PAD};
use iota_streams::ddml::types::Bytes;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::utility::iota_utility::{decrypt_data, encrypt_data};

pub trait StreamsPacketSerializer {
    fn serialize<T: Serialize>(data: &T) -> Result<String>;
    fn deserialize<T: DeserializeOwned>(data: &[u8]) -> Result<T>;
}

pub struct StreamsPacket<P>{
    p_data: Vec<u8>,
    m_data: Vec<u8>,
    _marker: PhantomData<P>,
    key_nonce: Option<([u8;32], [u8;24])>,
}

impl<P> StreamsPacket<P>
where
    P: StreamsPacketSerializer,
{
    fn new(p_data: &[u8], m_data: &[u8], key_nonce: Option<([u8;32], [u8;24])>) -> StreamsPacket<P>{
        StreamsPacket{
            p_data: p_data.to_vec(),
            m_data: m_data.to_vec(),
            _marker: PhantomData,
            key_nonce
        }
    }

    pub fn from_streams_response(p_data: &[u8], m_data: &[u8], key_nonce: &Option<([u8;32], [u8;24])>) -> Result<StreamsPacket<P>>{
        let (p, m) = match key_nonce{
            None => (p_data.to_vec(), m_data.to_vec()),
            Some((key, nonce)) => {
                let dec = decrypt_data(m_data, key, nonce)?;
                (p_data.to_vec(), dec)
            }
        };

        Ok(
            StreamsPacket{
            p_data: decode_config(p, URL_SAFE_NO_PAD)?,
            m_data: decode_config(m, URL_SAFE_NO_PAD)?,
            _marker: PhantomData,
            key_nonce: key_nonce.clone(),
            }
        )
    }

    pub fn public_data(&self) -> Result<Bytes> {
        let p = encode_config(&self.p_data, URL_SAFE_NO_PAD).as_bytes().to_vec();
        Ok(Bytes(p))
    }

    pub fn masked_data(&self) -> Result<Bytes> {
        let m = encode_config(&self.m_data, URL_SAFE_NO_PAD).as_bytes().to_vec();
        let data = match &self.key_nonce{
            None => m,
            Some((key, nonce)) => encrypt_data(&m, key, nonce)?
        };

        Ok(Bytes(data))
    }

    pub fn deserialize<U, T>(&self) -> Result<(U, T)>
    where
        U: DeserializeOwned,
        T: DeserializeOwned,
    {
        Ok((P::deserialize(&self.p_data)?, P::deserialize(&self.m_data)?))
    }

    pub fn deserialize_public<T>(&self) -> Result<T>
    where
        T: DeserializeOwned
    {
        Ok(P::deserialize(&self.p_data)?)
    }

    pub fn deserialize_masked<T>(&self) -> Result<T>
        where
            T: DeserializeOwned
    {
        Ok(P::deserialize(&self.m_data)?)
    }

}

pub struct StreamsPacketBuilder<P>{
    public: String,
    masked: String,
    _pub_marker: PhantomData<P>,
    key_nonce: Option<([u8;32], [u8;24])>,
}

impl<P> StreamsPacketBuilder<P>
where
    P: StreamsPacketSerializer,
{
    pub fn new() -> Self{
        StreamsPacketBuilder{
            public: String::new(),
            masked: String::new(),
            _pub_marker: PhantomData,
            key_nonce: None
        }
    }

    pub fn public<T>(&mut self, data: &T) -> Result<&mut Self>
    where
        T: serde::Serialize{
        self.public = P::serialize(data)?;
        Ok(self)
    }

    pub fn masked<T>(&mut self, data: &T) -> Result<&mut Self>
    where
        T: serde::Serialize
    {
        self.masked = P::serialize(data)?;
        Ok(self)
    }

    pub fn key_nonce(&mut self, key: &[u8;32], nonce: &[u8;24]) -> &mut Self{
        self.key_nonce = Some((key.clone(), nonce.clone()));
        self
    }

    pub fn build(&mut self) -> StreamsPacket<P> {
        StreamsPacket::new(&self.public.as_bytes(), &self.masked.as_bytes(), self.key_nonce.clone())
    }

}
