import paho.mqtt.client as mqtt
import msgpack


class Input:
    def __init__(self, client, name, topic):
        self.client = client
        self.name = name
        self.topic = topic
        self.subscription_mid = None
        self.is_client_connected = False
        self.callbacks = []

    def set_is_client_connected(self, is_connected):
        self.is_client_connected = is_connected

    def subscribe(self):
        if self.client.is_connected:
            (result, mid) = self.client.subscribe(
                self.topic, 0)  # TODO let user specify QoS
            if result is not mqtt.MQTT_ERR_SUCCESS:
                print("Failed to subscribe to topic " + self.topic)
            else:
                self.subscription_mid = mid
                self.client.message_callback_add(self.topic, self.on_message)
        else:
            print("Cannot subscribe; not connected to a broker")

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

    def remove_listener(self, callback):
        if callback is None:
            raise Exception("Cannot remove None listener callback")
            return

        self.callbacks = [cb for cb in self.callbacks if cb != callback]
        print("Removed listener callback from plug with name \"" +
              self.name + "\" and topic " + self.topic)

    def on_message(self, client, userdata, message):
        decoded = msgpack.unpackb(message.payload, raw=True)
        print("Received message on topic \"" + message.topic +
              "\" with payload: " + str(decoded))
        for cb in self.callbacks:
            cb((message.topic, decoded))


class Output:
    def __init__(self, client, name, topic):
        self.client = client
        self.name = name
        self.topic = topic
        self.is_client_connected = False

    def set_is_client_connected(self, is_connected):
        self.is_client_connected = is_connected

    def publish(self, payload=None):
        print("Publishing message on topic " +
              self.topic + " with payload: " + str(payload))
        self.client.publish(self.topic, msgpack.packb(
            payload, use_bin_type=True))


class TetherAgent:
    def __init__(self, agent_type, agent_id=None):
        self.agent_type = agent_type
        self.agent_id = agent_id if agent_id is not None else uuid4()
        self.client = mqtt.Client(agent_id)  # TODO expose other options here?
        self.client.on_connect = self.on_connect
        self.client.on_disconnect = self.on_disconnect
        self.client.on_subscribe = self.on_subscribe
        self.host = "127.0.0.1"
        self.port = 1883
        self.is_connected = False
        self.inputs = []
        self.outputs = []
        print("Tether Agent instance: agent_type=" +
              self.agent_type + ", agent_id=" + self.agent_id)

    def connect(self, host="127.0.0.1", port=1883, username=None, password=None, keepalive=60, local_ip=""):
        self.host = host
        self.port = port
        if username is not None:
            self.client.username_pw_set(username, password)
        self.client.connect_async(host, port, keepalive, local_ip)
        self.client.loop_start()

    def disconnect(self):
        self.client.disconnect()
        self.client.loop_stop()

    def get_is_connected(self):
        return self.is_connected

    def get_input(self, name):
        for plug in self.inputs:
            if plug.name == name:
                return plug
        return None

    def get_output(self, name):
        for plug in self.outputs:
            if plug.name == name:
                return plug
        return None

    def create_input(self, name, override_topic=None, callback=None):
        if name is None:
            raise Exception("Input name must have a value")
            return

        for plug in self.inputs:
            if plug.name == name:
                raise Exception("Input with name \"" +
                                name + "\" already exists")
                return

        topic = ("+/+/" + name) if override_topic is None else override_topic
        plug = Input(self.client, name, topic)
        self.inputs.append(plug)
        print("Created input plug: name=" + name + ", topic=" + topic)
        if self.is_connected:
            plug.subscribe()
        return plug

    def create_output(self, name, override_topic=None):
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
        print("Created output plug: name=" + name + ", topic=" + topic)
        return plug

    def on_connect(self, client, userdata, flags, result_code):
        print("Connected to MQTT broker at " + self.host + ":" +
              str(self.port) + " with result code " + str(result_code))
        for plug in self.inputs:
            plug.set_is_client_connected(True)
            plug.subscribe()
        for plug in self.outputs:
            plug.set_is_client_connected(True)
        self.is_connected = True

    def on_disconnect(self, client, userdata, response_code):
        self.is_connected = False
        print("Disconnected from MQTT" +
              (" unexpectedly" if response_code != 0 else ""))
        for plug in self.inputs:
            plug.set_is_client_connected(False)
        for plug in self.outputs:
            plug.set_is_client_connected(False)

    def on_subscribe(self, client, userdata, mid, granted_qos):
        for plug in self.inputs:
            if plug.subscription_mid == mid:
                print("Subscribed to topic: " + plug.topic)
