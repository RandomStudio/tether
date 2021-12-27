#include <iostream>
#include <cstdlib>
#include <thread>	// For sleep
#include <atomic>
#include <chrono>
#include <string>
#include <cstring>
#include "mqtt/async_client.h"

#include <msgpack.hpp>
#include <sstream>

using namespace std;

const string DFLT_SERVER_ADDRESS { "tcp://tether-io.dev:1883" };

const string TOPIC { "test" };
const int QOS = 1;


const auto TIMEOUT = std::chrono::seconds(10);

// // TODO: these should be msgpack buffers
// const char* PAYLOADS[] = {
// 	"Hello World!",
// 	"Hi there!",
// 	"Is anyone listening?",
// 	"Someone is always listening.",
// 	nullptr
// };

struct dummyData {
	std::string name;
	int distance;
	MSGPACK_DEFINE_MAP(name, distance);
};

int main(int argc, char* argv[])
{
  // Connect to MQTT Broker...
  string address = (argc > 1) ? string(argv[1]) : DFLT_SERVER_ADDRESS;

	cout << "Initializing for server '" << address << "'..." << endl;
	mqtt::connect_options options("tether", "sp_ceB0ss!");
	mqtt::async_client cli(address, "");

	cout << "  ...OK" << endl;


	try {
		cout << "\nConnecting..." << endl;
		cli.connect(options)->wait();
		cout << "  ...OK" << endl;

		cout << "\nPublishing messages..." << endl;

		mqtt::topic top(cli, "test", QOS);
		mqtt::token_ptr tok;

		// msgpack::type::tuple<int, bool, std::string> src(1, true, "example");
		dummyData d;
		d.name = "sensor";
		d.distance = 101;
		std::stringstream buffer;
    msgpack::pack(buffer, d);

		// Convert stringstream to const char, without copying
		// as per https://www.py4u.net/discuss/63761
		const std::string& tmp = buffer.str();   
		const char* cstr = tmp.c_str();

		tok = top.publish(cstr);

		tok->wait();	// Just wait for the last one to complete.
		cout << "OK" << endl;

		// Disconnect
		cout << "\nDisconnecting..." << endl;
		cli.disconnect()->wait();
		cout << "  ...OK" << endl;
	}
	catch (const mqtt::exception& exc) {
		cerr << exc << endl;
		return 1;
	}

 	return 0;

}