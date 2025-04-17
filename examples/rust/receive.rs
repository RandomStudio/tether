use env_logger::{Builder, Env};
use log::*;
use serde::Deserialize;
use tether_agent::TetherAgentBuilder;

#[allow(dead_code)]
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

    let tether_agent = TetherAgentBuilder::new("RustDemo")
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
        }
    }
}
