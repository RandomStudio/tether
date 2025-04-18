use anyhow::anyhow;
use log::{debug, error, info, trace, warn};
use rmp_serde::to_vec_named;
use rumqttc::tokio_rustls::rustls::ClientConfig;
use rumqttc::{Client, Event, MqttOptions, Packet, QoS, Transport};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::{sync::mpsc, thread, time::Duration};
use uuid::Uuid;

pub mod builder;

pub use builder::*;

use crate::definitions::receiver_def_builder::ChannelReceiverDefBuilder;
use crate::definitions::sender_def_builder::ChannelSenderDefBuilder;
use crate::definitions::ChannelDefBuilder;
use crate::definitions::{ChannelDef, ChannelReceiverDef, ChannelSenderDef};
use crate::receiver::ChannelReceiver;
use crate::sender::ChannelSender;
use crate::tether_compliant_topic::{TetherCompliantTopic, TetherOrCustomTopic};

const TIMEOUT_SECONDS: u64 = 3;

/**
A Tether Agent struct encapsulates everything required to set up a single
"Agent" as part of your Tether-based system. The only thing absolutely required is
a "role" - everything else is optional and sensible defaults will be used when
not explicitly specified.

By default, the Agent will connect (automatically) to an MQTT Broker on localhost:1883

It will **not** have an ID, and therefore publishing/subscribing topics will not append anything
this into the topic string when ChannelSender and ChannelReceiver instances are created using
this Tether Agent instance, unless explicitly provided on creation.

Note that you should typically not construct a new TetherAgent instance yourself; rather
use the provided TetherAgentBuilder to specify any options you might need, and call
.build to get a well-configured TetherAgent.
*/
pub struct TetherAgent {
    role: String,
    id: Option<String>,
    host: String,
    port: u16,
    protocol: String,
    username: String,
    password: String,
    base_path: String,
    mqtt_client_id: Option<String>,
    pub(crate) client: Option<Client>,
    message_sender: mpsc::Sender<(TetherOrCustomTopic, Vec<u8>)>,
    pub message_receiver: mpsc::Receiver<(TetherOrCustomTopic, Vec<u8>)>,
    is_connected: Arc<Mutex<bool>>,
    auto_connect_enabled: bool,
}

impl<'a> TetherAgent {
    /// The simplest way to create a ChannelSender.
    ///
    /// You provide only a Channel Name;
    /// configuration derived from your Tether Agent instance is used to construct
    /// the appropriate publishing topics.
    pub fn create_sender<T: Serialize>(&self, name: &str) -> ChannelSender<T> {
        ChannelSender::new(self, ChannelSenderDefBuilder::new(name).build(self))
    }

    /// Create a ChannelSender instance using a ChannelSenderDefinition already constructed
    /// elsewhere.
    pub fn create_sender_with_def<T: Serialize>(
        &self,
        definition: ChannelSenderDef,
    ) -> ChannelSender<T> {
        ChannelSender::new(self, definition)
    }

    /// The simplest way to create a Channel Receiver.
    ///
    /// You provide only a Channel Name;
    /// configuration derived from your Tether Agent instance is used to construct
    /// the appropriate subscribing topics.
    ///
    /// The actual subscription is also initiated automatically.
    pub fn create_receiver<T: Deserialize<'a>>(
        &'a self,
        name: &str,
    ) -> anyhow::Result<ChannelReceiver<'a, T>> {
        ChannelReceiver::new(self, ChannelReceiverDefBuilder::new(name).build(self))
    }

    /// Create a ChannelReceiver instance using a ChannelReceiverDefinition already constructed
    /// elsewhere.
    pub fn create_receiver_with_def<T: Deserialize<'a>>(
        &'a self,
        definition: ChannelReceiverDef,
    ) -> anyhow::Result<ChannelReceiver<'a, T>> {
        ChannelReceiver::new(self, definition)
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    pub fn auto_connect_enabled(&self) -> bool {
        self.auto_connect_enabled
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Returns the Agent Role, ID (group), Broker URI
    pub fn description(&self) -> (String, String, String) {
        (
            String::from(&self.role),
            match &self.id {
                Some(id) => String::from(id),
                None => String::from("any"),
            },
            self.broker_uri(),
        )
    }

    /// Return the URI (protocol, IP address, port, path) that
    /// was used to connect to the MQTT broker
    pub fn broker_uri(&self) -> String {
        format!(
            "{}://{}:{}{}",
            &self.protocol, self.host, self.port, self.base_path
        )
    }

    /// Change the role, even if it was set before. Be careful _when_ you call this,
    /// as it could affect any new Channel Senders/Receivers created after that point.
    pub fn set_role(&mut self, role: &str) {
        self.role = role.into();
    }

    /// Change the ID, even if it was set (or left empty) before.
    /// Be careful _when_ you call this,
    /// as it could affect any new Channel Senders/Receivers created after that point.
    pub fn set_id(&mut self, id: &str) {
        self.id = Some(id.into());
    }

    /// Use this function yourself **only if you explicitly disallowed auto connection**.
    /// Otherwise, this function is called automatically as part of the `.build` process.
    ///
    /// This function spawns a separate thread for polling the MQTT broker. Any events
    /// and messages are relayed via mpsc channels internally; for example, you will call
    /// `.check_messages()` to see if any messages were received and are waiting to be parsed.
    pub fn connect(&mut self) -> anyhow::Result<()> {
        info!(
            "Make new connection to the MQTT server at {}://{}:{}...",
            self.protocol, self.host, self.port
        );

        let mqtt_client_id = self
            .mqtt_client_id
            .clone()
            .unwrap_or(Uuid::new_v4().to_string());

        debug!("Using MQTT Client ID \"{}\"", mqtt_client_id);

        let mut mqtt_options = MqttOptions::new(mqtt_client_id.clone(), &self.host, self.port)
            .set_credentials(&self.username, &self.password)
            .set_keep_alive(Duration::from_secs(TIMEOUT_SECONDS))
            .to_owned();

        match self.protocol.as_str() {
            "mqtts" => {
                // Use rustls-native-certs to load root certificates from the operating system.
                let mut root_cert_store = rumqttc::tokio_rustls::rustls::RootCertStore::empty();
                root_cert_store.add_parsable_certificates(
                    rustls_native_certs::load_native_certs()
                        .expect("could not load platform certs"),
                );

                let client_config = ClientConfig::builder()
                    .with_root_certificates(root_cert_store)
                    .with_no_client_auth();
                mqtt_options.set_transport(Transport::tls_with_config(client_config.into()));
            }
            "wss" => {
                // If using websocket protocol, rumqttc does NOT automatically add protocol and port
                // into the URL!
                let full_host = format!(
                    "{}://{}:{}{}",
                    self.protocol, self.host, self.port, self.base_path
                );
                debug!("WSS using full host URL: {}", &full_host);
                mqtt_options = MqttOptions::new(mqtt_client_id.clone(), &full_host, self.port) // here, port is ignored anyway
                    .set_credentials(&self.username, &self.password)
                    .set_keep_alive(Duration::from_secs(TIMEOUT_SECONDS))
                    .to_owned();

                // Use rustls-native-certs to load root certificates from the operating system.
                let mut root_cert_store = rumqttc::tokio_rustls::rustls::RootCertStore::empty();
                root_cert_store.add_parsable_certificates(
                    rustls_native_certs::load_native_certs()
                        .expect("could not load platform certs"),
                );

                let client_config = ClientConfig::builder()
                    .with_root_certificates(root_cert_store)
                    .with_no_client_auth();
                mqtt_options.set_transport(Transport::wss_with_config(client_config.into()));
            }
            "ws" => {
                // If using websocket protocol, rumqttc does NOT automatically add protocol and port
                // into the URL!
                let full_host = format!(
                    "{}://{}:{}{}",
                    self.protocol, self.host, self.port, self.base_path
                );
                debug!("WS using full host URL: {}", &full_host);

                mqtt_options = MqttOptions::new(mqtt_client_id.clone(), &full_host, self.port) // here, port is ignored anyway
                    .set_credentials(&self.username, &self.password)
                    .set_keep_alive(Duration::from_secs(TIMEOUT_SECONDS))
                    .to_owned();

                mqtt_options.set_transport(Transport::Ws);
            }
            _ => {}
        };

        // Create the client connection
        let (client, mut connection) = Client::new(mqtt_options, 10);

        let message_tx = self.message_sender.clone();

        let connection_state = Arc::clone(&self.is_connected);

        thread::spawn(move || {
            for event in connection.iter() {
                match event {
                    Ok(e) => {
                        match e {
                            Event::Incoming(incoming) => match incoming {
                                Packet::ConnAck(_) => {
                                    info!("(Connected) ConnAck received!");
                                    let mut is_c =
                                        connection_state.lock().expect("failed to lock mutex");
                                    *is_c = true;
                                }
                                Packet::Publish(p) => {
                                    debug!("Incoming Publish packet (message received), {:?}", &p);
                                    let topic = p.topic;
                                    let payload: Vec<u8> = p.payload.into();
                                    match TetherCompliantTopic::try_from(topic.as_str()) {
                                        Ok(t) => {
                                            message_tx
                                            .send((TetherOrCustomTopic::Tether(t), payload))
                                            .expect(
                                            "failed to push message from thread; three-part-topic OK",
                                        );
                                        }
                                        Err(e) => {
                                            warn!(
                                                "Could not parse Three Part Topic from \"{}\": {}",
                                                &topic, e
                                            );
                                            message_tx
                                        .send((TetherOrCustomTopic::Custom(topic), payload))
                                        .expect("failed to push message from thread; custom topic");
                                        }
                                    }
                                }
                                _ => debug!("Ignore all others for now, {:?}", incoming),
                            },
                            Event::Outgoing(outgoing) => {
                                debug!("Ignore outgoing events, for now, {:?}", outgoing)
                            }
                        }
                    }
                    Err(e) => {
                        error!("Connection Error: {:?}", e);
                        std::thread::sleep(Duration::from_secs(1));
                        // connection_status_tx
                        //     .send(Err(anyhow!("MQTT Connection error")))
                        //     .expect("failed to push error message from thread");
                    }
                }
            }
        });

        let mut is_ready = false;

        while !is_ready {
            debug!("Check whether connected...");
            std::thread::sleep(Duration::from_millis(1));
            trace!("Is ready? {}", is_ready);
            let get_state = *self.is_connected.lock().expect("failed to lock mutex");
            if get_state {
                info!("Connection status confirmed");
                is_ready = true;
            } else {
                debug!("Not connected yet...");
            }
        }

        self.client = Some(client);

        Ok(())
    }

    /// If a message is waiting to be parsed by your application,
    /// this function will return Topic, Message, i.e. `(TetherOrCustomTopic, Message)`
    ///
    /// Messages received on topics that are not parseable as Tether Three Part Topics will be returned with
    /// the complete Topic string instead
    pub fn check_messages(&self) -> Option<(TetherOrCustomTopic, Vec<u8>)> {
        // if let Ok(e) = self.connection_status_receiver.try_recv() {
        //     panic!("check_messages received error: {}", e);
        // }
        if let Ok(message) = self.message_receiver.try_recv() {
            debug!("Message ready on queue");
            Some(message)
        } else {
            None
        }
    }

    /// Typically called via the Channel Sender itself.
    ///
    /// This function serializes the data (using Serde/MessagePack) automatically before publishing.
    ///
    /// Given a Channel Sender and serializeable data payload, publishes a message
    /// using an appropriate topic and with the QOS specified in the Channel Definition.
    ///
    /// Note that this function requires that the data payload be the same type <T> as
    /// the Channel Sender, so it will return an Error if the types do not match.
    pub fn send<T: Serialize>(
        &self,
        channel_sender: &ChannelSender<T>,
        data: &T,
    ) -> anyhow::Result<()> {
        match to_vec_named(&data) {
            Ok(payload) => self.send_raw(channel_sender.definition(), Some(&payload)),
            Err(e) => {
                error!("Failed to encode: {e:?}");
                Err(e.into())
            }
        }
    }

    /// Typically called via the Channel Sender itself.
    ///
    /// Unlike .send, this function does NOT serialize the data before publishing. It therefore
    /// does no type checking of the payload.
    ///
    /// Given a Channel Sender and a raw (u8 buffer) payload, publishes a message
    /// using an appropriate topic and with the QOS specified in the Channel Definition
    pub fn send_raw(
        &self,
        channel_definition: &ChannelSenderDef,
        payload: Option<&[u8]>,
    ) -> anyhow::Result<()> {
        let topic = channel_definition.generated_topic();
        let qos = match channel_definition.qos() {
            0 => QoS::AtMostOnce,
            1 => QoS::AtLeastOnce,
            2 => QoS::ExactlyOnce,
            _ => QoS::AtMostOnce,
        };

        if let Some(client) = &self.client {
            let res = client
                .publish(
                    topic,
                    qos,
                    channel_definition.retain(),
                    payload.unwrap_or_default(),
                )
                .map_err(anyhow::Error::msg);
            debug!("Published OK");
            res
        } else {
            Err(anyhow!("Client not ready for publish"))
        }
    }

    pub fn send_empty(&self, channel_definition: &ChannelSenderDef) -> anyhow::Result<()> {
        self.send_raw(channel_definition, None)
    }

    /// Publish an already-encoded payload using a provided
    /// **full topic string** - no need for passing a ChannelSender or
    /// ChannelSenderDefinition reference.
    ///
    /// **WARNING:** This is a back door to using MQTT directly, without any
    /// guarrantees of correctedness in a Tether-based system!
    pub fn publish_raw(
        &self,
        topic: &str,
        payload: &[u8],
        qos: Option<i32>,
        retained: Option<bool>,
    ) -> anyhow::Result<()> {
        let qos = match qos.unwrap_or(1) {
            0 => QoS::AtMostOnce,
            1 => QoS::AtLeastOnce,
            2 => QoS::ExactlyOnce,
            _ => QoS::AtMostOnce,
        };
        if let Some(client) = &self.client {
            client
                .publish(topic, qos, retained.unwrap_or_default(), payload)
                .map_err(anyhow::Error::msg)
        } else {
            Err(anyhow!("Client not ready for publish"))
        }
    }
}
