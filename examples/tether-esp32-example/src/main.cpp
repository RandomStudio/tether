#include <Arduino.h>
#include <WiFi.h>
#include <ArduinoJson.h>
#include <PubSubClient.h>

// Globals

WiFiClient wifiClient;
PubSubClient client(wifiClient);

const char* ssid = "lab_2.4";
const char* password = "sp_ceB0ss!";
const char* mqtt_server = "192.168.27.12";

unsigned long lastMsg = 0;

void connectWiFi() {
  Serial.println("Connecting to WiFi");
  WiFi.begin(ssid, password);
  delay(2000);
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }
  Serial.println("WiFi connected");
}

void connectMQTT() {
  // Loop until we're reconnected
  while (!client.connected()) {
    Serial.print("Attempting MQTT connection...");
    // Create a random client ID
    String clientId = "ESP8266Client-";
    clientId += String(random(0xffff), HEX);
    // Attempt to connect
    if (client.connect(clientId.c_str(), "tether", "sp_ceB0ss!")) {
      Serial.println("connected");
    } else {
      Serial.print("failed, rc=");
      Serial.print(client.state());
      Serial.println(" try again in 5 seconds");
      // Wait 5 seconds before retrying
      delay(5000);
    }
  }
}

void publish() {
  JsonDocument doc;
  doc["hello"] = "world";

  char buffer[256];
  serializeMsgPack(doc, buffer);

  client.publish("dummy/any/testMessage", buffer);
  
}

void setup() {
  Serial.begin(9600);
  while (!Serial) continue;

  sleep(3);
  Serial.println("start");

  connectWiFi();
  client.setServer(mqtt_server, 1883);
}

void loop() {
  if (!client.connected()) {
    connectMQTT();
  }
  client.loop();

  unsigned long now = millis();
  if (now - lastMsg > 4000) {
    lastMsg = now;
    publish();
  }
}
