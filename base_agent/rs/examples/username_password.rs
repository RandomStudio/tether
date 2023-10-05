use std::{thread, time::Duration};

use env_logger::{Builder, Env};
use log::{debug, info};
use serde::Serialize;
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CustomStruct {
    foo: String,
    bar: f32,
}
fn main() {
    println!("Rust Tether Agent: with username and password");

    let mut builder = Builder::from_env(Env::default().default_filter_or("info"));
    builder.init();

    debug!("Debugging is enabled; could be verbose");

    let tether_agent = TetherAgentOptionsBuilder::new("RustDemoAgent")
        .host("10.112.10.10")
        .username("connected.space")
        .password("connected.space")
        .build()
        .expect("Failed to initialise and connect");
    let (role, id, _) = tether_agent.description();
    info!("Created agent OK: {}, {}", role, id);

    let empty_message_output = PlugOptionsBuilder::create_output("nothing")
        .build(&tether_agent)
        .expect("failed to create output");
    let boolean_message_output = PlugOptionsBuilder::create_output("one")
        .build(&tether_agent)
        .expect("failed to create output");
    let custom_output = PlugOptionsBuilder::create_output("two")
        .build(&tether_agent)
        .expect("failed to create output");

    for i in 1..=10 {
        info!("#{i}: Sending empty message...");
        tether_agent.publish(&empty_message_output, None).unwrap();

        let bool = i % 2 == 0;
        info!("#{i}: Sending boolean message...");
        tether_agent
            .publish(&boolean_message_output, Some(&[bool.into()]))
            .unwrap();

        let custom_message = CustomStruct {
            foo: "hello".into(),
            bar: 0.42,
        };
        tether_agent
            .encode_and_publish(&custom_output, custom_message)
            .unwrap();

        thread::sleep(Duration::from_millis(1000))
    }
}
