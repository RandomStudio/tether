extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use rmps::{Deserializer, Serializer};

use futures::executor::block_on;
use paho_mqtt as mqtt;
use std::{ process};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Human {
    age: u32,
    name: String,
}

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

        let mut buf = Vec::new();
        let val = Human {
            age: 42,
            name: "John".into(),
        };
    
        val.serialize(&mut Serializer::new(&mut buf)).unwrap();

        let msg = mqtt::Message::new("/tetherRs/dummy/test", buf, mqtt::QOS_1);
        client.publish(msg).await?;

        // Disconnect from the broker
        println!("Disconnecting");
        client.disconnect(None).await?;

        Ok::<(), mqtt::Error>(())
    }) {
        eprintln!("{}", err);
    }
}