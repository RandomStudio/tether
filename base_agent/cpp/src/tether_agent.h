#include <string>

class TetherAgent {

  private:
    std::string mHost;
    int mPort;

  TetherAgent(std::string host = "tether-io.dev", int port = 1883);

};