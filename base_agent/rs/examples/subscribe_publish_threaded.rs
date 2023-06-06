//! # Publish and Subscribe on separate threads
//!
//! Demonstrates checking for messages on one (spawned) thread while publishing messages on another
//! thread (in this, case the main thread). The main thread loops exactly `publish_count_target times`,
//! while the spawned thread - the one checking for messages - loops "inifitely".
//!
//! You may notice that the CHECKING LOOP occasionally throws an error from try_lock: this is
//! because both threads may attempt to access (lock) the reference to the Tether Agent at the same
//! time. This is **not** a problem because the CHECKING LOOP will simply find another opportunity to
//! check the messages again later.
//!
//! When the program ends, you should see `publish_count_target times` messages published and the same
//! number received, proving that nothing was "lost" between the threads and no deadlock situations
//! occurred.
//!

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use tether_agent::TetherAgent;

fn main() {
    let check_interval = 0.01;
    let publish_count_target = 100;
    let publish_interval = 0.1;

    println!("Rust Tether Agent threaded publish-while-consuming example");

    let agent = Arc::new(Mutex::new(TetherAgent::new(
        "RustDemoAgent",
        Some("example"),
        None,
    )));

    let mut output_plug = None;

    // Here we call .lock() because it is OK to block while "setting up", connecting
    if let Ok(a) = agent.lock() {
        a.connect(None, None).expect("failed to connect");
        a.create_input_plug("one", None, None)
            .expect("failed to create Input Plug");
        let plug = a
            .create_output_plug("one", None, None)
            .expect("failed to create Output Plug");
        output_plug = Some(plug);
    } else {
        panic!("Error setting up Tether Agent!");
    }

    let receiver_agent = Arc::clone(&agent);
    thread::spawn(move || {
        println!("Checking messages every {check_interval}s...");

        let mut i = 0;
        let mut count_messages_received = 0;

        /*
         Infinite loop. But because we never join the threads, this thread will terminate
         as soon as the main thread does.
        */
        loop {
            i += 1;
            println!("CHECKING LOOP: Checking messages attempt #{i}...");

            /*
              Here we call try_lock() because we do not want to block
              if the Agent is currently locked by another thread.
              Just print a message, wait and try again later.
            */
            match receiver_agent.try_lock() {
                Ok(a) => {
                    if let Some((topic, _message)) = a.check_messages() {
                        count_messages_received += 1;
                        println!("<<<<<<<< CHECKING LOOP: Received a message on topic {topic}; Now has {count_messages_received} messages");
                    }
                }
                Err(e) => {
                    println!("CHECKING LOOP: Failed to acquire lock: {}", e);
                }
            }
            thread::sleep(Duration::from_secs_f32(check_interval));
        }
    });

    let sending_agent = Arc::clone(&agent);
    println!(
        "Sending a message, every {}s, exactly {}x times...",
        publish_interval, publish_count_target
    );
    let mut count_messages_sent = 0;
    for i in 1..=publish_count_target {
        println!("MAIN THREAD LOOP: Send attempt #{i}");
        /*
          In this particular case, lock() is preferable to try_lock() because
          we are not doing anything else on this thread. Waiting (blocking)
          to acquire the lock
          is fine; the other thread will let it go soon.
        */
        match sending_agent.lock() {
            Ok(a) => {
                count_messages_sent += 1;
                if let Some(plug) = &output_plug {
                    a.publish(plug, Some(&[0])).expect("Failed to publish");
                    println!(">>>>>>>> MAIN THREAD LOOP: sent {count_messages_sent} messages");
                }
            }
            Err(e) => {
                panic!("MAIN THREAD LOOP: Failed to acquire lock: {}", e);
            }
        }
        thread::sleep(Duration::from_secs_f32(publish_interval));
    }
}
