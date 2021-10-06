# Tether 2

## Structure of this project

- `base_agent`
  - `js`: The only base agent implemented so far. You don't even have to use this (see a few examples below).
- `explorer`: A proof-of-concept of a browser-based agent which uses _both_ the JS base agent and pure-MQTT-client approaches to demonstrate input and output being passed via the browser
- `examples`
  - `nodejs`: A demo agent that uses the same JS base agent as the "explorer". It publishes messages on two separate topics every 3 seconds, and also decodes any messages it receives on the "browserData" Input Plug.
  - `arduino`: Demonstrating how a Tether-like "agent" (without needing a Base Agent) can be written for a microcontroller
- `utilities`:
  - `cli`: Sending and Receiving from the command-line. These utilities can be installed globally on the system (via npm) to be used to test, monitor and troubleshoot a Tether-based system, interacting with it in pure text.
- `rabbitmq_docker`: A Dockerfile for building RabbitMQ with the MQTT and MQTT-web plugins enabled, and a docker-compose file to map the appropriate ports.

## Running RabbitMQ with websocket/mqtt support

A `docker-compose.yml` file (and corresponding `Dockerfile`) is available in `./rabbitmq_docker`. This installs the appropriate plugins (for web/MQTT) and sets up the port mappings needed.

Run with `docker-compose up` (attach, keep running in this shell) or `docker-compose up -d` (daemonise).

To keep it persistent (even between reboots) the following worked well:

- `restart: "unless-stopped" was added to the `docker-compose.yml`
- Made sure that Docker was enabled as a service in systemd: `sudo systemctl enable docker`
- Started the container (once!) using `docker-compose up -d`

### Use with SSL

Given a valid certificate generated for a particular domain, it is possible to manually copy these (e.g. during the docker-compose build step) into a known location, then connect using `wss://tether-io.dev:15676/ws` (note the `wss` protocol, domain name - not IP address! - associated with certificate, and specific port for TLS)/

## Tether 2 compared to Tether 1

### MQTT instead of AMQP

In Tether 1 we used AMQP. For Tether 2, we use MQTT. This in turn means:

- The exchange is set to a single default `amq.topic`. A different default exchange has to be configured in the mqtt plugin, and cannot be specified on the client side.
- The NodeJS base agent can use `mqtt` library (same as browser), rather than `amqplib` (browser could not use `amqplib` anyway)
- MQTT uses `/` not `.` separators, and wildcards are different (see https://www.hivemq.com/blog/mqtt-essentials-part-5-mqtt-topics-best-practices/), e.g. `+` not `*` and `#` may only appear at the _end_ of topic/routing key (or is the only symbol)

Because MQTT does not specify the need for "exchanges" and "queues" (though RabbitMQ may use these internally), the subscription process is quite different. Subscription is on the level of the _client_ not the (channel+exhange+)queue.

This means that if we keep the concept of a defined "Input Plug" (which seems sensible, since this can then be queried/reported elsewhere) then a little bit of extra work is done to match incoming messages with the correct Input instance, since there is no inherent link between the subscription and the messages that come in. Incoming messages include a topic (a string) - that is all that is needed to match with the Input.

It also means that we cannot (and probably don't need to) prevent the end-user completely ignoring the concept of an "Input Plug" altogether, listening for incoming messages and then breaking up and interpreting topic strings in order to redirect/handle messages as they see fit.

Perhaps most importantly, MQTT-over-Websocket is a standardised and easy-to-use protocol, entirely interoperable (from the point of view of the broker and the clients) with "normal" MQTT. This makes it easy to use in the browser, without any complicated "bridging" between protocols as was the case with AMQP (or any other raw TCP socket based protocols).

### MessagePack instead of Protocol Buffers

While Protocol Buffers were efficient and client libraries were available in multiple languages, they had some serious downsides:

- Strict schemas meant that it was impossible to encode or decode messages without the accompanying schema. This is by design, of course, but for our purposes introduces all kinds of extra complexity in getting even the most basic communication going.
- For most languages, protocol buffer client libraries would have to be specially (re)compiled in order to handle new schemas. This requires a lot of work to automate in a convenient way, and each programming language (and OS) could have its own quirks.
- Protocol Buffers assume you'd rather be very precise about the layout of data and how it is typed, so intentionally makes it relatively difficult to add, remove or otherwise modify anything about the data in your messages. This makes sense in many use cases, but for us (shorter-lived projects with rapid development timelines) it just made things unecessarily hard. We value speed, flexibility and readability over strict up-front correctness, in most of our installation development work.
- It is relatively difficult to onboard new developers with a system such as Protocol Buffers, since it has a non-trivial level of complexity. Schemas have their own syntax, for example, compiling tools must be learned, and data needs to be handled carefully in strictly-typed languages.

MessagePack, by contrast, represented a good compromise in terms of performance, message size and the ability to structure data (e.g. in nested objects and/or arrays), but with numerous advantages over Protocol Buffers, including:

- The ability to decode messages without needing a schema first. This is a massive help when monitoring, debugging and troubleshooting a system. Sometimes, you just need to see what messages are in the system, with minimal setup, and decide what to _do_ with the messages later.
- The ability to encode messages without needing a schema first. This of course has pitfalls (no enforcement of naming, structure or data types) but lowers the barrier to entry (in terms of time, complexity, configuration) to getting prototypes up and running with minimal fuss.
- Like JSON (or XML, for that matter), the data fields are _named_ with ordinary string "keys", and therefore in principle the messages can be self-describing and the contents self-explanatory. This puts an onus on the developer to name and structure things in a sensible way, of course, but when the alternative is either no structure, arbitrary structure (e.g. order of parameters), or overly-prescriptive (developer must define everything in advance), then at least the temptation to take shortcuts is minimised.
- The need for detailed documentation is lowered insofar as the messages themselves are accessible and readable in a working system. You do not _have_ to know server IPs, ports and message routing paths or have pre-compiled client libraries just to see the contents of messages.
- MessagePack is easy for new developers to pick up and use. Onboarding a new person onto the team is not difficult.

By switching from Protocol Buffers to MessagePack, we lost nothing in terms of wide support across multiple languages, platforms and even existing software such as game engines.

MessagePack really feels like "JSON, but less javascript-specific and more efficient".

### Standardisation instead of rules

In the first version of Tether, we had some complex, multi-stage processes for "agents" to gain the necessary information to even _start_ reading or publishing messages. The system assumed a centralised way of configuring and giving "permission" to publish or read anything.

Tether 2 does away with just about all of this complexity. We use a standard messaging protocol (MQTT) in a standard way. MessagePack allows the developer to be as structured as they like with data, and we do not (so far) add any further restrictions on message format.

The only "Tether-like" parts of the system are:

- A standardised, 3 part topic route convention (`agent-type/grouping-or-id/plug-name`)
- A default IP address or hostname for the "Tether Server" (really just an MQTT broker)
- The expectation that MessagePack will be used for the contents of the messages

As long as client applications conform to the above standards, they will be able to publish and read messages in a Tether system. The aim is to make it quick and easy to get messaging working within a distributed system, even with very diverse programming languages, hardware and software applications.

Various tools, naming conventions, permissions and choices of architecture can be built on top of this system. There is no guarantee that every Tether-like system will work perfectly or behave in the same way, but at least the hard part - distributed messaging - is "solved" so that developers can concentrate on more interesting concerns.

### The result

The combination of these two simple and robust technologies (MQTT and MessagePack) means that a Tether system is just about the _easiest and quickest_ way to get parts of a distributed system talking to each other. It requires very little code, minimal APIs and very little network configuration.

Other approaches (websocket servers, OSC, etc.) may sometimes appear easier to reach for in certain circumstances, but typically do not offer the flexibility of a "pub/sub" messaging system or a structured (but very transparent) data structure in the messages.

The fact that the system can be used very easily in everything from websites to microcontrollers to game engines is incredibly powerful.

## TODOs

- [x] use MQTT library for both NodeJS and browser
- [x] use same base agent in both NodeJS and browser
- [x] createOutput returns an Output object, but createInput does not... should be consistent?
- [x] demonstrate a simple CLI client (send and receive)
- [x] demonstrate microcontroller client (Arduino): without even a base agent, and no need for Plugs
- [x] tether agent should be able to disconnect (clean up, unsubscribe)
- [ ] topic patterns could be agent/id-or-group/plug ?
- [x] tether CLI client should be something you can install for the system
- [ ] proper event type definition for on "message", and the order of topic/payload should be payload, topic!
- [ ] call `.publish()` with no params should send empty message (`Buffer from([])`)
- [ ] allow client to get currently-applied Tether config?
- [ ] announcement/heartbeat/ping-pong messages standard?
- [ ] include msgpack encoding/decoding in Tether base agent?
