#include <Arduino.h>
#include <WiFi.h>
#include <ArduinoJson.h>

// Globals
JsonDocument doc;
String outputTopic = "tetherduino/output/mcuData";

WiFiClient wifiClient;

#define WLAN_SSID "Random Guest"
#define WLAN_PASS "beourguest"

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

void setup() {
  Serial.begin(9600);
  while (!Serial) continue;

  sleep(3);
  Serial.println("start");

  connectWiFi();

   const char* json = "{\"sensor\":\"gps\",\"time\":1351824120,\"data\":[48.756080,2.302038]}";


  DeserializationError error = deserializeJson(doc, json);
  if (error) {
    Serial.print(F("deserializeJson() failed: "));
    Serial.println(error.c_str());
    return;
  }

  // Extract the values
  const char* sensor = doc["sensor"];
  long time = doc["time"];
  double latitude = doc["data"][0];
  double longitude = doc["data"][1];

  // Print the values
  Serial.println(sensor);
  Serial.println(time);
  Serial.println(latitude, 6);
  Serial.println(longitude, 6);}

void loop() {
  // put your main code here, to run repeatedly:
}

// put function definitions here:
int myFunction(int x, int y) {
  return x + y;
}