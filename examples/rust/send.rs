use std::{thread, time::Duration};

use env_logger::{Builder, Env};
use log::{debug, info};
use serde::Serialize;
use tether_agent::{ChannelOptionsBuilder, TetherAgentOptionsBuilder};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CustomStruct {
    id: usize,
    name: String,
}
fn main() {
    println!("Rust Tether Agent publish example");

    let mut builder = Builder::from_env(Env::default().default_filter_or("info"));
    builder.init();

    debug!("Debugging is enabled; could be verbose");

    let mut tether_agent = TetherAgentOptionsBuilder::new("rustExample")
        .build()
        .expect("failed to connect Tether");
    let (role, id, _) = tether_agent.description();
    info!("Created agent OK: {}, {}", role, id);

    let empty_message_output = ChannelOptionsBuilder::create_sender("nothing")
        .build(&mut tether_agent)
        .expect("failed to create output");
    let boolean_message_output = ChannelOptionsBuilder::create_sender("one")
        .build(&mut tether_agent)
        .expect("failed to create output");
    let custom_output = ChannelOptionsBuilder::create_sender("two")
        .topic(Some("custom/custom/two"))
        .build(&mut tether_agent)
        .expect("failed to create output");
    let grouped_output_1 = ChannelOptionsBuilder::create_sender("one")
        .id(Some("groupMessages"))
        .build(&mut tether_agent)
        .expect("failed to create output");
    let grouped_output_2 = ChannelOptionsBuilder::create_sender("two")
        .id(Some("groupMessages"))
        .build(&mut tether_agent)
        .expect("failed to create output");

    for i in 1..=10 {
        info!("#{i}: Sending empty message...");
        tether_agent.send_raw(&empty_message_output, None).unwrap();

        let just_a_boolean = i % 2 == 0;
        info!("#{i}: Sending boolean message...");
        tether_agent
            .send(&boolean_message_output, just_a_boolean)
            .unwrap();

        info!("#{i}: Sending custom struct message...");
        let custom_message = CustomStruct {
            id: i,
            name: "hello".into(),
        };
        tether_agent.send(&custom_output, custom_message).unwrap();

        info!("#{i}: Sending grouped messages...");
        tether_agent.send_empty(&grouped_output_1).unwrap();
        tether_agent.send_empty(&grouped_output_2).unwrap();

        thread::sleep(Duration::from_millis(1000))
    }
}
