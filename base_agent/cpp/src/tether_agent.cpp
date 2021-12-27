#include "tether_agent.h"
#include <iostream>

TetherAgent::TetherAgent (std::string agentType, std::string agentID) {
  mAgentType = agentType;
  mAgentID = agentID;
}

void TetherAgent::connect (std::string protocol, std::string host, int port)  {
  std::string address = protocol + "://" + host + ":" + std::to_string(port);
  std::cout << "Connecting to broker at " << address << " ..." << std::endl;
}