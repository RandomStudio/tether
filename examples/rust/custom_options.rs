use std::time::Duration;

use tether_agent::{definitions::*, AgentConfig, TetherAgentBuilder};

fn main() {
    let tether_agent = TetherAgentBuilder::new("example")
        .id(None)
        .host(Some("localhost"))
        .port(Some(1883))
        .username(Some("tether"))
        .password(Some("sp_ceB0ss!"))
        .build()
        .expect("failed to create Tether Agent");

    let sender_channel_def = ChannelSenderDefBuilder::new("anOutput")
        .role(Some("pretendingToBeSomethingElse"))
        .qos(Some(rumqttc::QoS::ExactlyOnce))
        .retain(Some(true))
        .build(tether_agent.config());

    let sender_channel = tether_agent.create_sender_with_def(sender_channel_def);

    let input_wildcard_channel_def = ChannelReceiverDefBuilder::new("everything")
        .override_topic(Some("#"))
        .build(tether_agent.config());
    let input_wildcard_channel = tether_agent
        .create_receiver_with_def::<u8>(input_wildcard_channel_def)
        .expect("failed to create Channel Receiver");

    // let input_customid_channel_def = ChannelReceiverOptions::new("someData")
    //     .role(None) // i.e., just use default
    //     .id(Some("specificIDonly"))
    //     .build();

    println!("Agent looks like this: {:?}", tether_agent.config());
    let AgentConfig { role, id, .. } = &tether_agent.config();
    assert_eq!(role, "example");
    assert!(id.is_none()); // because we set None

    println!(
        "wildcard input channel: {:?}",
        input_wildcard_channel.definition().generated_topic()
    );

    sender_channel
        .send(&tether_agent, &String::from("boo"))
        .expect("failed to publish");

    std::thread::sleep(Duration::from_millis(4000));
}
