use rumqttc::{AsyncClient, MqttOptions, QoS};
// use std::error::Error;
use std::time::Duration;
use tokio::{task, time};

#[tokio::main]
async fn main() -> Result<(), ()> {
    let mqttoptions = MqttOptions::new("rumqtt-async", "localhost", 1883)
        .set_credentials("tether", "sp_ceB0ss!")
        .set_keep_alive(Duration::from_secs(5))
        .to_owned();
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client
        .subscribe("hello/rumqtt", QoS::AtMostOnce)
        .await
        .unwrap();

    task::spawn(async move {
        for i in 0..10 {
            client
                .publish("hello/rumqtt", QoS::AtLeastOnce, false, vec![i; i as usize])
                .await
                .unwrap();
            time::sleep(Duration::from_millis(100)).await;
        }
    });

    while let Ok(notification) = eventloop.poll().await {
        println!("Received = {:?}", notification);
    }

    Ok(())
}
