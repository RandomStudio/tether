use std::{
    fs::File,
    io::{LineWriter, Write},
    sync::mpsc,
    time::{Duration, SystemTime},
};

use clap::Args;
use log::{debug, info, warn};
use tether_agent::{ChannelOptionsBuilder, TetherAgent};

use crate::tether_playback::{SimulationMessage, SimulationRow};

#[derive(Args, Clone)]
pub struct RecordOptions {
    /// Specify the full path for the recording file; overrides any other file args
    pub file_override_path: Option<String>,

    /// Base path for recording file
    #[arg(long = "file.path", default_value_t=String::from("./"))]
    pub file_base_path: String,

    /// Base name for recording file, excluding timestamp and .json extension
    #[arg(long = "file.name", default_value_t=String::from("recording"))]
    pub file_base_name: String,

    /// Flag to disable appending timestamp onto recording file name
    #[arg(long = "file.noTimestamp")]
    pub file_no_timestamp: bool,

    /// Topic to subscribe; by default we recording everything
    #[arg(long = "topic", default_value_t=String::from("#"))]
    pub topic: String,

    /// Flag to disable zero-ing of the first entry's deltaTime; using this
    /// flag will count time from launch, not first message received
    #[arg(long = "timing.nonzeroStart")]
    pub timing_nonzero_start: bool,

    /// Time (in seconds) to delay writing anything to disk, even if messages are
    /// received
    #[arg(long = "timing.delay")]
    pub timing_delay: Option<f32>,

    /// Duration (in seconds) to stop recording even if Ctrl+C has not been encountered
    /// yet
    #[arg(long = "timing.duration")]
    pub timing_duration: Option<f32>,

    /// Flag to disable registration of Ctrl+C handler - this is usually necessary
    /// when using the utility programmatically (i.e. not via CLI)
    #[arg(long = "ignoreCtrlC")]
    pub ignore_ctrl_c: bool,
}

impl Default for RecordOptions {
    fn default() -> Self {
        RecordOptions {
            file_override_path: None,
            file_base_path: "./".into(),
            file_base_name: "recording".into(),
            file_no_timestamp: false,
            topic: "#".into(),
            timing_nonzero_start: false,
            timing_delay: None,
            timing_duration: None,
            ignore_ctrl_c: false,
        }
    }
}

pub struct TetherRecordUtil {
    stop_request_tx: mpsc::Sender<bool>,
    stop_request_rx: mpsc::Receiver<bool>,
    options: RecordOptions,
}

impl TetherRecordUtil {
    pub fn new(options: RecordOptions) -> Self {
        info!("Tether Record Utility: initialise");

        let (tx, rx) = mpsc::channel();

        TetherRecordUtil {
            stop_request_tx: tx,
            stop_request_rx: rx,
            options,
        }
    }

    pub fn get_stop_tx(&self) -> mpsc::Sender<bool> {
        self.stop_request_tx.clone()
    }
    pub fn start_recording(&self, tether_agent: &mut TetherAgent) {
        info!("Tether Record Utility: start recording");

        // The channel definition is never actually used, since we do no matching from topics to channel!
        // But we must set it up so that subscription happens.
        let _channel_def = ChannelOptionsBuilder::create_receiver("all")
            .topic(Some(self.options.topic.clone()).as_deref()) // TODO: should be possible to build TPT
            .build(tether_agent)
            .expect("failed to create Channel Receiver");

        let file_path = match &self.options.file_override_path {
            Some(override_path) => String::from(override_path),
            None => {
                if self.options.file_no_timestamp {
                    format!(
                        "{}{}.json",
                        self.options.file_base_path, self.options.file_base_name
                    )
                } else {
                    let timestamp = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or(Duration::from_secs(0))
                        .as_secs();
                    format!(
                        "{}{}-{}.json",
                        self.options.file_base_path, self.options.file_base_name, timestamp
                    )
                }
            }
        };

        info!("Writing recorded data to \"{}\" ...", &file_path);

        let file = File::create(&file_path).expect("failed to create file");
        let mut file = LineWriter::new(file);

        let buf = b"[";
        file.write_all(buf).expect("failed to write first line");

        let max_duration = match self.options.timing_duration {
            Some(dur) => {
                warn!(
                    "Max duration was set to {}s; Ctr+C to stop before that point ...",
                    dur
                );
                Some(Duration::from_secs_f32(dur))
            }
            None => {
                warn!("No duration provided; recording runs until you press Ctrl+C ...");
                None
            }
        };

        let start_delay = match self.options.timing_delay {
            Some(dur) => {
                warn!("Recording will only start after {}s", dur);
                Some(Duration::from_secs_f32(dur))
            }
            None => {
                debug!("No start delay set");
                None
            }
        };

        let start_application_time = SystemTime::now();
        let mut first_message_time = SystemTime::now();
        let mut previous_message_time = SystemTime::now();

        let mut count: i128 = 0;

        let stop_from_key = self.stop_request_tx.clone();
        let stop_from_timer = self.stop_request_tx.clone();
        // let stop_tx_clone = stop_tx.clone();

        // let should_stop = Arc::new(AtomicBool::new(false));
        // let should_stop_clone = Arc::clone(&should_stop);

        if !self.options.ignore_ctrl_c {
            ctrlc::set_handler(move || {
                // should_stop_clone.store(true, Ordering::Relaxed);
                stop_from_key
                    .send(true)
                    .expect("failed to send stop from key");
                warn!("received Ctrl+C!");
            })
            .expect("Error setting Ctrl-C handler");
        } else {
            warn!(
                "No Ctrl+C handler set; you may need to kill this process manually, PID: {}",
                std::process::id()
            );
        }

        let mut finished = false;

        while !finished {
            if let Some(delay) = start_delay {
                if let Ok(elapsed) = start_application_time.elapsed() {
                    if elapsed < delay {
                        continue;
                    }
                }
            }

            if let Some(dur) = max_duration {
                if let Ok(elapsed) = first_message_time.elapsed() {
                    if elapsed > dur {
                        if count == 0 && !self.options.timing_nonzero_start {
                            debug!("Ignore duration; nothing received yet")
                        } else {
                            warn!(
                                "Exceeded the max duration specified ({}s); will stop now...",
                                dur.as_secs_f32()
                            );
                            // should_stop.store(true, Ordering::Relaxed);
                            stop_from_timer
                                .send(true)
                                .expect("failed to send stop from timer");
                        }
                    }
                }
            }
            if let Ok(_should_stop) = self.stop_request_rx.try_recv() {
                info!(
                    "Stopping after {} entries written to disk; end file cleanly, wait then exit",
                    count
                );
                file.write_all(b"\n]")
                    .expect("failed to close JSON file properly");
                file.flush().unwrap();
                std::thread::sleep(Duration::from_secs(2));
                debug!("Exit now");
                // exit(0);
                finished = true;
            } else {
                let mut did_work = false;
                while let Some((topic, payload)) = tether_agent.check_messages() {
                    did_work = true;

                    let delta_time = if count == 0 && !self.options.timing_nonzero_start {
                        first_message_time = SystemTime::now();
                        Duration::ZERO
                    } else {
                        previous_message_time.elapsed().unwrap_or_default()
                    };
                    previous_message_time = SystemTime::now();

                    let full_topic_string = topic.full_topic_string();

                    debug!("Received message on topic \"{}\"", &full_topic_string);
                    let row = SimulationRow {
                        topic: full_topic_string,
                        message: SimulationMessage {
                            r#type: "Buffer".into(),
                            data: payload.to_vec(),
                        },
                        delta_time: delta_time.as_millis() as u64,
                    };

                    if count == 0 {
                        file.write_all(b"\n").unwrap(); // line break only
                        info!("First message written to disk");
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
}
