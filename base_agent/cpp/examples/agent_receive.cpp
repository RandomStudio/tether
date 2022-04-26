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

  Output* outputPlug = agent.createOutput("testOut");
  Input* inputPlug = agent.createInput("testInput", [&](std::string payload, std::string topic) -> void{
    std::cout << "----------> onMessage got: " << payload << " from " << topic << std::endl;

    msgpack::object_handle oh = msgpack::unpack(payload.data(), payload.size());
    msgpack::object const& obj = oh.get();
    std::cout << "Unpacked msgpack object." << std::endl;
    std::cout << obj << std::endl;


  });
  // inputPlug->onMessage();
  

  //Create a dummy struct instance to send...
  dummyData d {
    "comet", 101, 98.0
  };

  // // Make a buffer, pack data using messagepack...
  std::stringstream buffer;
  msgpack::pack(buffer, d);

  outputPlug->publish(buffer.str());

  std::cout << "OK" << std::endl;
  
  while (std::tolower(std::cin.get()) != 'q')
  ;


  agent.disconnect();

  return 0;
}