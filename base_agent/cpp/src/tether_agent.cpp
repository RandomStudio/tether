#include "tether_agent.h"
#include <iostream>

TetherAgent::TetherAgent (std::string agentType, std::string agentID) {
  mAgentType = agentType;
  mAgentID = agentID;
}

void TetherAgent::connect (std::string protocol, std::string address, int port)  {
  std::cout << "Connecting to broker at " << protocol << address << port << std::endl;
}