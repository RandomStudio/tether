use std::{thread::spawn, time::SystemTime};

use env_logger::{Builder, Env};
use log::LevelFilter;
use tether_agent::TetherAgentOptionsBuilder;
use tether_utils::{
    tether_playback::{PlaybackOptions, TetherPlaybackUtil},
    tether_receive::{receive, ReceiveOptions},
    tether_record::{RecordOptions, TetherRecordUtil},
    tether_send::{send, SendOptions},
    tether_topics::{insights::Insights, TopicOptions},
};

fn demo_receive() {
    let mut tether_agent = TetherAgentOptionsBuilder::new("demoReceive")
        .build()
        .expect("failed to init/connect Tether Agent");

    let options = ReceiveOptions::default();

    receive(
        &options,
        &mut tether_agent,
        |_channel_name, topic, decoded| {
            let contents = decoded.unwrap_or("(empty/invalid message)".into());
            println!("RECEIVE: \"{}\" :: {}", topic, contents);
        },
    )
}

fn demo_send() {
    let mut tether_agent = TetherAgentOptionsBuilder::new("demoSend")
        .build()
        .expect("failed to init/connect Tether Agent");

    let mut count = 0;

    let options = SendOptions {
        channel_name: Some("dummyData".into()),
        channel_topic: None,
        channel_id: None,
        channel_role: None,
        message_payload_json: None,
        use_dummy_data: true,
    };

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        count += 1;
        println!("SEND: sending message #{}", count);
        send(&options, &mut tether_agent).expect("failed to send");
    }
}

fn demo_topics() {
    let mut tether_agent = TetherAgentOptionsBuilder::new("demoTopics")
        .build()
        .expect("failed to init/connect Tether Agent");

    let options = TopicOptions {
        topic: "#".into(),
        sampler_interval: 1000,
        graph_enable: false,
    };

    let mut insights = Insights::new(&options, &mut tether_agent);

    loop {
        while let Some((topic, payload)) = tether_agent.check_messages() {
            if insights.update(&topic, payload) {
                println!("TOPICS: Insights update: \n{}", insights);
            }
        }
    }
}

fn demo_playback() {
    let options = PlaybackOptions {
        file_path: "./demo.json".into(),
        override_topic: None,
        loop_count: 1, // ignored anyway, in this case
        loop_infinite: true,
        ignore_ctrl_c: true, // this is important for programmatic use
        playback_speed: 1.0,
        topic_filters: None,
    };

    let tether_agent = TetherAgentOptionsBuilder::new("demoTopics")
        .build()
        .expect("failed to init/connect Tether Agent");

    let player = TetherPlaybackUtil::new(options);
    let stop_request_tx = player.get_stop_tx();

    let start_time = SystemTime::now();

    let handles = vec![
        spawn(move || {
            player.start(&tether_agent);
        }),
        spawn(move || {
            let mut time_to_end = false;
            while !time_to_end {
                if let Ok(elapsed) = start_time.elapsed() {
                    if elapsed > std::time::Duration::from_secs(3) {
                        println!("Time to stop! {}s elapsed", elapsed.as_secs());
                        stop_request_tx
                            .send(true)
                            .expect("failed to send stop request via channel");
                        time_to_end = true;
                    }
                }
            }
            println!("Playback should have stopped now; wait 1 more seconds...");
            std::thread::sleep(std::time::Duration::from_secs(1));

            println!("...Bye");
        }),
    ];
    for handle in handles {
        handle.join().expect("failed to join handle");
    }
}

fn demo_record() {
    let mut tether_agent = TetherAgentOptionsBuilder::new("demoPlayback")
        .build()
        .expect("failed to init/connect Tether Agent");

    let options = RecordOptions {
        file_override_path: None,
        file_base_path: "./".into(),
        file_base_name: "recording".into(),
        file_no_timestamp: false,
        topic: "#".into(),
        timing_nonzero_start: false,
        timing_delay: None,
        timing_duration: None,
        ignore_ctrl_c: true, // this is important for programmatic use
    };

    let recorder = TetherRecordUtil::new(options);
    let stop_request_tx = recorder.get_stop_tx();

    let start_time = SystemTime::now();
    let handles = vec![
        spawn(move || {
            let mut time_to_end = false;
            while !time_to_end {
                if let Ok(elapsed) = start_time.elapsed() {
                    if elapsed > std::time::Duration::from_secs(3) {
                        println!("Time to stop! {}s elapsed", elapsed.as_secs());
                        stop_request_tx
                            .send(true)
                            .expect("failed to send stop request via channel");
                        time_to_end = true;
                    }
                }
            }
            println!("Recording should have stopped now; wait 4 more seconds...");
            std::thread::sleep(std::time::Duration::from_secs(4));
            println!("...Bye");
        }),
        spawn(move || {
            recorder.start_recording(&mut tether_agent);
        }),
    ];

    for handle in handles {
        handle.join().expect("RECORDER: failed to join handle");
    }
}

fn main() {
    println!(
        "This example shows how the tether-utils library can be used programmatically,
    i.e. not from the CLI"
    );
    println!("Press Ctrl+C to stop");

    let mut env_builder = Builder::from_env(Env::default().default_filter_or("info"));
    env_builder.filter_module("paho_mqtt", LevelFilter::Warn);
    env_builder.init();

    let handles = vec![
        spawn(demo_receive),
        spawn(demo_send),
        spawn(demo_topics),
        spawn(|| {
            std::thread::sleep(std::time::Duration::from_secs(4));
            demo_playback();
        }),
        spawn(demo_record),
    ];

    for handle in handles {
        handle.join().expect("failed to join handle");
    }
}
