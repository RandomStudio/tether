use std::{thread, time::Duration};

use env_logger::{Builder, Env};
use log::{debug, info, warn};
use rmp_serde::from_slice;
use serde::Deserialize;
use tether_agent::{ChannelOptionsBuilder, TetherAgentOptionsBuilder};

#[derive(Deserialize, Debug)]
struct CustomMessage {
    id: usize,
    name: String,
}
// Test this by sending a message like
// tether send --topic specific/any/two --message '{"id":1,"name":"boo"}'

fn main() {
    println!("Rust Tether Agent subscribe example");

    let mut builder = Builder::from_env(Env::default().default_filter_or("info"));
    builder.filter_module("tether_agent", log::LevelFilter::Warn);
    builder.filter_module("rumqttc", log::LevelFilter::Warn);
    builder.init();

    debug!("Debugging is enabled; could be verbose");

    let tether_agent = TetherAgentOptionsBuilder::new("RustDemo")
        .id(Some("example"))
        .build()
        .expect("failed to init Tether agent");

    let receiver_of_numbers = tether_agent
        .create_receiver::<u8>("numbersOnly")
        .expect("failed to create receiver");

    let receiver_of_custom_structs = tether_agent
        .create_receiver::<CustomMessage>("values")
        .expect("failed to create receiver");

    loop {
        debug!("Checking for messages...");
        while let Some((topic, payload)) = tether_agent.check_messages() {
            if let Some(m) = receiver_of_numbers.parse(&topic, &payload) {
                info!("Decoded a message for our 'numbers' Channel: {:?}", m);
            }
            if let Some(m) = receiver_of_custom_structs.parse(&topic, &payload) {
                info!(
                    "Decoded a message for our 'custom structs' Channel: {:?}",
                    m
                );
            }
            // if receiver.matches(&topic) {
            //     let decoded = from_slice::<CustomMessage>(&payload);
            //     match decoded {
            //         Ok(d) => {
            //             info!("Yes, we decoded the MessagePack payload as: {:?}", d);
            //             let CustomMessage { name, id } = d;
            //             debug!("Name is {} and ID is {}", name, id);
            //         }
            //         Err(e) => {
            //             warn!("Failed to decode the payload: {}", e)
            //         }
            //     };
            // }
        }
    }
}
