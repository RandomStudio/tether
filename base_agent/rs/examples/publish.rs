use futures::executor::block_on;
use paho_mqtt as mqtt;
use std::{ process};

/////////////////////////////////////////////////////////////////////////////

fn main() {
    // Create the client
    let client = mqtt::AsyncClient::new("tcp://localhost:1883".to_string()).unwrap_or_else(|err| {
        println!("Error creating the client: {}", err);
        process::exit(1);
    });

    if let Err(err) = block_on(async {
        // Connect with default options and wait for it to complete or fail
        println!("Connecting to the MQTT server");
        client.connect(mqtt::ConnectOptionsBuilder::new().user_name("tether").password("sp_ceB0ss!").finalize()).await?;

        // Create a message and publish it
        println!("Publishing a message on the topic 'test'");
        let msg = mqtt::Message::new("test", "Hello Rust MQTT world!", mqtt::QOS_1);
        client.publish(msg).await?;

        // Disconnect from the broker
        println!("Disconnecting");
        client.disconnect(None).await?;

        Ok::<(), mqtt::Error>(())
    }) {
        eprintln!("{}", err);
    }
}