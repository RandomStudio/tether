from enum import Enum
from uuid import uuid4
import logging
import paho.mqtt.client as mqtt
import msgpack

# Convenience enum purely meant to reduce the need for additional imports in the script that uses this module


class LogLevel(Enum):
    DEBUG = logging.DEBUG
    INFO = logging.INFO
    WARNING = logging.WARNING
    ERROR = logging.ERROR
    CRITICAL = logging.CRITICAL

# Tether Agent plug to subscribe to incoming data. Create these via `tether_agent.create_input(name, topic?)`
# and call `add_listener(callback)` to execute a callback function when messages come in on this topic.


class Input:
    def __init__(self, client, name, topic, qos=0):
        self.client = client
        self.name = name
        self.topic = topic
        self.qos = qos
        self.subscription_mid = None
        self.is_client_connected = False
        self.callbacks = []

    def set_is_client_connected(self, is_connected):
        self.is_client_connected = is_connected

    def subscribe(self):
        if self.client.is_connected:
            (result, mid) = self.client.subscribe(self.topic, self.qos)
            if result is not mqtt.MQTT_ERR_SUCCESS:
                logging.warning("Failed to subscribe to topic " + self.topic + (
                    "; client is not connected" if result is mqtt.MQTT_ERR_NO_CONN else "; reason unknown"
                ))
            else:
                self.subscription_mid = mid
                self.client.message_callback_add(self.topic, self.on_message)
        else:
            logging.warning("Cannot subscribe; not connected to a broker")

    def add_listener(self, callback):
        if callback is None:
            raise Exception("Callback function for listener cannot be None")
            return

        # TODO for Python 3.x but before 3.2, use hassattr(cb, '__call__)
        if not callable(callback):
            raise Exception("Invalid callback; listener must be callable")
            return

        for cb in self.callbacks:
            if cb == callback:
                raise Exception("The provided callback is already registered")
                return

        self.callbacks.append(callback)
        logging.debug("Added listener callback to plug with name \"" +
                      self.name + "\" and topic " + self.topic)

    def remove_listener(self, callback):
        if callback is None:
            raise Exception("Cannot remove None listener callback")
            return

        self.callbacks = [cb for cb in self.callbacks if cb != callback]
        logging.debug("Removed listener callback from plug with name \"" +
                      self.name + "\" and topic " + self.topic)

    def on_message(self, client, userdata, message):
        decoded = msgpack.unpackb(message.payload, raw=True)
        logging.debug("Received message on topic \"" +
                      message.topic + "\" with payload: " + str(decoded))
        for cb in self.callbacks:
            cb((message.topic, decoded))

# Tether Agent output plug, used for publishing of data via a specific topic.
# Create an Output using `tether_agent.create_output(name, topic?)` and publish data viA `plug.publish(payload?)`.


class Output:
    def __init__(self, client, name, topic, qos=0, retain=False):
        self.client = client
        self.name = name
        self.topic = topic
        self.qos = qos
        self.retain = retain
        self.is_client_connected = False

    def set_is_client_connected(self, is_connected):
        self.is_client_connected = is_connected

    def publish(self, payload=None):
        logging.debug("Publishing message on topic " +
                      self.topic + " with payload: " + str(payload))
        self.client.publish(self.topic, msgpack.packb(
            payload, use_bin_type=True), self.qos, self.retain)

# Convenience class to connect to an MQTT broker and interface with it by publishing data and subscribing to topics.


class TetherAgent:
    # Constructor. Expects an agent type and id, which are used in the (automatically generated) topics that it publishes data to.
    def __init__(self, agent_type, agent_id=None, loglevel=LogLevel.INFO):
        self.agent_type = agent_type
        self.agent_id = agent_id if agent_id is not None else str(uuid4())
        self.client = mqtt.Client(agent_id)  # TODO expose other options here?
        self.client.on_connect = self.on_connect
        self.client.on_disconnect = self.on_disconnect
        self.client.on_subscribe = self.on_subscribe
        self.host = "127.0.0.1"
        self.port = 1883
        self.is_connected = False
        self.inputs = []
        self.outputs = []
        logging.basicConfig(
            level=loglevel.value, format='[%(asctime)s] %(levelname)s : %(module)s : %(message)s', datefmt='%Y/%m/%d %H:%M:%S')
        logging.info("Tether Agent instance: agent_type=" +
                     self.agent_type + ", agent_id=" + self.agent_id)

    # Connect to an MQTT broker.
    # @param host The location of the MQTT broker on the network. Defaults to localhost.
    # @param port The port over which to connect to the broker. Defaults to 1883.
    # @param username The username to use to authenticate to the broker, if required. Defaults to `None`` (i.e. no authentication).
    # @param password The password to use when authenticating to the broker, if any. Only used if `username` is not `None`. Defaults to `None`.
    # @param keepalive Maximum delay in seconds between messages. If no messages are sent for this amount of time, this will be the ping rate.
    # @param local_ip The local network interface to use for the MQTT connection. You only need to provide this value if you require a specific interface to be used.
    def connect(self, host="127.0.0.1", port=1883, username="tether", password="sp_ceB0ss!", keepalive=60, local_ip=""):
        self.host = host
        self.port = port
        if username is not None:
            logging.debug("Setting username" + ((" and password")
                          if password is not None else ""))
            self.client.username_pw_set(username, password)
        logging.info("Connecting to MQTT at " + host + ":" + str(port) +
                     ((" via local interface " + local_ip) if local_ip is not None else ""))
        self.client.connect_async(host, port, keepalive, local_ip)
        self.client.loop_start()

    # Disconnect from the MQTT broker
    def disconnect(self):
        logging.info("Disconnecting from MQTT broker")
        self.client.disconnect()
        self.client.loop_stop()

    def get_is_connected(self):
        return self.is_connected

    # Retrieve an input plug with the given name, if any. The plug contains
    # its name and the topic it subscribes to.
    def get_input(self, name):
        for plug in self.inputs:
            if plug.name == name:
                return plug
        return None

    # Retrieve an output plug with the given name, if any. The plug contains
    # its name and the topic it publishes to.
    def get_output(self, name):
        for plug in self.outputs:
            if plug.name == name:
                return plug
        return None

    # Create a new input plug. Note that only one input plug is allowed to exist with any unique name.
    # @param name The name of the input plug. This is used for easy reference, as well as in the topic that the plug subscribes to, unless an `override_topic` is specified.
    # @param qos The quality of service expected from messages received on this plug.
    # @param override_topic A manually defined topic that this plug should subscribe to, rather than the default `+/+/<plug name>`.
    # @param callback A callback function to execute whenever messages arrive on this plug. The handler function should expect a tuple of (topic, payload).
    def create_input(self, name, qos=0, override_topic=None, callback=None):
        if name is None:
            raise Exception("Input name must have a value")
            return

        for plug in self.inputs:
            if plug.name == name:
                raise Exception("Input with name \"" +
                                name + "\" already exists")
                return

        topic = ("+/+/" + name) if override_topic is None else override_topic
        plug = Input(self.client, name, topic, qos)
        if callback is not None:
            plug.add_listener(callback)
        self.inputs.append(plug)
        logging.debug("Created input plug: name=" + name + ", topic=" + topic)
        if self.is_connected:
            plug.subscribe()
        return plug

    # Create a new output plug. Note that only one output plug is allowed to exist with any unique name.
    # @param name The name of the output plug. This is used for easy reference, as well as in the topic that the plug publishes to, unless an `override_topic` is specified.
    # @param qos Quality of service used for MQTT messaging from this plug.
    # @param retain Whether or not the messages published from this plug should be retained.
    # @param override_topic A manually defined topic that this plug should publish to, rather than the default `<agent type>/<agent id>/<plug name>`.
    def create_output(self, name, qos=0, retain=False, override_topic=None):
        if name is None:
            raise Exception("Output name must have a value")
            return

        for plug in self.outputs:
            if plug.name == name:
                raise Exception("Output with name \"" +
                                name + "\" already exists")
                return

        topic = (self.agent_type + "/" + self.agent_id + "/" +
                 name) if override_topic is None else override_topic
        plug = Output(self.client, name, topic)
        self.outputs.append(plug)
        logging.debug("Created output plug: name=" + name + ", topic=" + topic)
        return plug

    # Connection event handler. This updates the plugs' connection state and will attempt to let all input plugs resubscribe.
    def on_connect(self, client, userdata, flags, result_code):
        logging.info("Connected to MQTT broker at " + self.host + ":" +
                     str(self.port) + " with result code " + str(result_code))
        for plug in self.inputs:
            plug.set_is_client_connected(True)
            plug.subscribe()
        for plug in self.outputs:
            plug.set_is_client_connected(True)
        self.is_connected = True

    # Disconnection event handler. Updates the plugs' connection state.
    def on_disconnect(self, client, userdata, response_code):
        self.is_connected = False
        logging.info("Disconnected from MQTT" +
                     (" unexpectedly" if response_code != 0 else ""))
        for plug in self.inputs:
            plug.set_is_client_connected(False)
        for plug in self.outputs:
            plug.set_is_client_connected(False)

    # Subscription handler. For event logging purpoises only.
    def on_subscribe(self, client, userdata, mid, granted_qos):
        for plug in self.inputs:
            if plug.subscription_mid == mid:
                logging.debug("Subscribed to topic: " + plug.topic)


if __name__ == "__main__":
    from time import sleep

    def on_message(message):
        (topic, payload) = message
        print("Received message on topic " + topic + ": " + str(payload))

    agent = TetherAgent("test", "test", LogLevel.DEBUG)
    in_plug = agent.create_input("test")
    in_plug.add_listener(on_message)
    out_plug = agent.create_output("test")
    agent.connect("127.0.0.1", 1883, "tether", "sp_ceB0ss!")

    while True:
        if agent.get_is_connected():
            sleep(1)
            out_plug.publish(3.1415926536)
            sleep(1)
            quit()
