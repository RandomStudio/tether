# Python Tether agent

This module contains a Tether agent for Python.  
It has been tested with Python 3.11 (the same version currently used by TouchDesigner)

## Installation

```
pip install tether_agent
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

## Developer / packager notes
Create (if necessary) and activate a virtual environment for building/publishing, using pyenv and virtualenv:
```
pyenv virtualenv tether_build
pyenv activate tether_build
```

Upgrade (if necessary) Python Packaging build:
```
pip install --upgrade build
```

Install Twine (distribution uploading):
```
pip install --upgrade twine
```

Upload to TEST pypi
```
python -m twine upload --repository testpypi dist/*
```

Upload to ACTUAL pypi
```
python -m twine upload dist/*
```
