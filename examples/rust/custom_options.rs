use std::time::Duration;

use tether_agent::{ChannelOptionsBuilder, TetherAgentOptionsBuilder};

fn main() {
    let mut tether_agent = TetherAgentOptionsBuilder::new("example")
        .id(None)
        .host(Some("localhost"))
        .port(Some(1883))
        .username(Some("tether"))
        .password(Some("sp_ceB0ss!"))
        .build()
        .expect("failed to create Tether Agent");

    let sender_channel = ChannelOptionsBuilder::create_sender("anOutput")
        .role(Some("pretendingToBeSomethingElse"))
        .qos(Some(2))
        .retain(Some(true))
        .build()
        .expect("failed to create sender channel");
    let input_wildcard_channel = ChannelOptionsBuilder::create_receiver("everything")
        .topic(Some("#"))
        .build(&mut tether_agent);

    let input_customid_channel = ChannelOptionsBuilder::create_receiver("someData")
        .role(None) // i.e., just use default
        .id(Some("specificIDonly"))
        .build(&mut tether_agent);

    println!("Agent looks like this: {:?}", tether_agent.description());
    let (role, id, _) = tether_agent.description();
    assert_eq!(role, "example");
    assert_eq!(id, "any"); // because we set None

    if let ChannelDefinition::ChannelSender(p) = &sender_channel {
        println!("sender channel: {:?}", p);
        assert_eq!(
            p.generated_topic(),
            "pretendingToBeSomethingElse/any/anOutput"
        );
    }

    println!("wildcard input channel: {:?}", input_wildcard_channel);
    println!("speific ID input channel: {:?}", input_customid_channel);

    let payload =
        rmp_serde::to_vec::<String>(&String::from("boo")).expect("failed to serialise payload");
    tether_agent
        .send(&sender_channel, Some(&payload))
        .expect("failed to publish");

    std::thread::sleep(Duration::from_millis(4000));
}
