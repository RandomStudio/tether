use std::time::Duration;

use tether_agent::{
    channels::options::ChannelOptions, receiver_options::ChannelReceiverOptionsBuilder,
    ChannelCommon, ChannelSenderOptionsBuilder, TetherAgentOptionsBuilder,
};

fn main() {
    let mut tether_agent = TetherAgentOptionsBuilder::new("example")
        .id(None)
        .host(Some("localhost"))
        .port(Some(1883))
        .username(Some("tether"))
        .password(Some("sp_ceB0ss!"))
        .build()
        .expect("failed to create Tether Agent");

    let sender_channel = ChannelSenderOptionsBuilder::new("anOutput")
        .role(Some("pretendingToBeSomethingElse"))
        .qos(Some(2))
        .retain(Some(true))
        .build(&mut tether_agent)
        .expect("failed to create sender channel");
    let input_wildcard_channel = ChannelReceiverOptionsBuilder::new("everything")
        .override_topic(Some("#"))
        .build::<u8>(&mut tether_agent)
        .expect("failed to create receiver channel");

    let input_customid_channel = ChannelReceiverOptionsBuilder::new("someData")
        .role(None) // i.e., just use default
        .id(Some("specificIDonly"))
        .build::<u8>(&mut tether_agent)
        .expect("failed to create receiver channel");

    println!("Agent looks like this: {:?}", tether_agent.description());
    let (role, id, _) = tether_agent.description();
    assert_eq!(role, "example");
    assert_eq!(id, "any"); // because we set None

    println!(
        "wildcard input channel: {:?}",
        input_wildcard_channel.generated_topic()
    );
    println!(
        "speific ID input channel: {:?}",
        input_customid_channel.generated_topic()
    );

    let payload =
        rmp_serde::to_vec::<String>(&String::from("boo")).expect("failed to serialise payload");
    tether_agent
        .send(&sender_channel, Some(&payload))
        .expect("failed to publish");

    std::thread::sleep(Duration::from_millis(4000));
}
