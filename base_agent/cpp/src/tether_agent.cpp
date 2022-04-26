#include "tether_agent.h"
#include <iostream>

TetherAgent::TetherAgent (std::string agentType, std::string agentID) {
  mAgentType = agentType;
  mAgentID = agentID;
  mClient = NULL;
  // mIsConnected = false;
}

int TetherAgent::connect (std::string protocol, std::string host, int port)  {
  std::string address = protocol + "://" + host + ":" + std::to_string(port);

  mosquitto_lib_init();
  mClient = mosquitto_new(NULL, true, this);

  mosquitto_connect_callback_set(mClient, on_connect_wrapper);
  mosquitto_subscribe_callback_set(mClient, on_subscribe_wrapper);
  mosquitto_message_callback_set(mClient, on_message_wrapper);

  mosquitto_username_pw_set(mClient, "tether", "sp_ceB0ss!");

  std::cout << "Connecting to broker at " << address << " ..." << std::endl;

  int rc = mosquitto_connect(mClient, host.c_str(), port, 60);

  if (rc != MOSQ_ERR_SUCCESS) {
    std::cout << "Connect error: " << mosquitto_strerror(rc);
    return 0;
  }

  /* Run the network loop in a background thread, this call returns quickly. */
	rc = mosquitto_loop_start(mClient);
	if(rc != MOSQ_ERR_SUCCESS){
		mosquitto_destroy(mClient);
		fprintf(stderr, "Error: %s\n", mosquitto_strerror(rc));
		return 1;
	}

  return 1;

}

Output* TetherAgent::createOutput(std::string name) {
  // TODO: check for mClient == NULL ?

  PlugDefinition def {
    name, 
    mAgentType + "/" + mAgentID + "/" + name
  };

  Output* p = new Output(def, mClient);
  mOutputs.push_back(p);

  std::cout << "Tether Agent now has " << mOutputs.size() << " output plug(s)" << std::endl;

  return p;
}

Input* TetherAgent::createInput(std::string name, std::function<void(std::string, std::string)> callback) {
  std::string topic = "+/+/" + name;
  std::cout << "Creating input for topic " + topic << std::endl;
  PlugDefinition def {
    name,
    topic
  };

  Input* p = new Input(def, mClient, callback);

  return p;
} 

void TetherAgent::disconnect() {
  std::cout << "\nDisconnecting..." << std::endl;
  // mClient->disconnect()->wait();
  std::cout << "  ...OK" << std::endl;
}