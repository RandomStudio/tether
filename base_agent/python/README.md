# Python Tether agent

This module contains a Tether agent for Python.  
It has been tested with Python 3.9.

## Dependencies

This module has the following package dependencies:

- [https://pypi.org/project/paho-mqtt/](paho-mqtt)
- [https://pypi.org/project/msgpack/](msgpack)

## Usage

Using the Tether agent is quite straightforward: connect to the broker, then create inputs to subscribe to and/or outputs to publish on as desired.  
Here is an example script that outputs an ever increasing digit every second and listens for it as well:

```
from time import sleep
from tether_agent import TetherAgent

def on_message(message):
    (topic, payload) = message
    print("*** Received from topic " + topic + ": " + str(payload))

agent = TetherAgent("test", "abc123")
in_plug = agent.create_input("test")
in_plug.add_listener(on_message)
out_plug = agent.create_output("test")
agent.connect("127.0.0.1", 1883, "tether", "sp_ceB0ss!")

value = 0

while True:
    if agent.get_is_connected():
        sleep(1)
        out_plug.publish(None if value <= 0 else value)
        value += 1
```
