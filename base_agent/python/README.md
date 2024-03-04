# Python Tether agent

This module contains a Tether agent for Python.  
It has been tested with Python 3.9.

## Installation

In future, we may package this and publish to a registry (e.g. PyPi), but for now:

- Copy the `tether_agent.py` file into your own project (e.g., in the same folder as your `main.py`)
- Copy the entries in `requirements.txt` into your own `requirements.txt` as necessary

## Dependencies

Install the dependencies by running

```
python -m pip install -r requirements.txt
```

## Usage

- Create the Tether Agent instance
- Connect
- Create Inputs and/or Outputs
- Define a message handler function if you expect to receive messages, and add it to each Input Plug
- Call publish on the Output Plug each time you want to publish something (payload automatically encoded as messagepack)

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

# Don't forget to connect!
agent.connect("127.0.0.1", 1883, "tether", "sp_ceB0ss!")

value = 0

while True:
    if agent.get_is_connected():
        sleep(1)
        out_plug.publish(None if value <= 0 else value)
        value += 1
```
