use log::{debug, error, info};
use mqtt::{Client, Message, MessageBuilder, Receiver};
pub use paho_mqtt as mqtt;
pub use rmp_serde;
use rmp_serde::to_vec_named;
pub use serde;
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

const TIMEOUT_SECONDS: u64 = 10;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlugDefinition {
    pub name: String,
    pub topic: String,
    pub qos: i32,
}

pub struct TetherAgent {
    role: String,
    id: String,
    client: Client,
    broker_uri: String,
    receiver: Receiver<Option<Message>>,
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

    pub fn new(role: &str, id: Option<&str>, broker_host: Option<IpAddr>) -> Self {
        let tether_host = broker_host.unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

        let broker_uri = format!("tcp://{tether_host}:1883");

        info!("Attempt connection broker at {}", &broker_uri);

        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(broker_uri.clone())
            .client_id("")
            .finalize();

        // Create the client connection
        let client = mqtt::Client::new(create_opts).unwrap();

        // Initialize the consumer before connecting
        let receiver = client.start_consuming();

        TetherAgent {
            role: String::from(role),
            id: String::from(id.unwrap_or("any")),
            client,
            receiver,
            broker_uri: broker_uri.clone(),
        }
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

    pub fn create_input_plug(
        &self,
        name: &str,
        qos: Option<i32>,
        override_topic: Option<&str>,
    ) -> Result<PlugDefinition, ()> {
        let name = String::from(name);
        let topic = String::from(override_topic.unwrap_or(&default_subscribe_topic(&name)));
        let qos = qos.unwrap_or(1);

        match self.client.subscribe(&topic, qos) {
            Ok(_res) => {
                info!("Subscribed to topic {} OK", &topic);
                let plug = PlugDefinition { name, topic, qos };
                debug!("Creating plug: {:?}", &plug);
                // self.input_plugs.push(plug);
                Ok(plug)
            }
            Err(e) => {
                error!("Error subscribing to topic {}: {:?}", &topic, e);
                Err(())
            }
        }
    }

    pub fn create_output_plug(
        &self,
        name: &str,
        qos: Option<i32>,
        override_topic: Option<&str>,
    ) -> Result<PlugDefinition, ()> {
        let name = String::from(name);
        let topic =
            String::from(override_topic.unwrap_or(&build_topic(&self.role, &self.id, &name)));
        let qos = qos.unwrap_or(1);

        let plug = PlugDefinition { name, topic, qos };
        debug!("Adding output plug: {:?}", &plug);
        Ok(plug)
    }

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
    pub fn publish(&self, plug: &PlugDefinition, payload: Option<&[u8]>) -> Result<(), ()> {
        let message = MessageBuilder::new()
            .topic(&plug.topic)
            .payload(payload.unwrap_or(&[]))
            .qos(plug.qos)
            .finalize();
        if let Err(e) = self.client.publish(message) {
            error!("Error publishing: {:?}", e);
            Err(())
        } else {
            Ok(())
        }
    }

    /// Similar to `publish` but serializes the data automatically before sending
    pub fn encode_and_publish<T: Serialize>(
        &self,
        plug: &PlugDefinition,
        data: T,
    ) -> Result<(), ()> {
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
    match parts.get(0) {
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
