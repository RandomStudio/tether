use log::{debug, error, info, warn};
use mqtt::{Client, Message, MessageBuilder, Receiver};
pub use paho_mqtt as mqtt;
pub use rmp_serde;
use rmp_serde::to_vec_named;
pub use serde;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const TIMEOUT_SECONDS: u64 = 10;

#[derive(Debug, Clone)]
pub struct PlugDefinitionCommon {
    pub name: String,
    pub topic: String,
    pub qos: i32,
}

/// This is the definition of an Input or Output Plug
/// You should never use this directly; call finalize()
/// to get a usable Plug
enum PlugDefinition<'a> {
    InputPlug {
        common: PlugDefinitionCommon,
        tether_agent: &'a TetherAgent,
    },
    OutputPlug {
        common: PlugDefinitionCommon,
        retain: bool,
    },
}

pub struct Plug<'a> {
    pub definition: PlugDefinition<'a>,
}

impl PlugDefinition<'_> {
    fn common(&mut self) -> &mut PlugDefinitionCommon {
        match self {
            PlugDefinition::InputPlug { common, .. } => common,
            PlugDefinition::OutputPlug { common, retain: _ } => common,
        }
    }

    pub fn new_input<'a>(tether_agent: &'a TetherAgent, name: &str) -> PlugDefinition<'a> {
        PlugDefinition::InputPlug {
            common: PlugDefinitionCommon {
                name: name.into(),
                topic: default_subscribe_topic(&name),
                qos: 1,
            },
            tether_agent,
        }
    }

    pub fn new_output<'a>(tether_agent: &TetherAgent, name: &str) -> PlugDefinition<'a> {
        PlugDefinition::OutputPlug {
            common: PlugDefinitionCommon {
                name: name.into(),
                topic: build_topic(&tether_agent.role, &tether_agent.id, &name),
                qos: 1,
            },
            retain: false,
        }
    }

    pub fn qos(mut self, qos: i32) -> Self {
        self.common().qos = qos;
        self
    }

    pub fn topic(mut self, override_topic: &str) -> Self {
        self.common().topic = override_topic.into();
        self
    }

    pub fn retain(mut self, should_retain: bool) -> Self {
        match self {
            Self::InputPlug {
                common,
                tether_agent,
            } => {
                panic!("Cannot set retain flag on Input Plug / subscription")
            }
            Self::OutputPlug { common, mut retain } => {
                retain = should_retain;
                self
            }
        }
    }

    pub fn finalize<'a>(self) -> Plug<'a> {
        match self {
            PlugDefinition::InputPlug {
                common,
                tether_agent,
            } => {
                tether_agent.client.subscribe(&common.topic, common.qos);
                Plug { definition: self }
            }
            PlugDefinition::OutputPlug { common, retain } => Plug { definition: self },
        }
    }
}

pub struct TetherAgent {
    role: String,
    id: String,
    client: Client,
    broker_uri: String,
    receiver: Receiver<Option<Message>>,
}

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
    /// Call `finalize()` to get the actual TetherAgent instance (and connect automatically, by default)
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

    pub fn host(mut self, host: &str) -> Self {
        self.host = Some(host.into());
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

    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn auto_connect(mut self, should_auto_connect: bool) -> Self {
        self.auto_connect = should_auto_connect;
        self
    }

    pub fn finalize(self) -> Result<TetherAgent, ()> {
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
            match agent.connect(self.username.clone(), self.password.clone()) {
                Ok(()) => Ok(agent),
                Err(_) => Err(()),
            }
        } else {
            warn!("Auto-connect disabled; you must call .connect explicitly");
            Ok(agent)
        }
    }
}

impl TetherAgent {
    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    /// Returns the Agent Role and ID (group)
    pub fn description(&self) -> (&str, &str) {
        (&self.role, &self.id)
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

    pub fn connect(
        &self,
        user: Option<String>,
        password: Option<String>,
    ) -> Result<(), mqtt::Error> {
        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .user_name(user.unwrap_or(String::from("tether")))
            .password(password.unwrap_or(String::from("sp_ceB0ss!")))
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

    // pub fn create_input_plug(&self, name: &str) -> PlugDefinition {
    //     let name = String::from(name);
    //     let topic = String::from(override_topic.unwrap_or(&default_subscribe_topic(&name)));
    //     let qos = qos.unwrap_or(1);

    //     match self.client.subscribe(&topic, qos) {
    //         Ok(_res) => {
    //             info!("Subscribed to topic {} OK", &topic);
    //             let plug = PlugDefinition::InputPlug {
    //                 common: PlugDefinitionCommon { name, topic, qos },
    //             };
    //             debug!("Creating plug: {:?}", &plug);
    //             // self.input_plugs.push(plug);
    //             Ok(plug)
    //         }
    //         Err(e) => {
    //             error!("Error subscribing to topic {}: {:?}", &topic, e);
    //             Err(())
    //         }
    //     }
    // }

    // pub fn create_output_plug(
    //     &self,
    //     name: &str,
    //     qos: Option<i32>,
    //     retain: Option<bool>,
    //     override_topic: Option<&str>,
    // ) -> Result<PlugDefinition, ()> {
    //     let name = String::from(name);
    //     let topic =
    //         String::from(override_topic.unwrap_or(&build_topic(&self.role, &self.id, &name)));
    //     let qos = qos.unwrap_or(1);
    //     let retain = retain.unwrap_or(false);

    //     let plug = PlugDefinition {
    //         name,
    //         topic,
    //         qos,
    //         retain,
    //     };
    //     debug!("Adding output plug: {:?}", &plug);
    //     Ok(plug)
    // }

    /// If a message is waiting return Plug Name, Message (String, Message)
    pub fn check_messages(&self) -> Option<(String, Message)> {
        if let Some(message) = self.receiver.try_iter().find_map(|m| m) {
            let topic = message.topic();
            if let Some(plug_name) = parse_plug_name(topic) {
                Some((String::from(plug_name), message))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Given a plug definition and a raw (u8 buffer) payload, generate a message
    /// on an appropriate topic and with the QOS specified in the Plug Definition
    pub fn publish<'a>(&self, plug: &Plug, payload: Option<&[u8]>) -> Result<(), ()> {
        match &plug.definition {
            PlugDefinition::InputPlug {
                common: _,
                tether_agent: _,
            } => panic!("You cannot publish using an Input Plug"),
            PlugDefinition::OutputPlug { common, retain } => {
                let message = MessageBuilder::new()
                    .topic(&common.topic)
                    .payload(payload.unwrap_or(&[]))
                    .retained(*retain)
                    .qos(common.qos)
                    .finalize();
                if let Err(e) = self.client.publish(message) {
                    error!("Error publishing: {:?}", e);
                    Err(())
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Similar to `publish` but serializes the data automatically before sending
    pub fn encode_and_publish<T: Serialize>(&self, plug: &Plug, data: T) -> Result<(), ()> {
        let payload = to_vec_named(&data).unwrap();
        self.publish(plug, Some(&payload))
    }
}

pub fn parse_plug_name(topic: &str) -> Option<&str> {
    let parts: Vec<&str> = topic.split('/').collect();
    match parts.get(2) {
        Some(s) => Some(*s),
        None => None,
    }
}

pub fn parse_agent_id(topic: &str) -> Option<&str> {
    let parts: Vec<&str> = topic.split('/').collect();
    match parts.get(1) {
        Some(s) => Some(*s),
        None => None,
    }
}

pub fn parse_agent_role(topic: &str) -> Option<&str> {
    let parts: Vec<&str> = topic.split('/').collect();
    match parts.first() {
        Some(s) => Some(*s),
        None => None,
    }
}

pub fn build_topic(role: &str, id: &str, plug_name: &str) -> String {
    format!("{role}/{id}/{plug_name}")
}

pub fn default_subscribe_topic(plug_name: &str) -> String {
    format!("+/+/{plug_name}")
}
