# MQTT Brokers for Tether

A Tether system always requires a single MQTT Broker that all the Agents can connect to.

This broker could be running on a dedicated machine on site (for the production version of an installation) or your own machine (when developing and testing).

The easiest way to deploy an MQTT Broker is to use Docker. To use this, make sure you have [Docker Desktop](https://www.docker.com/products/docker-desktop/) installed for your platform.

You can use our public `tether-broker` Docker container, running it as follows using custom credentials:

```
docker run -d \
  -e "USERNAME=my_user" \
  -e "PASSWORD=my_pass" \
  -p 15675:9001 \
  -p 1883:1883 \
  randomstudiotools/tether-broker
```

or straight up with the default credentials (`tether`:`sp_ceB0ss!`):

```
docker run -p 15675:9001 -p 1883:1883 randomstudiotools/tether-broker
```

If you want to run the broker as a daemon and start at launch, include the `-d` and `--restart unless-stopped` arguments:

```
docker run -d --restart unless-stopped -p 15675:9001 -p 1883:1883 randomstudiotools/tether-broker
```

Note: in case you need to run the Tether broker on a Linux ARMv7 system (e.g. Raspberry Pi 3), you can use the `randomstudiotools/tether-broker:armv7` version of the image.

We also provide Docker Compose configurations for 3 popular Brokers, namely

- Mosquitto (recommended)
- NanoMQ
- RabbitMQ

To set up Mosquitto, for example:

- In the command line, go into the relevant directory for the broker, e.g. `cd tether/brokers/mosquitto`
- Run Docker Compose: `docker compose up -d`
