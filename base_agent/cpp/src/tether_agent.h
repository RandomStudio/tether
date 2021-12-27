#include <string>
#include <sstream>

#include "mqtt/async_client.h"

class TetherAgent {

  private:
    std::string mAgentType;
    std::string mAgentID;

  public:
    TetherAgent(std::string agentType, std::string agentID);
    void connect (std::string protocol, std::string host, int port);
  
};