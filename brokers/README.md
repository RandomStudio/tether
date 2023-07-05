# MQTT Brokers for Tether

A Tether system always requires a single MQTT Broker that all the Agents can connect to.

This broker could be running on a dedicated machine on site (for the production version of an installation) or your own machine (when developing and testing).

The easiest way to deploy an MQTT Broker is to use Docker. To use this, make sure you have [Docker Desktop](https://www.docker.com/products/docker-desktop/) installed for your platform.

You can use our public `tether-broker` Docker container, running it as follows using custom credentials:

```
docker run -d \
  -e "USERNAME=my_user" \
  -e "PASSWORD=my_pass" \
  -p 9001:15675 \
  -p 1883:1883 \
  tether-broker
```

or straight up with the default credentials (`guest`:`guest`):

```
docker run -p 9001:15675 -p 1883:1883 tether-broker
```

We also provide Docker Compose configurations for 3 popular Brokers, namely

- Mosquitto (recommended)
- NanoMQ
- RabbitMQ

To set up Mosquitto, for example:

- In the command line, go into the relevant directory for the broker, e.g. `cd tether/brokers/mosquitto`
- Run Docker Compose: `docker compose up -d`
