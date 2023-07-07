use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};
use tether_utils::{
    tether_send::{send, SendOptions},
    tether_topics::{subscribe, Insights, TopicOptions},
};

fn demo_receive() {
    let tether_agent = TetherAgentOptionsBuilder::new("demoReceive")
        .build()
        .expect("failed to init/connect Tether Agent");

    let _input_plug = PlugOptionsBuilder::create_input("dummyData")
        .build(&tether_agent)
        .expect("failed to create input plug");

    let mut count = 0;

    loop {
        while let Some((_plug_name, message)) = &tether_agent.check_messages() {
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

    let mut count = 0;

    let options = SendOptions {
        plug_name: "dummyData".into(),
        plug_topic: None,
        custom_message: None,
    };

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        count += 1;
        println!("SEND: sending message #{}", count);
        send(&options, &tether_agent).expect("failed to send");
    }
}

pub fn demo_topics() {
    let tether_agent = TetherAgentOptionsBuilder::new("demoTopics")
        .build()
        .expect("failed to init/connect Tether Agent");

    let options = TopicOptions {
        subscribe_topic: "#".into(),
    };

    let mut insights = Insights::new();

    let input = subscribe(&options, &tether_agent).expect("failed to subscribe");

    loop {
        if insights.check_for_updates(&input, &tether_agent) {
            println!("Insights update: {:#?}", insights);
            let Insights {
                topics,
                roles,
                ids,
                plugs,
            } = &insights;
            println!(
                "counted {} topics, {} roles, {} ids and {} plugs",
                topics.len(),
                roles.len(),
                ids.len(),
                plugs.len()
            );
        }
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
    handles.push(std::thread::spawn(move || {
        demo_topics();
    }));

    for handle in handles {
        handle.join().expect("failed to join handle");
    }
}
