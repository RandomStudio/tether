use std::time::Duration;

use clap::Args;
use log::{debug, info};
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

use crate::{defaults, Cli};

#[derive(Args)]
pub struct ReceiveOptions {
    #[arg(long = "topic", default_value_t=String::from("#"))]
    subscribe_topic: String,
}

pub fn receive(cli: &Cli, options: &ReceiveOptions) {
    info!("Tether Receive Utility");
    let tether_agent = TetherAgentOptionsBuilder::new(defaults::AGENT_ROLE)
        .host(&cli.tether_host)
        .port(cli.tether_port)
        .username(&cli.tether_username)
        .password(&cli.tether_password)
        .build()
        .expect("failed to connect Tether");

    let _input = PlugOptionsBuilder::create_input("all")
        .topic(&options.subscribe_topic)
        .build(&tether_agent)
        .expect("failed to create input plug");

    loop {
        let mut did_work = false;
        while let Some((plug_name, message)) = tether_agent.check_messages() {
            did_work = true;
            debug!("Received message on plug {}: {:?}", plug_name, message);
            info!("Received message on topic \"{}\"", message.topic());
            let bytes = message.payload();
            if bytes.is_empty() {
                info!("Empty message payload");
            } else {
                let value: rmpv::Value =
                    rmp_serde::from_slice(bytes).expect("failed to decode msgpack");
                let json = serde_json::to_string(&value).expect("failed to stringify JSON");
                info!("Decoded MessagePack payload: {}", json);
            }
        }
        if !did_work {
            std::thread::sleep(Duration::from_micros(100)); //0.1 ms
        }
    }
}
