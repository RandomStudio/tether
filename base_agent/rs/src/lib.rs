use paho_mqtt as mqtt;

pub struct TetherAgent {
  agent_type: String,
  agent_id: String,
  client: paho_mqtt::AsyncClient
}

pub fn create_agent (agent_type: String, agent_id: String) -> TetherAgent {
  TetherAgent {
    agent_type,
    agent_id,
    client: mqtt::AsyncClient::new("tcp://localhost:1883".to_string()).unwrap()
  }
}

impl TetherAgent {
  pub async fn connect(&self)  {
    println!("Connecting to the MQTT server...");
    self.client.connect(mqtt::ConnectOptionsBuilder::new().user_name("tether").password("sp_ceB0ss!").finalize());
  }

  pub async fn disconnect(&self) {
    println!("Disconnecting MQTT...");
    self.client.disconnect(None);
  }

  pub fn get_id(&self) -> String {
    self.agent_id.to_string()
  }
}

pub fn dummy() {
  println!("Hello world!");
}
