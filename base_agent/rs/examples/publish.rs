use std::{thread, time::Duration};

use env_logger::{Builder, Env};
use log::{debug, info};
use serde::Serialize;
use tether_agent::TetherAgentOptionsBuilder;

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
        .finalize()
        .expect("failed to connect Tether");
    let (role, id) = agent.description();
    info!("Created agent OK: {}, {}", role, id);

    let empty_message_output: tether_agent::PlugDefinition = agent
        .create_output_plug("nothing", None, None, None)
        .unwrap();
    let boolean_message_output = agent.create_output_plug("one", None, None, None).unwrap();
    let custom_output = agent.create_output_plug("two", None, None, None).unwrap();

    for i in 1..=10 {
        info!("#{i}: Sending empty message...");
        agent.publish(&empty_message_output, None).unwrap();

        let bool = i % 2 == 0;
        info!("#{i}: Sending boolean message...");
        agent
            .publish(&boolean_message_output, Some(&[bool.into()]))
            .unwrap();

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
