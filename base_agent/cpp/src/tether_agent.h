#include <string>
#include <sstream>

#include "mqtt/async_client.h"

enum FlowDirection {
  IN, OUT
};
struct PlugDefinition {
  std::string name;
  std::string topic;
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
      std::cout << "Output plug created: " << definition.name << std::endl;  
    };

    void publish(mqtt::binary_ref payload) {
      mToken = mTopic->publish(payload);
    }
};



class Input : Plug, public virtual mqtt::callback,
					public virtual mqtt::iaction_listener {
  private:
  public:

    Input(PlugDefinition definition, mqtt::async_client * client): Plug (definition, client) {
      std::cout << "Input plug created: " << definition.name << std::endl;  
      client->subscribe(definition.topic, 1);
    }

    void message_arrived(mqtt::const_message_ptr msg) override {
      std::cout << "Message arrived" << std::endl;
      std::cout << "\ttopic: '" << msg->get_topic() << "'" << std::endl;
      std::cout << "\tpayload: '" << msg->to_string() << "'\n" << std::endl;
	}

  // Re-connection failure
	void on_failure(const mqtt::token& tok) override {
		std::cout << "Connection attempt failed" << std::endl;
	}

	void on_success(const mqtt::token& tok) override {}

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
    Input* createInput(std::string name);

    void publish();
  
};