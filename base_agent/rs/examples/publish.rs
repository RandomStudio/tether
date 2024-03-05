use std::{thread, time::Duration};

use env_logger::{Builder, Env};
use log::{debug, info};
use serde::Serialize;
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

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

    let mut tether_agent = TetherAgentOptionsBuilder::new("RustDemoAgent")
        .build()
        .expect("failed to connect Tether");
    let (role, id, _) = tether_agent.description();
    info!("Created agent OK: {}, {}", role, id);

    let empty_message_output = PlugOptionsBuilder::create_output("nothing")
        .build(&mut tether_agent)
        .expect("failed to create output");
    let boolean_message_output = PlugOptionsBuilder::create_output("one")
        .build(&mut tether_agent)
        .expect("failed to create output");
    let custom_output = PlugOptionsBuilder::create_output("two")
        .topic(Some("custom/custom/two"))
        .build(&mut tether_agent)
        .expect("failed to create output");
    let grouped_output_1 = PlugOptionsBuilder::create_output("one")
        .id(Some("groupMessages"))
        .build(&mut tether_agent)
        .expect("failed to create output");
    let grouped_output_2 = PlugOptionsBuilder::create_output("two")
        .id(Some("groupMessages"))
        .build(&mut tether_agent)
        .expect("failed to create output");

    for i in 1..=10 {
        info!("#{i}: Sending empty message...");
        tether_agent.publish(&empty_message_output, None).unwrap();

        let bool = i % 2 == 0;
        info!("#{i}: Sending boolean message...");
        tether_agent
            .publish(&boolean_message_output, Some(&[bool.into()]))
            .unwrap();

        info!("#{i}: Sending custom struct message...");
        let custom_message = CustomStruct {
            id: i,
            name: "hello".into(),
        };
        tether_agent
            .encode_and_publish(&custom_output, custom_message)
            .unwrap();

        info!("#{i}: Sending grouped messages...");
        tether_agent.publish(&grouped_output_1, None).unwrap();
        tether_agent.publish(&grouped_output_2, None).unwrap();

        thread::sleep(Duration::from_millis(1000))
    }
}
