use log::{debug, error, info, warn};
use rmp_serde::to_vec_named;
use rumqttc::{Client, Connection, Event, Incoming, MqttOptions, Outgoing, Packet, Publish, QoS};
use serde::Serialize;
use std::{sync::mpsc, thread, time::Duration};

use crate::{
    three_part_topic::ThreePartTopic, PlugDefinition, PlugDefinitionCommon, TetherOrCustomTopic,
};

const TIMEOUT_SECONDS: u64 = 10;
const DEFAULT_USERNAME: &str = "tether";
const DEFAULT_PASSWORD: &str = "sp_ceB0ss!";

pub struct TetherAgent {
    role: String,
    id: String,
    client: Client,
    broker_uri: String,
    message_sender: mpsc::Sender<(TetherOrCustomTopic, Vec<u8>)>,
    message_receiver: mpsc::Receiver<(TetherOrCustomTopic, Vec<u8>)>,
}

#[derive(Clone)]
pub struct TetherAgentOptionsBuilder {
    role: String,
    id: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    auto_connect: bool,
}

impl TetherAgentOptionsBuilder {
    /// Initialise Tether Options struct with default options; call other methods to customise.
    /// Call `build()` to get the actual TetherAgent instance (and connect automatically, by default)
    pub fn new(role: &str) -> Self {
        TetherAgentOptionsBuilder {
            role: String::from(role),
            id: None,
            host: None,
            port: None,
            username: None,
            password: None,
            auto_connect: true,
        }
    }

    /// Provide Some(value) to override or None to use default
    pub fn id(mut self, id: Option<&str>) -> Self {
        self.id = id.map(|x| x.into());
        self
    }

    /// Provide Some(value) to override or None to use default
    pub fn host(mut self, host: Option<&str>) -> Self {
        self.host = host.map(|x| x.into());
        self
    }

    pub fn port(mut self, port: Option<u16>) -> Self {
        self.port = port;
        self
    }

    /// Provide Some(value) to override or None to use default
    pub fn username(mut self, username: Option<&str>) -> Self {
        self.username = username.map(|x| x.into());
        self
    }

    /// Provide Some(value) to override or None to use default
    pub fn password(mut self, password: Option<&str>) -> Self {
        self.password = password.map(|x| x.into());
        self
    }

    pub fn auto_connect(mut self, should_auto_connect: bool) -> Self {
        self.auto_connect = should_auto_connect;
        self
    }

    pub fn build(self) -> anyhow::Result<TetherAgent> {
        let broker_host = self.host.clone().unwrap_or("localhost".into());
        let broker_port = self.port.unwrap_or(1883);

        let broker_uri = format!("tcp://{broker_host}:{broker_port}");

        info!("Broker at {}", &broker_uri);

        // let create_opts = mqtt::CreateOptionsBuilder::new()
        //     .server_uri(broker_uri.clone())
        //     .client_id("")
        //     .finalize();

        let mqttoptions = MqttOptions::new("rumqtt-sync", "localhost", 1883)
            .set_credentials("tether", "sp_ceB0ss!")
            .set_keep_alive(Duration::from_secs(5))
            .to_owned();

        // Create the client connection
        let (client, connection) = Client::new(mqttoptions, 10);

        // Initialize the consumer before connecting
        // let receiver = client.start_consuming();

        let (message_sender, message_receiver) = mpsc::channel::<(TetherOrCustomTopic, Vec<u8>)>();

        let mut agent = TetherAgent {
            role: self.role.clone(),
            id: self.id.clone().unwrap_or("any".into()),
            client,
            broker_uri,
            message_sender,
            message_receiver,
        };

        if self.auto_connect {
            match agent.connect() {
                Ok(()) => Ok(agent),
                Err(e) => Err(e.into()),
            }
        } else {
            warn!("Auto-connect disabled; you must call .connect explicitly");
            Ok(agent)
        }
    }
}

impl TetherAgent {
    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }

    // pub fn connection(&self) -> &Connection {
    //     &self.connection
    // }

    pub fn is_connected(&self) -> bool {
        // self.client.is_connected()
        true
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the Agent Role, ID (group) and Broker URI
    pub fn description(&self) -> (&str, &str, &str) {
        (&self.role, &self.id, &self.broker_uri)
    }

    /// Return the URI (protocol, IP address, port, path) that
    /// was used to connect to the MQTT broker
    pub fn broker_uri(&self) -> &str {
        &self.broker_uri
    }

    pub fn set_role(&mut self, role: &str) {
        self.role = role.into();
    }

    pub fn set_id(&mut self, id: &str) {
        self.id = id.into();
    }

    pub fn connect(&mut self) -> anyhow::Result<()> {
        // if self.client.is_connected() {
        //     warn!("Was already connected. First disconnect...");
        //     self.client
        //         .disconnect(None)
        //         .expect("...Failed to disconnect");
        //     info!("...Disconnected");
        // }

        // let conn_opts = mqtt::ConnectOptionsBuilder::new()
        //     .server_uris(&[self.broker_uri.clone()])
        //     .user_name(&self.username)
        //     .password(&self.password)
        //     .connect_timeout(Duration::from_secs(TIMEOUT_SECONDS))
        //     .keep_alive_interval(Duration::from_secs(TIMEOUT_SECONDS))
        //     // .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
        //     .clean_session(true)
        //     .finalize();

        // Make the connection to the broker
        info!(
            "Make new connection to the MQTT server at {}...",
            &self.broker_uri
        );

        let mqtt_options =
            MqttOptions::new(format!("tether-rumqtt-{}", self.role), "localhost", 1883)
                .set_credentials("tether", "sp_ceB0ss!")
                .set_keep_alive(Duration::from_secs(1))
                .to_owned();

        // Create the client connection
        let (client, mut connection) = Client::new(mqtt_options, 10);

        let tx = self.message_sender.clone();

        thread::spawn(move || {
            for event in connection.iter() {
                match event {
                    Ok(e) => match e {
                        Event::Incoming(incoming) => match incoming {
                            Packet::ConnAck(_) => {
                                info!("ConnAck received!");
                            }
                            Packet::Publish(p) => {
                                debug!("Incoming Publish packet (message received), {:?}", &p);
                                let topic = p.topic;
                                let payload: Vec<u8> = p.payload.into();
                                if let Ok(t) = ThreePartTopic::try_from(topic.as_str()) {
                                    tx.send((TetherOrCustomTopic::Tether(t), payload)).expect(
                                        "failed to push message from thread; three-part-topic OK",
                                    );
                                } else {
                                    warn!("Could not pass Three Part Topic from \"{}\"", &topic);
                                    tx.send((TetherOrCustomTopic::Custom(topic), payload))
                                        .expect("failed to push message from threadl custom topic");
                                }
                            }
                            _ => debug!("Ignore all others for now, {:?}", incoming),
                        },
                        Event::Outgoing(outgoing) => {
                            debug!("Ignore outgoing events, for now, {:?}", outgoing)
                        }
                    },
                    Err(e) => {
                        error!("Connection Error: {:?}", e);
                    }
                }
            }
        });

        self.client = client;

        // match self.client.connect(conn_opts) {
        //     Ok(res) => {
        //         info!("Connected OK: {res:?}");
        //         Ok(())
        //     }
        //     Err(e) => {
        //         error!("Error connecting to the broker: {e:?}");
        //         // self.client.stop_consuming();
        //         // self.client.disconnect(None).expect("failed to disconnect");
        //         Err(e)
        //     }
        // }
        Ok(())
    }

    /// If a message is waiting return ThreePartTopic, Message (String, Message)
    /// Messages received on topics that are not parseable as Tether Three Part Topics will be returned with
    /// the complete Topic string instead
    pub fn check_messages(&mut self) -> Option<(TetherOrCustomTopic, Vec<u8>)> {
        if let Ok(message) = self.message_receiver.try_recv() {
            debug!("Message ready on queue");
            Some(message)
        } else {
            None
        }
        // if let Some(message) = self.connection.try_recv().iter().find_map(|| m)) {
        //     if let Ok(t) = ThreePartTopic::try_from(message.topic()) {
        //         Some((TetherOrCustomTopic::Tether(t), message))
        //     } else {
        //         warn!(
        //             "Could not pass Three Part Topic from \"{}\"",
        //             message.topic()
        //         );
        //         Some((
        //             TetherOrCustomTopic::Custom(String::from(message.topic())),
        //             message,
        //         ))
        //     }
        // } else {
        //     None
        // }
        // if let Ok(res) = self.connection.try_recv() {
        //     match res {
        //         Ok(e) => match e {
        //             Event::Incoming(i) => match i {
        //                 Packet::Publish(p) => {
        //                     let Publish { topic, payload, .. } = p;
        //                     Some((
        //                         TetherOrCustomTopic::Custom(topic.to_string()),
        //                         payload.into(),
        //                     ))
        //                 }
        //                 _ => None,
        //             },
        //             Event::Outgoing(_) => None,
        //         },
        //         Err(e) => {
        //             error!("Got error {}", e);
        //             None
        //         }
        //     }
        // } else {
        //     None
        // }
    }

    /// Given a plug definition and a raw (u8 buffer) payload, generate a message
    /// on an appropriate topic and with the QOS specified in the Plug Definition
    pub fn publish(
        &mut self,
        plug_definition: &PlugDefinition,
        payload: Option<&[u8]>,
    ) -> anyhow::Result<()> {
        match plug_definition {
            PlugDefinition::InputPlug(_) => {
                panic!("You cannot publish using an Input Plug")
            }
            PlugDefinition::OutputPlug(output_plug_definition) => {
                let topic = output_plug_definition.topic_str();
                let qos = match output_plug_definition.qos() {
                    0 => QoS::AtMostOnce,
                    1 => QoS::AtLeastOnce,
                    2 => QoS::ExactlyOnce,
                    _ => QoS::AtMostOnce,
                };
                // let message = MessageBuilder::new()
                //     .topic(topic)
                //     .payload(payload.unwrap_or(&[]))
                //     .retained(output_plug_definition.retain())
                //     .qos(qos)
                //     .finalize();
                // if let Err(e) = self.client.publish(message) {
                //     error!("Error publishing: {:?}", e);
                //     Err(e.into())
                // } else {
                //     Ok(())
                // }
                match self.client.publish(
                    topic,
                    qos,
                    output_plug_definition.retain(),
                    payload.unwrap_or_default(),
                ) {
                    Ok(_) => {
                        debug!("Publish OK on topic {}", topic);
                        Ok(())
                    }
                    Err(e) => Err(e.into()),
                }
            }
        }
    }

    /// Similar to `publish` but serializes the data automatically before sending
    pub fn encode_and_publish<T: Serialize>(
        &mut self,
        plug_definition: &PlugDefinition,
        data: T,
    ) -> anyhow::Result<()> {
        match to_vec_named(&data) {
            Ok(payload) => self.publish(plug_definition, Some(&payload)),
            Err(e) => {
                error!("Failed to encode: {e:?}");
                Err(e.into())
            }
        }
    }

    pub fn publish_raw(
        &mut self,
        topic: &str,
        payload: &[u8],
        qos: Option<i32>,
        retained: Option<bool>,
    ) -> anyhow::Result<()> {
        // let message = MessageBuilder::new()
        //     .topic(topic)
        //     .payload(payload)
        //     .retained(retained.unwrap_or(false))
        //     .qos(qos.unwrap_or(1))
        //     .finalize();
        let qos = match qos.unwrap_or(1) {
            0 => QoS::AtMostOnce,
            1 => QoS::AtLeastOnce,
            2 => QoS::ExactlyOnce,
            _ => QoS::AtMostOnce,
        };
        if let Err(e) = self
            .client
            .publish(topic, qos, retained.unwrap_or(false), payload)
        {
            error!("Error publishing: {:?}", e);
            Err(e.into())
        } else {
            Ok(())
        }
    }
}
