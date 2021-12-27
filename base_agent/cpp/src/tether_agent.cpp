#include "tether_agent.h"
#include <iostream>

using namespace std;

TetherAgent::TetherAgent (string agentType, string agentID) {
  mAgentType = agentType;
  mAgentID = agentID;
  mClient = NULL;
}

int TetherAgent::connect (string protocol, string host, int port)  {
  string address = protocol + "://" + host + ":" + to_string(port);
  cout << "Connecting to broker at " << address << " ..." << endl;

  mqtt::connect_options options("tether", "sp_ceB0ss!");
  mClient = new mqtt::async_client(address, "");

  try {
    mClient->connect(options)->wait();
    cout << "Connected OK!" << endl;
  } catch (const mqtt::exception& exc) {
		cerr << exc << endl;
		return 1;
	}

  return 0;

}

Output* TetherAgent::createOutput(string name) {
  // TODO: check for mClient == NULL ?

  PlugDefinition def {
    name, 
    mAgentType + "/" + mAgentID + "/" + name,
    OUT
  };

  Output* p = new Output(def, mClient);
  mOutputs.push_back(p);

  cout << "Tether Agent now has " << mOutputs.size() << " output plug(s)" << endl;

  return p;
}

void TetherAgent::disconnect() {
  cout << "\nDisconnecting..." << endl;
  mClient->disconnect()->wait();
  cout << "  ...OK" << endl;
}