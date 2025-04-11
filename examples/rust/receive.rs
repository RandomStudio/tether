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

    let mut builder = Builder::from_env(Env::default().default_filter_or("debug"));
    builder.filter_module("tether_agent", log::LevelFilter::Warn);
    builder.filter_module("rumqttc", log::LevelFilter::Warn);
    builder.init();

    debug!("Debugging is enabled; could be verbose");

    let tether_agent = TetherAgentOptionsBuilder::new("RustDemo")
        .id(Some("example"))
        .build()
        .expect("failed to init Tether agent");

    let receiver = tether_agent
        .create_receiver::<u8>("numbersOnly")
        .expect("failed to create receiver");
}
