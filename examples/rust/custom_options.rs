use std::time::Duration;

use tether_agent::{
    options::{
        definitions::ChannelDefinition, receiver_options::ChannelReceiverOptions,
        sender_options::ChannelSenderOptions, ChannelOptions,
    },
    TetherAgentOptionsBuilder,
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

    let sender_channel_def = ChannelSenderOptions::new("anOutput")
        .role(Some("pretendingToBeSomethingElse"))
        .qos(Some(2))
        .retain(Some(true))
        .build(&tether_agent);

    let sender_channel = tether_agent.create_sender_with_definition(sender_channel_def);

    let input_wildcard_channel_def = ChannelReceiverOptions::new("everything")
        .override_topic(Some("#"))
        .build();
    let input_wildcard_channel = tether_agent
        .create_receiver_with_definition::<u8>(input_wildcard_channel_def)
        .expect("failed to create Channel Receiver");

    // let input_customid_channel_def = ChannelReceiverOptions::new("someData")
    //     .role(None) // i.e., just use default
    //     .id(Some("specificIDonly"))
    //     .build();

    println!("Agent looks like this: {:?}", tether_agent.description());
    let (role, id, _) = tether_agent.description();
    assert_eq!(role, "example");
    assert_eq!(id, "any"); // because we set None

    println!(
        "wildcard input channel: {:?}",
        input_wildcard_channel.generated_topic()
    );
    // println!(
    //     "speific ID input channel: {:?}",
    //     input_customid_channel_def.generated_topic()
    // );

    let payload =
        rmp_serde::to_vec::<String>(&String::from("boo")).expect("failed to serialise payload");
    tether_agent
        .send(&sender_channel, Some(&payload))
        .expect("failed to publish");

    std::thread::sleep(Duration::from_millis(4000));
}
