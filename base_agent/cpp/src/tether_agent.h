#include <string>
#include <sstream>

#include "mqtt/async_client.h"

enum FlowDirection {
  IN, OUT
};
struct PlugDefinition {
  std::string name;
  std::string topic;
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
      std::cout << "Output created: " << definition.name << std::endl;  
    };

    void publish(mqtt::binary_ref payload) {
      mToken = mTopic->publish(payload);
    }
};

class TetherAgent {

  private:
    std::string mAgentType;
    std::string mAgentID;

    mqtt::async_client*  mClient;
    std::vector<Output*> mOutputs;

  public:
    TetherAgent(std::string agentType, std::string agentID);

    int connect (std::string protocol, std::string host, int port);
    void disconnect();

    Output* createOutput(std::string name);

    void publish();
  
};