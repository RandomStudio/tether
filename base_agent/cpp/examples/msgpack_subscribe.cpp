// Using nlohmann/json instead of msgpack-c due to the convenience of the package offers
#include <iostream>
#include <cstdlib>
#include <string>
#include <cstring>
#include <cctype>
#include <thread>
#include <chrono>

#include <mqtt/async_client.h>
#include <nlohmann/json.hpp>

using json = nlohmann::json;

const std::string SERVER_ADDRESS("tcp://localhost:1883");
const std::string CLIENT_ID("MQTT-PubSub-Test");
const std::string TOPIC("+/+/dummyData");
const std::string TOPIC_STRING("+/+/dummyString");

const int QOS = 1;
const int N_RETRY_ATTEMPTS = 5;
const int MAX_BUFFERED_MSGS = 120;	// 120 * 5sec => 10min off-line buffering
const auto TIMEOUT = std::chrono::seconds(5);


/////////////////////////////////////////////////////////////////////////////

// Callbacks for the success or failures of requested actions.
class action_listener: public virtual mqtt::iaction_listener
{
	std::string name_;

	void on_failure(const mqtt::token& tok) override {
		std::cout << name_ << " failure";
		if (tok.get_message_id() != 0)
			std::cout << " for token: [" << tok.get_message_id() << "]" << std::endl;
		std::cout << std::endl;
	}

	void on_success(const mqtt::token& tok) override {
		std::cout << name_ << " success";
		if (tok.get_message_id() != 0)
			std::cout << " for token: [" << tok.get_message_id() << "]" << std::endl;

		auto top = tok.get_topics();
		if (top && !top->empty())
			std::cout << "\ttoken topic: '" << (*top)[0] << "', ..." << std::endl;

		std::cout << std::endl;
	}

public:
	action_listener(const std::string& name): name_(name) {}
};

/////////////////////////////////////////////////////////////////////////////

/**
 * Local callback & listener class for use with the client connection.
 * This is primarily intended to receive messages, but it will also monitor
 * the connection to the broker. If the connection is lost, it will attempt
 * to restore the connection and re-subscribe to the topic.
 */
class callback: public virtual mqtt::callback,
	public virtual mqtt::iaction_listener

{
	// Counter for the number of connection retries
	int nretry_;
	// The MQTT client
	mqtt::async_client& cli_;
	// Options to use if we need to reconnect
	mqtt::connect_options& connOpts_;
	// An action listener to display the result of actions.
	action_listener subListener_;

	// This deomonstrates manually reconnecting to the broker by calling
	// connect() again. This is a possibility for an application that keeps
	// a copy of it's original connect_options, or if the app wants to
	// reconnect with different options.
	// Another way this can be done manually, if using the same options, is
	// to just call the async_client::reconnect() method.
	void reconnect() {
		std::this_thread::sleep_for(std::chrono::milliseconds(2500));
		try {
			cli_.connect(connOpts_, nullptr, *this);
		}
		catch (const mqtt::exception& exc) {
			std::cerr << "Error: " << exc.what() << std::endl;
			exit(1);
		}
	}

	// Re-connection failure
	void on_failure(const mqtt::token& tok) override {
		std::cout << "Connection attempt failed" << std::endl;
		if (++nretry_ > N_RETRY_ATTEMPTS)
			exit(1);
		reconnect();
	}

	// (Re)connection success
	// Either this or connected() can be used for callbacks.
	void on_success(const mqtt::token& tok) override {}

	// (Re)connection success
	void connected(const std::string& cause) override {
		std::cout << "\nConnection success" << std::endl;
		std::cout << "\nSubscribing to topic '" << TOPIC << "'\n"
			<< "\tfor client " << CLIENT_ID
			<< " using QoS" << QOS << "\n"
			<< "\nPress Q<Enter> to quit\n" << std::endl;

		cli_.subscribe(TOPIC, QOS, nullptr, subListener_);
	}

	// Callback for when the connection is lost.
	// This will initiate the attempt to manually reconnect.
	void connection_lost(const std::string& cause) override {
		std::cout << "\nConnection lost" << std::endl;
		if (!cause.empty())
			std::cout << "\tcause: " << cause << std::endl;

		std::cout << "Reconnecting..." << std::endl;
		nretry_ = 0;
		reconnect();
	}

	// Callback for when a message arrives.
	void message_arrived(mqtt::const_message_ptr msg) override {
		std::cout << "Message arrived" << std::endl;
		std::cout << "\ttopic: '" << msg->get_topic() << "'" << std::endl;
		std::cout << "\tpayload: '" << msg->to_string() << "'\n" << std::endl;

		// Example supporting msgpack format only
		std::string topic = msg->get_topic();
		std::string payload = msg->to_string();
		json jsonFromMsg = json::from_msgpack(payload);

		// Check existence of key "hello". Technically a 'map' but imagine it as JSON
		if (jsonFromMsg.contains("hello")) {
			std::cout << std::setw(2) << jsonFromMsg << std::endl << jsonFromMsg["hello"] << std::endl;
		}
		else {
			std::cout << "key of 'hello' not found in received object" << std::endl;
		}

	}

	void delivery_complete(mqtt::delivery_token_ptr token) override {}

public:
	// Not sure when this gets called
	callback(mqtt::async_client& cli, mqtt::connect_options& connOpts)
		: nretry_(0), cli_(cli), connOpts_(connOpts), subListener_("Subscription") {}
};

/////////////////////////////////////////////////////////////////////////////

int main(int argc, char* argv[])
{
	// A subscriber often wants the server to remember its messages when its
	// disconnected. In that case, it needs a unique ClientID and a
	// non-clean session.
	mqtt::connect_options connOpts("tether", "sp_ceB0ss!");
	mqtt::async_client client(SERVER_ADDRESS, CLIENT_ID);
	connOpts.set_clean_session(false);

	/////
	// Install the callback(s) before connecting.
	/////
	callback mqttCallback(client, connOpts);
	client.set_callback(mqttCallback);

	/////
	// Start the connection.
	// When completed, the callback will subscribe to topic.
	/////
	try {
		std::cout << "Connecting to the MQTT server..." << std::flush;
		mqtt::token_ptr connectToken = client.connect(connOpts, nullptr, mqttCallback);
		connectToken->wait();


		/////
		// Publish a msgpack message
		/////

		// Haven't gotting JSON -> MSGpack publishing to work
		// json publishJSON = R"({ "hello": "world", "message": "publish test", "number": 12345, "array": [1,2,3,4,5] })"_json;
		// // serialize it to MessagePack
		// std::vector<std::uint8_t> publishMessage = json::to_msgpack(publishJSON);


		std::cout << "\nSending message..." << std::endl;
		mqtt::message_ptr pubmsg = mqtt::make_message(TOPIC_STRING, "Hello space cowboy!");
		pubmsg->set_qos(QOS);
		client.publish(pubmsg)->wait_for(TIMEOUT);
		std::cout << "  Message sent" << std::endl;

		std::cout << "\nSending message 2..." << std::endl;
		mqtt::topic pubTopic(client, TOPIC_STRING, QOS);
		mqtt::token_ptr tokenPointer;
		tokenPointer = pubTopic.publish("publishMessage");
		tokenPointer->wait();
		std::cout << "  Message 2 sent" << std::endl;
	}
	catch (const mqtt::exception& exc) {
		std::cerr << "\nERROR: Unable to connect to MQTT server: '"
			<< SERVER_ADDRESS << "'" << exc << std::endl;
		return 1;
	}

	/////
	// Just block till user tells us to quit.
	/////
	while (std::tolower(std::cin.get()) != 'q');

	/////
	// Disconnect
	/////
	try {
		std::cout << "\nDisconnecting from the MQTT server..." << std::flush;
		client.disconnect()->wait();
		std::cout << "OK" << std::endl;
	}
	catch (const mqtt::exception& exc) {
		std::cerr << exc << std::endl;
		return 1;
	}

	return 0;
}
