#include "tether_agent.h"
#include <iostream>

int main() {

  std::cout << "Starting Tether Agent example..." << std::endl;

  TetherAgent agent ("dummy", "dummy01");

  agent.connect("tcp", "tether-io.dev", 1883);

  std::cout << "OK" << std::endl;

  return 0;
}