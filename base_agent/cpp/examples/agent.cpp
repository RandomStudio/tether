#include "tether_agent.h"
#include <iostream>
#include <msgpack.hpp>

// Define your own data payload type as a struct
// Don't forget the MSGPACK_DEFINE_MAP macro, so that MessagePack
// knows how to pack the data.
struct dummyData {
	std::string name;
	int distance;
  double probabilityOfImpact;
	MSGPACK_DEFINE_MAP(name, distance, probabilityOfImpact);
};

int main() {

  std::cout << "Starting Tether Agent example..." << std::endl;

  TetherAgent agent ("dummy", "dummy01");

  agent.connect("tcp", "tether-io.dev", 1883);

  Output* outputPlug = agent.createOutput("testout");

  // Create a dummy struct instance to send...
  dummyData d {
    "comet", 101, 98.785
  };

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