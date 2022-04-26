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

  mosquitto_lib_init();
  mClient = mosquitto_new("Tether", true, this);

  

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

// Input* TetherAgent::createInput(std::string name, std::function<void(std::string, std::string)> callback) {
//   std::string topic = "+/+/" + name;
//   std::cout << "Creating input for topic " + topic << std::endl;
//   PlugDefinition def {
//     name,
//     // mAgentType + "/" + mAgentID + "/" + name
//     topic
//   };

//   Input* p = new Input(def, mClient, callback);
//   mClient->set_callback(*p);

//   return p;
// } 

void TetherAgent::disconnect() {
  std::cout << "\nDisconnecting..." << std::endl;
  // mClient->disconnect()->wait();
  std::cout << "  ...OK" << std::endl;
}