#include <string>
#include <sstream>

#include "mqtt/async_client.h"

using namespace std;

enum FlowDirection {
  IN, OUT
};
struct PlugDefinition {
  string name;
  string topic;
  FlowDirection flowDirection;
};

class Plug {
  protected:
    PlugDefinition mDefinition;
    mqtt::topic * mTopic;
    mqtt::token_ptr mToken;


  public:
    Plug (PlugDefinition definition, mqtt::async_client * client){
      mDefinition = definition;
      const int QOS = 1;
      mTopic = new mqtt::topic(*client, definition.topic, QOS);
    };

}; 

class Output : Plug {
  private:
  public:

    Output(PlugDefinition definition, mqtt::async_client * client): Plug (definition, client) {
      cout << "Output created: " << definition.name << endl;  
    };

    void publish(mqtt::binary_ref payload) {
      mToken = mTopic->publish(payload);
    }
};

class TetherAgent {

  private:
    string mAgentType;
    string mAgentID;

    mqtt::async_client*  mClient;
    vector<Output*> mOutputs;

  public:
    TetherAgent(string agentType, string agentID);

    int connect (string protocol, string host, int port);
    void disconnect();

    Output* createOutput(string name);

    void publish();
  
};