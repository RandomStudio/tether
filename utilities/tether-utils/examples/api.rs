use std::thread::spawn;

use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};
use tether_utils::{
    tether_playback::{playback, PlaybackOptions},
    tether_receive::{receive, ReceiveOptions},
    tether_send::{send, SendOptions},
    tether_topics::{subscribe, Insights, TopicOptions},
};

fn demo_receive() {
    let tether_agent = TetherAgentOptionsBuilder::new("demoReceive")
        .build()
        .expect("failed to init/connect Tether Agent");

    let options = ReceiveOptions {
        subscribe_topic: "#".into(),
    };

    receive(&options, &tether_agent, |_plug_name, message, decoded| {
        let contents = decoded.unwrap_or("(empty/invalid message)".into());
        println!("RECEIVE: \"{}\" :: {}", message.topic(), contents);
    })
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

pub fn demo_playback() {
    let tether_agent = TetherAgentOptionsBuilder::new("demoPlayback")
        .build()
        .expect("failed to init/connect Tether Agent");

    let options = PlaybackOptions {
        file_path: "./demo.json".into(),
        override_topic: None,
        loop_count: 1, // ignored anyway
        loop_infinite: true,
    };

    playback(&options, &tether_agent);
}

fn main() {
    println!(
        "This example shows how the tether-utils library can be used programmatically, 
    i.e. not from the CLI"
    );
    println!("Press Ctrl+C to stop");

    let handles = vec![
        spawn(|| demo_receive()),
        spawn(|| demo_send()),
        spawn(|| demo_topics()),
        spawn(|| {
            std::thread::sleep(std::time::Duration::from_secs(4));
            demo_playback();
        }),
    ];

    for handle in handles {
        handle.join().expect("failed to join handle");
    }
}
