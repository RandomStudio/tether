use anyhow::anyhow;
use log::{debug, error, info, trace, warn};
use rmp_serde::to_vec_named;
use rumqttc::tokio_rustls::rustls::ClientConfig;
use rumqttc::{Client, Event, MqttOptions, Packet, QoS, Transport};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::{sync::mpsc, thread, time::Duration};
use uuid::Uuid;

use crate::definitions::receiver_builder::ChannelReceiverBuilder;
use crate::definitions::sender_builder::ChannelSenderBuilder;
use crate::definitions::ChannelBuilder;
use crate::definitions::{ChannelDefinition, ChannelReceiverDefinition, ChannelSenderDefinition};
use crate::receiver::ChannelReceiver;
use crate::sender::ChannelSender;
use crate::tether_compliant_topic::{TetherCompliantTopic, TetherOrCustomTopic};

const TIMEOUT_SECONDS: u64 = 3;
const DEFAULT_USERNAME: &str = "tether";
const DEFAULT_PASSWORD: &str = "sp_ceB0ss!";

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

/**
Typically, you will use this to construct a well-configured TetherAgent with a combination
of sensible defaults and custom overrides.

Make a new instance of TetherAgentBuilder with `TetherAgentBuilder::new()`, chain whatever
overrides you might need, and finally call `build()` to get the actual TetherAgent instance.
*/
#[derive(Clone)]
pub struct TetherAgentBuilder {
    role: String,
    id: Option<String>,
    protocol: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    base_path: Option<String>,
    auto_connect: bool,
    mqtt_client_id: Option<String>,
}

impl TetherAgentBuilder {
    /// Initialise Tether Options struct with default options; call other methods to customise.
    /// Call `build()` to get the actual TetherAgent instance (and connect automatically, by default)
    pub fn new(role: &str) -> Self {
        TetherAgentBuilder {
            role: String::from(role),
            id: None,
            protocol: None,
            host: None,
            port: None,
            username: None,
            password: None,
            base_path: None,
            auto_connect: true,
            mqtt_client_id: None,
        }
    }

    /// Optionally sets the **Tether ID**, as used in auto-generating topics such as `myRole/myID/myChannel` _not_ the MQTT Client ID.
    /// Provide Some(value) to override or None to use the default `any` (when publishing) or `+` when subscribing.
    pub fn id(mut self, id: Option<&str>) -> Self {
        self.id = id.map(|x| x.into());
        self
    }

    /// Provide Some(value) to override or None to use default
    pub fn protocol(mut self, protocol: Option<&str>) -> Self {
        self.protocol = protocol.map(|x| x.into());
        self
    }

    /// Optionally set the **MQTT Client ID** used when connecting to the MQTT broker. This is _not_ the same as the **Tether ID**
    /// used for auto-generating topics.
    ///
    /// By default we use a UUID for this value, in order to avoid hard-to-debug issues where Tether Agent instances share
    /// the same Client ID and therefore events/messages are not handled properly by all instances.
    pub fn mqtt_client_id(mut self, client_id: Option<&str>) -> Self {
        self.mqtt_client_id = client_id.map(|x| x.into());
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

    /// Provide Some(value) to override or None to use default
    pub fn base_path(mut self, base_path: Option<&str>) -> Self {
        self.base_path = base_path.map(|x| x.into());
        self
    }

    /// Specify explicitly whether to attempt auto-connection on build;
    /// if set to `false` you will need to connect the TetherAgent (and therefore
    /// its underlying MQTT client) yourself after creating the instance.
    pub fn auto_connect(mut self, should_auto_connect: bool) -> Self {
        self.auto_connect = should_auto_connect;
        self
    }

    /// Using a combination of sensible defaults and any overrides you might
    /// have provided using other functions called on TetherAgentOptions, this
    /// function returns a well-configured TetherAgent instance.
    ///
    /// Unless you set `.auto_connect(false)`, the TetherAgent will attempt to
    /// connect to the MQTT broker automatically upon creation.
    pub fn build(self) -> anyhow::Result<TetherAgent> {
        let protocol = self.protocol.clone().unwrap_or("mqtt".into());
        let host = self.host.clone().unwrap_or("localhost".into());
        let port = self.port.unwrap_or(1883);
        let username = self.username.unwrap_or(DEFAULT_USERNAME.into());
        let password = self.password.unwrap_or(DEFAULT_PASSWORD.into());
        let base_path = self.base_path.unwrap_or("/".into());

        debug!(
            "final build uses options protocol = {}, host = {}, port = {}",
            protocol, host, port
        );

        let (message_sender, message_receiver) = mpsc::channel::<(TetherOrCustomTopic, Vec<u8>)>();

        let mut agent = TetherAgent {
            role: self.role.clone(),
            id: self.id,
            host,
            port,
            username,
            password,
            protocol,
            base_path,
            client: None,
            message_sender,
            message_receiver,
            mqtt_client_id: self.mqtt_client_id,
            is_connected: Arc::new(Mutex::new(false)),
            auto_connect_enabled: self.auto_connect,
        };

        if self.auto_connect {
            match agent.connect() {
                Ok(()) => Ok(agent),
                Err(e) => Err(e),
            }
        } else {
            warn!("Auto-connect disabled; you must call .connect explicitly");
            Ok(agent)
        }
    }
}

impl<'a> TetherAgent {
    /// The simplest way to create a ChannelSender.
    ///
    /// You provide only a Channel Name;
    /// configuration derived from your Tether Agent instance is used to construct
    /// the appropriate publishing topics.
    pub fn create_sender<T: Serialize>(&self, name: &str) -> ChannelSender<T> {
        ChannelSender::new(self, ChannelSenderBuilder::new(name).build(self))
    }

    /// Create a ChannelSender instance using a ChannelSenderDefinition already constructed
    /// elsewhere.
    pub fn create_sender_with_definition<T: Serialize>(
        &self,
        definition: ChannelSenderDefinition,
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
        ChannelReceiver::new(self, ChannelReceiverBuilder::new(name).build())
    }

    /// Create a ChannelReceiver instance using a ChannelReceiverDefinition already constructed
    /// elsewhere.
    pub fn create_receiver_with_definition<T: Deserialize<'a>>(
        &'a self,
        definition: ChannelReceiverDefinition,
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
        channel_definition: &ChannelSenderDefinition,
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

    pub fn send_empty(&self, channel_definition: &ChannelSenderDefinition) -> anyhow::Result<()> {
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
