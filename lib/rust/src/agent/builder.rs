use std::sync::{mpsc, Arc, Mutex};

use log::*;
use uuid::Uuid;

use crate::{tether_compliant_topic::TetherOrCustomTopic, AgentConfig};

use super::TetherAgent;

const DEFAULT_USERNAME: &str = "tether";
const DEFAULT_PASSWORD: &str = "sp_ceB0ss!";

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
        let url_base_path = self.base_path.unwrap_or("/".into());

        debug!(
            "final build uses options protocol = {}, host = {}, port = {}",
            protocol, host, port
        );

        let config = AgentConfig {
            role: self.role.clone(),
            id: self.id,
            host,
            port,
            protocol,
            username,
            password,
            url_base_path,
            mqtt_client_id: self.mqtt_client_id.unwrap_or(Uuid::new_v4().to_string()),
            auto_connect_enabled: self.auto_connect,
        };

        let (message_sender, message_receiver) = mpsc::channel::<(TetherOrCustomTopic, Vec<u8>)>();

        let mut agent = TetherAgent {
            config,
            client: None,
            message_sender,
            message_receiver,
            is_connected: Arc::new(Mutex::new(false)),
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
