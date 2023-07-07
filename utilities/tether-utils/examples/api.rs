use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

fn demo_receive() {
    let tether_agent = TetherAgentOptionsBuilder::new("demo")
        .build()
        .expect("failed to init/connect Tether Agent");

    let input_plug = PlugOptionsBuilder::create_input("dummyData")
        .build(&tether_agent)
        .expect("failed to create input plug");

    loop {
        while let Some((plug_name, message)) = &tether_agent.check_messages() {
            println!("RECEIVE: got message on topic \"{}\"", message.topic());
        }
    }
}

fn main() {
    println!(
        "This example shows how the tether-utils library can be used programmatically, 
    i.e. not from the CLI"
    );
    demo_receive();
}
