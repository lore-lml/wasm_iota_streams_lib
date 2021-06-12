use crate::user_builders::author_builder::AuthorBuilder;
use iota_streams::app::transport::tangle::client::SendOptions;
use crate::channels::{ChannelReader, ChannelWriter};
use crate::user_builders::subscriber_builder::SubscriberBuilder;


pub struct ChannelWriterBuilder{
    author_builder: AuthorBuilder
}

impl ChannelWriterBuilder{

    pub fn new() -> ChannelWriterBuilder{
        ChannelWriterBuilder{
            author_builder: AuthorBuilder::new()
        }
    }

    pub fn seed(mut self, seed: &str) -> Self{
        self.author_builder = self.author_builder.seed(seed);
        self
    }

    pub fn node(mut self, node_url: &str) -> Self{
        self.author_builder = self.author_builder.node(node_url);
        self
    }

    pub fn send_options(mut self, send_options: SendOptions) -> Self{
        self.author_builder = self.author_builder.send_options(send_options);
        self
    }

    pub fn build(self) -> ChannelWriter{
        ChannelWriter::new(self.author_builder.build())
    }
}


pub struct ChannelReaderBuilder{
    subscriber_builder: SubscriberBuilder
}

impl ChannelReaderBuilder{

    pub fn new() -> ChannelReaderBuilder{
        ChannelReaderBuilder{
            subscriber_builder: SubscriberBuilder::new()
        }
    }

    pub fn seed(mut self, seed: &str) -> Self{
        self.subscriber_builder = self.subscriber_builder.seed(seed);
        self
    }

    pub fn node(mut self, node_url: &str) -> Self{
        self.subscriber_builder = self.subscriber_builder.node(node_url);
        self
    }

    pub fn send_options(mut self, send_options: SendOptions) -> Self{
        self.subscriber_builder = self.subscriber_builder.send_options(send_options);
        self
    }

    pub fn build(self, channel_id: &str, announce_id: &str) -> ChannelReader{
        ChannelReader::new(self.subscriber_builder.build(), channel_id, announce_id)
    }
}
