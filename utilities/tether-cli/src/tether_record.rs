use std::{
    fs::File,
    io::{LineWriter, Write},
    process::exit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, SystemTime},
};

use clap::Args;
use log::{debug, info, warn};
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

use crate::{
    defaults,
    tether_playback::{SimulationMessage, SimulationRow},
    Cli,
};

#[derive(Args)]
pub struct RecordOptions {
    /// Specify the full path for the recording file; overrides any other file args
    #[arg(long = "file.overridePath")]
    file_override_path: Option<String>,

    /// Base path for recording file
    #[arg(long = "file.path", default_value_t=String::from("./"))]
    file_base_path: String,

    /// Base name for recording file, excluding timestamp and .json extension
    #[arg(long = "file.name", default_value_t=String::from("recording"))]
    file_base_name: String,

    /// Flag to disable appending timestamp onto recording file name
    #[arg(long = "file.noTimestamp")]
    file_no_timestamp: bool,

    /// Topic to subscribe; by default we recording everything
    #[arg(long = "topic", default_value_t=String::from("#"))]
    subscribe_topic: String,
}

pub fn record(cli: &Cli, options: &RecordOptions) {
    info!("Tether Record Utility");
    let tether_agent = TetherAgentOptionsBuilder::new(defaults::AGENT_ROLE)
        .host(&cli.tether_host)
        .port(cli.tether_port)
        .username(&cli.tether_username)
        .password(&cli.tether_password)
        .build()
        .expect("failed to connect Tether");

    let _input = PlugOptionsBuilder::create_input("all")
        .topic(&options.subscribe_topic)
        .build(&tether_agent)
        .expect("failed to create input plug");

    let file_path = match &options.file_override_path {
        Some(override_path) => String::from(override_path),
        None => {
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();
            format!(
                "{}{}-{}.json",
                options.file_base_path, options.file_base_name, timestamp
            )
        }
    };

    info!("Writing recorded data to {} ...", &file_path);

    let file = File::create(&file_path).expect("failed to create file");
    let mut file = LineWriter::new(file);

    let buf = b"[";
    file.write_all(buf).expect("failed to write first line");

    warn!("This utility runs until you press Ctrl+C ...");

    let mut previous_message_time = SystemTime::now();

    let mut count = 0;
    let should_stop = Arc::new(AtomicBool::new(false));
    let should_stop_clone = Arc::clone(&should_stop);

    ctrlc::set_handler(move || {
        should_stop_clone.store(true, Ordering::Relaxed);
        warn!("received Ctrl+C!");
    })
    .expect("Error setting Ctrl-C handler");

    loop {
        if should_stop.load(Ordering::Relaxed) {
            info!("Stopping; end file cleanly, wait then exit");
            file.write_all(b"\n]")
                .expect("failed to close JSON file properly");
            file.flush().unwrap();
            std::thread::sleep(Duration::from_secs(2));
            debug!("Exit now");
            exit(0);
        } else {
            let mut did_work = false;
            while let Some((_plug_name, message)) = tether_agent.check_messages() {
                did_work = true;
                let delta_time = previous_message_time.elapsed().unwrap_or_default();
                previous_message_time = SystemTime::now();
                debug!("Received message on topic \"{}\"", message.topic());
                let bytes = message.payload();
                let row = SimulationRow {
                    topic: message.topic().into(),
                    message: SimulationMessage {
                        r#type: "Buffer".into(),
                        data: bytes.to_vec(),
                    },
                    delta_time: delta_time.as_millis() as u64,
                };

                if count == 0 {
                    file.write_all(b"\n").unwrap(); // line break only
                } else {
                    file.write_all(b",\n").unwrap(); // comma, line break
                }

                let json =
                    serde_json::to_string(&row).expect("failed to convert to stringified JSON");
                file.write_all(json.as_bytes())
                    .expect("failed to write new entry");

                file.flush().unwrap();

                count += 1;

                debug!("Wrote {} rows", count);
            }
            if !did_work {
                std::thread::sleep(std::time::Duration::from_micros(100)); //0.1 ms
            }
        }
    }
}
