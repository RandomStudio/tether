use tether_agent::{
    PlugDefinition, PlugDefinitionCommon, PlugOptionsBuilder, TetherAgentOptionsBuilder,
};

fn main() {
    let tether_agent = TetherAgentOptionsBuilder::new("example")
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
        .build(&tether_agent);
    let input_wildcard_plug = PlugOptionsBuilder::create_input("everything")
        .topic(Some("#"))
        .build(&tether_agent);

    let input_customid_plug = PlugOptionsBuilder::create_input("someData")
        .role(None) // i.e., just use default
        .id(Some("specificIDonly"))
        .build(&tether_agent);

    println!("Agent looks like this: {:?}", tether_agent.description());
    let (role, id, _) = tether_agent.description();
    assert_eq!(role, "example");
    assert_eq!(id, "any"); // because we set None

    if let PlugDefinition::OutputPlug(p) = output_plug.unwrap() {
        println!("output plug: {:?}", p);
        assert_eq!(p.topic_str(), "pretendingToBeSomethingElse/any/anOutput");
    }

    println!("wildcard input plug: {:?}", input_wildcard_plug);
    println!("speific ID input plug: {:?}", input_customid_plug);
}
