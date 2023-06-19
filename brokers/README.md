# MQTT Brokers for Tether

A Tether system always requires a single MQTT Broker that all the Agents can connect to.

This broker could be running on a dedicated machine on site (for the production version of an installation) or your own machine (when developing and testing).

The easiest way to deploy an MQTT Broker is to use Docker. We provide Docker Compose configurations for 3 popular Brokers, namely

- Mosquitto (recommended)
- NanoMQ
- RabbitMQ

To set up Mosquitto, for example:

- Make sure you have [Docker Deskto](https://www.docker.com/products/docker-desktop/) installed for your platform
- In the command line, go into the relevant directory for the broker, e.g. `cd tether/brokers/mosquitto`
- Run Docker Compose: `docker compose up -d`
