#include <WiFi.h>
#include <ArduinoMqttClient.h>
#include <ArduinoJson.h> // ArduinoJson v6 has built-in MessagePack (de)serialization capabilities

#define WLAN_SSID "Random Wireless"
#define WLAN_PASS "Fragile-Freedom"

#define MQTT_IP "192.168.1.130"
#define MQTT_PORT 1883
#define MQTT_USER "guest"
#define MQTT_PASS "guest"

String inputTopic = "+/+/dummyData";
String outputTopic = "tetherduino/output/mcuData";

WiFiClient wifiClient;
MqttClient mqtt(wifiClient);

StaticJsonDocument<512> inputDoc;
StaticJsonDocument<16> outputDoc;
std::string outputMessage;

void setup() {
  Serial.begin(115200);
  delay(10);
  connectWiFi();
  mqtt.onMessage(onMqttMessage);
}

void loop() {
  // check if we're connected, and reconnect if not
  MQTT_connect();

  // listen for incoming subscription packets
  mqtt.poll();

  if (!mqtt.connected()) {
    mqtt.stop();
  }
}

void connectWiFi() {
  Serial.println("Connecting to WiFi");
  WiFi.begin(WLAN_SSID, WLAN_PASS);
  delay(2000);
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }
  Serial.println("WiFi connected");
}

// Function to connect and reconnect as necessary to the MQTT server.
// Should be called in the loop function and it will take care of connecting.
void MQTT_connect() {
  int8_t ret;

  // Stop if already connected.
  if (mqtt.connected()) {
    return;
  }

  Serial.println("Connecting to MQTT... ");

  mqtt.unsubscribe(inputTopic);
  mqtt.setUsernamePassword(MQTT_USER, MQTT_PASS);
  while (!mqtt.connect(MQTT_IP, MQTT_PORT)) {
     Serial.println(mqtt.connectError());
     Serial.println("Retrying MQTT connection in 5 seconds...");
     delay(5000);  // wait 5 seconds
  }

  Serial.println("MQTT Connected!");

  mqtt.subscribe(inputTopic);
}

void send(int number) {
  outputDoc["number"] = number; // set the data
  outputMessage = ""; // clear the output string
  serializeMsgPack(outputDoc, outputMessage); // serialize the data
  // send
  mqtt.beginMessage(outputTopic);
  // mqtt.print(outputMessage.c_str());
  for (int i = 0; i < outputMessage.size(); i++) {
    mqtt.print(outputMessage[i]);
  }
  mqtt.endMessage();
  Serial.println("Published number: " + String(number));
}

void onMqttMessage(int len) {
  String topic = mqtt.messageTopic();
  Serial.println("Received message of length " + String(len) + " on topic " + topic);
//  if (topic == inputTopic) {
    uint8_t buf[len];
    int i = 0;
    while(mqtt.available()) {
      buf[i] = mqtt.read();
      ++i;
    }
    decodeMessage(&(buf[0]), len);
    send(inputDoc["someNumber"].as<int>());
//  }
}

void decodeMessage(unsigned char* message, uint16_t len) {
  deserializeMsgPack(inputDoc, message, len);
  Serial.println("Unpacked message of length " + String(len) + ":");
  Serial.print(" from: "); Serial.println(inputDoc["from"].as<char*>());
  Serial.print(" hello: "); Serial.println(inputDoc["hello"].as<char*>());
  Serial.print(" someNumber: "); Serial.println(inputDoc["someNumber"].as<int>());
  Serial.print(" isEven: "); Serial.println(inputDoc["isEven"].as<bool>());
  Serial.print(" randomArray: [ ");
  for (JsonVariant value : inputDoc["randomArray"].as<JsonArray>()) {
    Serial.print(String(value.as<float>()) + " ");
  }
  Serial.println(" ]");
}
