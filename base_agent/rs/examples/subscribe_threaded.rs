//! # Subscribe on one thread
//!
//! This example demonstrates checking for messages repeatedly on a separate, spawned thread.
//! If any messages are found, it will relay them to the other (main) thread via an
//! async Channel.
//!
//! To keep this example reasonably realistic, the message-checking thread loops more quickly
//! (1ms interval) while the main thread simulates being very busy with other things (1 second
//! interval).
//!
//! When the main thread comes to a non-empty "queue" of messages on the Receiver, it processes
//! them as quickly as possible. Once it has received at least 10 messages, it simply quits.
//!
//! **This example needs some published messages to be useful!** Try running the publish example
//! at the same time: `cargo run --example publish`

use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use tether_agent::TetherAgent;

fn main() {
    println!("Rust Tether Agent subscribe example");

    let agent = Arc::new(Mutex::new(TetherAgent::new(
        "RustDemoAgent",
        Some("example"),
        None,
    )));

    match agent.lock() {
        Ok(a) => {
            a.connect(None, None).expect("failed to connect");
            a.create_input_plug("one", None, None)
                .expect("failed to create Input Plug");
        }
        Err(e) => {
            panic!("Failed to acquire lock for Tether Agent setup: {}", e);
        }
    };

    let (tx, rx) = mpsc::channel();

    let receiver_agent = Arc::clone(&agent);
    thread::spawn(move || {
        println!("Checking messages every 1s, 10x...");

        let mut message_count = 0;
        let mut i = 0;

        loop {
            i += 1;
            // println!("#{i}: Checking messages...");
            match receiver_agent.try_lock() {
                Ok(a) => {
                    if let Some((topic, _message)) = a.check_messages() {
                        message_count += 1;
                        println!("<<<<<<<< CHECKING LOOP: Received a message on topic {topic}",);
                        tx.send(format!("received message #{message_count}"))
                            .expect("failed to send message via channel");
                    }
                }
                Err(e) => {
                    println!("Failed to acquire lock: {}", e);
                }
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    let mut main_thread_received_count = 0;

    loop {
        println!("Main thread sleep...");
        for rx in rx.try_iter() {
            main_thread_received_count += 1;
            println!(
                "<<<<<<<< MAIN THREAD: received {} (count: {})",
                rx, main_thread_received_count
            );
        }
        if main_thread_received_count >= 10 {
            println!("We're done!");
            std::process::exit(0);
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}
