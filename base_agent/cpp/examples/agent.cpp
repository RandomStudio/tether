#include "tether_agent.h"
#include <iostream>
#include <msgpack.hpp>

struct dummyData {
	std::string name;
	int distance;
	MSGPACK_DEFINE_MAP(name, distance);
};

int main() {

  std::cout << "Starting Tether Agent example..." << std::endl;

  TetherAgent agent ("dummy", "dummy01");

  agent.connect("tcp", "tether-io.dev", 1883);

  Output* outputPlug = agent.createOutput("testout");


  // Create a dummy struct instance to send...
  dummyData d;
  d.name = "sensor";
  d.distance = 101;

  // Make a buffer, pack data using messagepack...
  std::stringstream buffer;
  msgpack::pack(buffer, d);
  const std::string& tmp = buffer.str();   
  const char* payload = tmp.c_str();

  outputPlug->publish(payload);

  std::cout << "OK" << std::endl;

  agent.disconnect();

  return 0;
}