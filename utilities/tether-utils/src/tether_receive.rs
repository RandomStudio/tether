use clap::Args;
use log::{debug, error, info, warn};
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

use crate::{defaults, Cli};

#[derive(Args)]
pub struct ReceiveOptions {
    /// Topic to subscribe; by default we recording everything
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
            } else if let Ok(value) = rmp_serde::from_slice::<rmpv::Value>(bytes) {
                let json = serde_json::to_string(&value).expect("failed to stringify JSON");
                info!("Decoded MessagePack payload: {}", json);
            } else {
                warn!("Failed to decode MessagePack payload");
                if let Ok(s) = String::from_utf8(bytes.to_vec()) {
                    warn!("String representation of payload: \"{}\"", s);
                } else {
                    error!("Could not decode payload bytes as string, either");
                }
            }
        }
        if !did_work {
            std::thread::sleep(std::time::Duration::from_micros(100)); //0.1 ms
        }
    }
}
