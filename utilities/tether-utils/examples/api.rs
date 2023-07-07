use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

fn demo_receive() {
    let tether_agent = TetherAgentOptionsBuilder::new("demoReceive")
        .build()
        .expect("failed to init/connect Tether Agent");

    let input_plug = PlugOptionsBuilder::create_input("dummyData")
        .build(&tether_agent)
        .expect("failed to create input plug");

    let mut count = 0;

    loop {
        while let Some((plug_name, message)) = &tether_agent.check_messages() {
            count += 1;
            println!(
                "RECEIVE: got message#{} on topic \"{}\"",
                count,
                message.topic()
            );
        }
    }
}

fn demo_send() {
    let tether_agent = TetherAgentOptionsBuilder::new("demoSend")
        .build()
        .expect("failed to init/connect Tether Agent");

    let output_plug = PlugOptionsBuilder::create_output("dummyData")
        .build(&tether_agent)
        .expect("failed to create output plug");

    let mut count = 0;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        count += 1;
        println!("SEND: sending message #{}", count);
        tether_agent
            .encode_and_publish(&output_plug, count)
            .expect("failed to publish");
    }
}

fn main() {
    println!(
        "This example shows how the tether-utils library can be used programmatically, 
    i.e. not from the CLI"
    );
    println!("Press Ctrl+C to stop");

    let mut handles = Vec::new();
    handles.push(std::thread::spawn(move || {
        demo_receive();
    }));
    handles.push(std::thread::spawn(move || {
        demo_send();
    }));

    for handle in handles {
        handle.join().expect("failed to join handle");
    }
}
