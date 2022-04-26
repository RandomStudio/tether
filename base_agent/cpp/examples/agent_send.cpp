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

unsigned long someBigNumber;

int main() {

  someBigNumber = 0;

  std::cout << "Starting Tether Agent example..." << std::endl;

  TetherAgent agent ("dummy", "dummy01");

  int result = agent.connect("tcp", "tether-io.dev", 1883);

  if (result) {
    std::cout << "Connected OK" << std::endl;
  } else {
    std::cout << "Connection error!" << std::endl;
  }

  Output* outputPlug = agent.createOutput("testout");

  //Create a dummy struct instance to send...
  dummyData d {
    "comet", 101, 98.0
  };

  while (someBigNumber < 10) {
    someBigNumber++;
    sleep(1);

    // // Make a buffer, pack data using messagepack...
    std::stringstream buffer;
    msgpack::pack(buffer, d);

    outputPlug->publish(buffer.str());

    std::cout << "OK" << std::endl;
  }


  agent.disconnect();

  return 0;
}