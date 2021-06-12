use iota_streams::app::transport::{
    TransportOptions,
    tangle::client::{Client as StreamsClient, SendOptions}
};
use iota_streams::app_channels::api::tangle::Author;
use crate::utility::iota_utility::{random_seed, hash_string};
use anyhow::Result;
use iota_streams::app_channels::api::ChannelType;

pub struct AuthorBuilder{
    seed: String,
    node_url: String,
    send_options: SendOptions
}

impl AuthorBuilder{
    pub fn new() -> AuthorBuilder{
        let mut send_opts = SendOptions::default();
        send_opts.local_pow = false;

        AuthorBuilder{
            seed: random_seed(),
            node_url: "https://api.lb-0.testnet.chrysalis2.com".to_string(),
            send_options: send_opts
        }
    }

    pub fn build_from_state(author_state: &[u8],
                            psw: &str,
                            node_url: Option<&str>,
                            send_option: Option<SendOptions>) -> Result<Author<StreamsClient>>{

        let psw_hash = hash_string(psw);
        let node = match node_url {
            Some(url) => url,
            None => "https://api.lb-0.testnet.chrysalis2.com"
        };
        let options = match send_option {
            Some(so) => so,
            None => {
                let mut s = SendOptions::default();
                s.local_pow = false;
                s
            }
        };

        let mut client = StreamsClient::new_from_url(&node);
        client.set_send_options(options);
        Author::import(author_state, &psw_hash, client)
    }
}

impl AuthorBuilder{
    pub fn seed(mut self, seed: &str) -> Self{
        self.seed = seed.to_string();
        self
    }

    pub fn node(mut self, node_url: &str) -> Self{
        self.node_url = node_url.to_string();
        self
    }

    pub fn send_options(mut self, send_options: SendOptions) -> Self{
        self.send_options = send_options;
        self
    }

    pub fn build(self) -> Author<StreamsClient>{
        let mut client = StreamsClient::new_from_url(&self.node_url);
        client.set_send_options(self.send_options);

        Author::new(
            &self.seed,
            ChannelType::SingleBranch,
            client
        )
    }
}
