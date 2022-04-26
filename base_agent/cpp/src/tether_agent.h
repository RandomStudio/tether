#include <iostream>
#include <vector>

#include "mosquitto.h"

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
    mosquitto * mClient;

  public:
    Plug (PlugDefinition definition, mosquitto * client){
      mDefinition = definition;
      mClient = client;
    };

}; 

class Output : Plug {
  // private:
    
  public:

    Output(PlugDefinition definition, mosquitto * client): Plug (definition, client) {
      std::cout << "Output plug created: \"" << definition.name << "\" with topic " << mDefinition.topic << std::endl;  
    };

    void publish(std::string payload) {
      int rc = mosquitto_publish(mClient, NULL, mDefinition.topic.c_str(), int(payload.length()), payload.c_str(), 0, false);
      if(rc != MOSQ_ERR_SUCCESS){
        fprintf(stderr, "Error publishing: %s\n", mosquitto_strerror(rc));
      } else {
        // std::cout << "Send " << payload << " on " << mDefinition.topic << " OK" << std::endl;
      }
    }
};



class Input : Plug {
  private:
     std::function<void(std::string, std::string)> mCallback;

  public:

    Input(PlugDefinition definition, mosquitto * client, std::function<void(std::string, std::string)> callback): Plug (definition, client) {
      std::cout << "Input plug created: " << definition.name << std::endl;  

      int rc = mosquitto_subscribe(mClient, NULL, mDefinition.topic.c_str(), 0);
      if(rc != MOSQ_ERR_SUCCESS){
        fprintf(stderr, "Error subscribing: %s\n", mosquitto_strerror(rc));
        // /* We might as well disconnect if we were unable to subscribe */
        // mosquitto_disconnect(mClient);
      }

      // client->subscribe(definition.topic, 1);
      mCallback = callback;
    }

    // void message_arrived(mqtt::const_message_ptr msg) override {
    //   // std::cout << "Message arrived" << std::endl;
    //   // std::cout << "\ttopic: '" << msg->get_topic() << "'" << std::endl;
    //   // std::cout << "\tpayload: '" << msg->get_payload() << "'\n" << std::endl;
    //     // mCallback(msg->get_payload(), msg->get_topic());
	// }

  // // Re-connection failure
	// void on_failure(const mqtt::token& tok) override {
	// 	std::cout << "Connection attempt failed" << std::endl;
	// }

	// void on_success(const mqtt::token& tok) override {}

  // void onMessage(std::function<void(std::string, std::string)> callback) {
  //   mCallback = &callback;
  //   std::cout << "Registered onMessage callback OK" << std::endl;
  //   if (mCallback == nullptr) {
  //   std::cout << "Actually, no" << std::endl;
      
  //   }
  //   // callback("payload", "topic");
  // }

};

class TetherAgent {

  private:
    std::string mAgentType;
    std::string mAgentID;

    struct mosquitto * mClient;
    std::vector<Output*> mOutputs;

    static void on_connect_wrapper(struct mosquitto *, void *userData, int rc) {
      std::cout << "Connected with rc " << rc << std::endl;
    }

    static void on_subscribe_wrapper(struct mosquitto *mosq, void *obj, int mid, int qos_count, const int *granted_qos) {
      int i;
      bool have_subscription = false;

      /* In this example we only subscribe to a single topic at once, but a
      * SUBSCRIBE can contain many topics at once, so this is one way to check
      * them all. */
      for(i=0; i<qos_count; i++){
        printf("on_subscribe: %d:granted qos = %d\n", i, granted_qos[i]);
        if(granted_qos[i] <= 2){
          have_subscription = true;
        }
      }
      if(have_subscription == false){
        fprintf(stderr, "Error: All subscriptions rejected.\n");
        // mosquitto_disconnect(mosq);
      }
    }

    static void on_message_wrapper(struct mosquitto *mosq, void *obj, const struct mosquitto_message *msg) {
      printf("%s %d %s\n", msg->topic, msg->qos, (char *)msg->payload);
    }

  public:
    TetherAgent(std::string agentType, std::string agentID);

    int connect (std::string protocol, std::string host, int port);
    void disconnect();

    Output* createOutput(std::string name);
    Input* createInput(std::string name, std::function<void(std::string, std::string)> callback);

  
};