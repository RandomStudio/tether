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

    let agent = TetherAgentOptionsBuilder::new("RustDemoAgent")
        .build()
        .expect("failed to connect Tether");
    let (role, id, _) = agent.description();
    info!("Created agent OK: {}, {}", role, id);

    let empty_message_output = PlugOptionsBuilder::create_output("nothing")
        .build(&agent)
        .expect("failed to create output");
    let boolean_message_output = PlugOptionsBuilder::create_output("one")
        .build(&agent)
        .expect("failed to create output");
    let custom_output = PlugOptionsBuilder::create_output("two")
        .topic("custom/custom/two")
        .build(&agent)
        .expect("failed to create output");

    for i in 1..=10 {
        info!("#{i}: Sending empty message...");
        agent.publish(&empty_message_output, None).unwrap();

        let bool = i % 2 == 0;
        info!("#{i}: Sending boolean message...");
        agent
            .publish(&boolean_message_output, Some(&[bool.into()]))
            .unwrap();

        info!("#{i}: Sending custom struct message...");
        let custom_message = CustomStruct {
            id: i,
            name: "hello".into(),
        };
        agent
            .encode_and_publish(&custom_output, custom_message)
            .unwrap();

        thread::sleep(Duration::from_millis(1000))
    }
}
