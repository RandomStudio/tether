use std::time::Duration;

use tether_agent::{
    PlugDefinition, PlugDefinitionCommon, PlugOptionsBuilder, TetherAgentOptionsBuilder,
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

    let output_plug = PlugOptionsBuilder::create_output("anOutput")
        .role(Some("pretendingToBeSomethingElse"))
        .qos(Some(2))
        .retain(Some(true))
        .build(&mut tether_agent)
        .expect("failed to create output plug");
    let input_wildcard_plug = PlugOptionsBuilder::create_input("everything")
        .topic(Some("#"))
        .build(&mut tether_agent);

    let input_customid_plug = PlugOptionsBuilder::create_input("someData")
        .role(None) // i.e., just use default
        .id(Some("specificIDonly"))
        .build(&mut tether_agent);

    println!("Agent looks like this: {:?}", tether_agent.description());
    let (role, id, _) = tether_agent.description();
    assert_eq!(role, "example");
    assert_eq!(id, "any"); // because we set None

    if let PlugDefinition::OutputPlug(p) = &output_plug {
        println!("output plug: {:?}", p);
        assert_eq!(
            p.generated_topic(),
            "pretendingToBeSomethingElse/any/anOutput"
        );
    }

    println!("wildcard input plug: {:?}", input_wildcard_plug);
    println!("speific ID input plug: {:?}", input_customid_plug);

    let payload =
        rmp_serde::to_vec::<String>(&String::from("boo")).expect("failed to serialise payload");
    tether_agent
        .publish(&output_plug, Some(&payload))
        .expect("failed to publish");

    std::thread::sleep(Duration::from_millis(4000));
}
