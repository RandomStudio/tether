#include <WiFi.h>
#include <Adafruit_MQTT_FONA.h>
#include <Adafruit_MQTT.h>
#include <Adafruit_MQTT_Client.h>
#include <ArduinoJson.h> // ArduinoJson v6 has built-in MessagePack (de)serialization capabilities

#define WLAN_SSID "MySSID"
#define WLAN_PASS "MyPass"
#define MQTT_IP "192.168.0.2"
#define MQTT_PORT 1883
#define MQTT_USER "guest"
#define MQTT_PASS "guest"

WiFiClient client;
Adafruit_MQTT_Client mqtt(&client, MQTT_IP, MQTT_PORT, MQTT_USER, MQTT_PASS);

Adafruit_MQTT_Publish output = Adafruit_MQTT_Publish(&mqtt, "tetherduino/output/mcuData");
Adafruit_MQTT_Subscribe input = Adafruit_MQTT_Subscribe(&mqtt, "dummy/nodejs_dummy/dummyData");

StaticJsonDocument<512> inputDoc;
StaticJsonDocument<16> outputDoc;
std::string outputMessage;

void setup() {
  Serial.begin(115200);
  delay(10);
  connectWiFi();
  mqtt.subscribe(&input);
}

void loop() {
  // check if we're connected, and reconnect if not
  MQTT_connect();

  // listen for incoming subscription packets
  receive();

  if (!mqtt.ping()) {
    mqtt.disconnect();
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

  while ((ret = mqtt.connect()) != 0) { // connect will return 0 for connected
     Serial.println(mqtt.connectErrorString(ret));
     Serial.println("Retrying MQTT connection in 5 seconds...");
     mqtt.disconnect();
     delay(5000);  // wait 5 seconds
  }

  Serial.println("MQTT Connected!");
}

void send(int number) {
  outputDoc["number"] = number; // set the data
  outputMessage = ""; // clear the output string
  serializeMsgPack(outputDoc, outputMessage); // serialize the data
  // send
  if (output.publish(outputMessage.c_str())) {
    Serial.println("Published number: " + String(number));
  } else {
    Serial.println("Could not publish number: " + String(number));
  }
}

void receive() {
  Adafruit_MQTT_Subscribe *subscription;
  while ((subscription = mqtt.readSubscription(5000))) {
    if (subscription == &input) {
      decodeMessage(&(input.lastread[0]), input.datalen);
      send(inputDoc["someNumber"].as<int>());
    }
  }
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
