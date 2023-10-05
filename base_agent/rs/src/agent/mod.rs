use log::{error, info, warn};
use mqtt::{Client, Message, MessageBuilder, Receiver};
use paho_mqtt as mqtt;
use rmp_serde::to_vec_named;
use serde::Serialize;
use std::time::Duration;

use crate::{PlugDefinition, PlugDefinitionCommon, ThreePartTopic};

const TIMEOUT_SECONDS: u64 = 10;
pub struct TetherAgent {
    role: String,
    id: String,
    client: Client,
    broker_uri: String,
    receiver: Receiver<Option<Message>>,
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

fn convert_optional<T, U: std::convert::From<T>>(optional_value: Option<T>) -> Option<U> {
    match optional_value {
        Some(v) => Some(v.into()),
        None => None,
    }
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

    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Provide Some(value) to override or None to use default
    pub fn id_optional(mut self, id: Option<String>) -> Self {
        self.id = convert_optional(id);
        self
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Provide Some(value) to override or None to use default
    pub fn host_optional(mut self, host: Option<String>) -> Self {
        self.host = convert_optional(host);
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn username(mut self, username: &str) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Provide Some(value) to override or None to use default
    pub fn username_optional(mut self, username: Option<String>) -> Self {
        self.username = convert_optional(username);
        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Provide Some(value) to override or None to use default
    pub fn password_optional(mut self, password: Option<String>) -> Self {
        self.password = convert_optional(password);
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

        info!("Create connection for broker at {}", &broker_uri);

        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(broker_uri.clone())
            .client_id("")
            .finalize();

        // Create the client connection
        let client = mqtt::Client::new(create_opts).unwrap();

        // Initialize the consumer before connecting
        let receiver = client.start_consuming();

        let agent = TetherAgent {
            role: self.role.clone(),
            id: self.id.clone().unwrap_or("any".into()),
            client,
            broker_uri,
            receiver,
        };

        if self.auto_connect {
            match agent.connect(&self) {
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

    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
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

    pub fn connect(&self, options: &TetherAgentOptionsBuilder) -> Result<(), mqtt::Error> {
        let username = options.clone().username.unwrap_or("tether".into());
        let password = options.clone().password.unwrap_or("sp_ceB0ss!".into());
        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .user_name(username)
            .password(password)
            .connect_timeout(Duration::from_secs(TIMEOUT_SECONDS))
            .keep_alive_interval(Duration::from_secs(TIMEOUT_SECONDS))
            // .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
            .clean_session(true)
            .finalize();

        // Make the connection to the broker
        info!("Connecting to the MQTT server...");

        match self.client.connect(conn_opts) {
            Ok(res) => {
                info!("Connected OK: {res:?}");
                Ok(())
            }
            Err(e) => {
                error!("Error connecting to the broker: {e:?}");
                // self.client.stop_consuming();
                // self.client.disconnect(None).expect("failed to disconnect");
                Err(e)
            }
        }
    }

    /// If a message is waiting return ThreePartTopic, Message (String, Message)
    pub fn check_messages(&self) -> Option<(ThreePartTopic, Message)> {
        if let Some(message) = self.receiver.try_iter().find_map(|m| m) {
            if let Ok(t) = ThreePartTopic::try_from(message.topic()) {
                Some((t, message))
            } else {
                error!("Could not pass Three Part Topic from \"{}\"", message.topic());
                warn!("Message was ignored");
                None                
            }
        } else {
            None
        }
    }

    /// Given a plug definition and a raw (u8 buffer) payload, generate a message
    /// on an appropriate topic and with the QOS specified in the Plug Definition
    pub fn publish(
        &self,
        plug_definition: &PlugDefinition,
        payload: Option<&[u8]>,
    ) -> anyhow::Result<()> {
        match plug_definition {
            PlugDefinition::InputPlugDefinition(_) => {
                panic!("You cannot publish using an Input Plug")
            }
            PlugDefinition::OutputPlugDefinition(output_plug_definition) => {
                let PlugDefinitionCommon { three_part_topic, qos, .. } = &plug_definition.common();
                let message = MessageBuilder::new()
                    .topic(three_part_topic.topic())
                    .payload(payload.unwrap_or(&[]))
                    .retained(output_plug_definition.retain())
                    .qos(*qos)
                    .finalize();
                if let Err(e) = self.client.publish(message) {
                    error!("Error publishing: {:?}", e);
                    Err(e.into())
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Similar to `publish` but serializes the data automatically before sending
    pub fn encode_and_publish<T: Serialize>(
        &self,
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
        &self,
        topic: &str,
        payload: &[u8],
        qos: Option<i32>,
        retained: Option<bool>,
    ) -> anyhow::Result<()> {
        let message = MessageBuilder::new()
            .topic(topic)
            .payload(payload)
            .retained(retained.unwrap_or(false))
            .qos(qos.unwrap_or(1))
            .finalize();
        if let Err(e) = self.client.publish(message) {
            error!("Error publishing: {:?}", e);
            Err(e.into())
        } else {
            Ok(())
        }
    }
}
