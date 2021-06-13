use crate::user_builders::subscriber_builder::SubscriberBuilder;
use crate::bindings::channels::ChannelReader;
use crate::channels::ChannelReader as ChRd;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ChannelReaderBuilder{
    subscriber_builder: SubscriberBuilder
}

#[wasm_bindgen]
impl ChannelReaderBuilder{

    #[wasm_bindgen(constructor)]
    pub fn new() -> ChannelReaderBuilder{
        ChannelReaderBuilder{
            subscriber_builder: SubscriberBuilder::new()
        }
    }

    pub fn seed(mut self, seed: &str) -> ChannelReaderBuilder{
        self.subscriber_builder = self.subscriber_builder.seed(seed);
        self
    }

    pub fn node(mut self, node_url: &str) -> ChannelReaderBuilder{
        self.subscriber_builder = self.subscriber_builder.node(node_url);
        self
    }

    pub fn build(self, channel_id: &str, announce_id: &str) -> ChannelReader{
        let ch = ChRd::new(self.subscriber_builder.build(), channel_id, announce_id);
        ChannelReader::new(ch)
    }
}
