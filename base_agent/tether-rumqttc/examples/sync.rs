use rumqttc::{Client, MqttOptions, QoS};
use std::thread;
use std::time::Duration;

fn main() {
    let mqttoptions = MqttOptions::new("rumqtt-sync", "localhost", 1884)
        .set_credentials("tether", "sp_ceB0ss!")
        .set_keep_alive(Duration::from_secs(5))
        .to_owned();

    let (mut client, mut connection) = Client::new(mqttoptions, 10);
    // client.subscribe("hello/rumqtt", QoS::AtMostOnce).unwrap();
    thread::spawn(move || {
        for i in 0..10 {
            client
                .publish("hello/rumqtt", QoS::AtLeastOnce, false, vec![i; i as usize])
                .unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Iterate to poll the eventloop for connection progress
    for (i, notification) in connection.iter().enumerate() {
        println!("Notification = {:?}", notification);
    }
}
