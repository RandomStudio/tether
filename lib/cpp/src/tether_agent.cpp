#include "tether_agent.h"
#include <iostream>

TetherAgent::TetherAgent (std::string agentType, std::string agentID) {
  mAgentType = agentType;
  mAgentID = agentID;
  mClient = NULL;
}

int TetherAgent::connect (std::string protocol, std::string host, int port)  {
  std::string address = protocol + "://" + host + ":" + std::to_string(port);
  std::cout << "Connecting to broker at " << address << " ..." << std::endl;

  mqtt::connect_options options("tether", "sp_ceB0ss!");
  mClient = new mqtt::async_client(address, "");

  try {
    mClient->connect(options)->wait();
    std::cout << "Connected OK!" << std::endl;
  } catch (const mqtt::exception& exc) {
		std::cerr << exc << std::endl;
		return 1;
	}

  return 0;

}

Output* TetherAgent::createOutput(std::string name) {
  // TODO: check for mClient == NULL ?

  PlugDefinition def {
    name, 
    // mAgentType + "/" + mAgentID + "/" + name
  };

  Output* p = new Output(def, mClient);
  mOutputs.push_back(p);

  std::cout << "Tether Agent now has " << mOutputs.size() << " output plug(s)" << std::endl;

  return p;
}

Input* TetherAgent::createInput(std::string name, std::function<void(std::string, std::string)> callback) {
  PlugDefinition def {
    name,
    // mAgentType + "/" + mAgentID + "/" + name
    "+/+/" + name
  };

  Input* p = new Input(def, mClient, callback);
  mClient->set_callback(*p);

  return p;
} 

void TetherAgent::disconnect() {
  std::cout << "\nDisconnecting..." << std::endl;
  mClient->disconnect()->wait();
  std::cout << "  ...OK" << std::endl;
}