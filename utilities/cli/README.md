# Tether CLI

## Install on your system

It's very convenient to have the command available system-wide. From _this_ directory, you can run the following (possibly with `sudo`):

```
npm install -g @tether/tether-cli --registry https://registry.tether-io.dev/
```

Now you can run any of the following from the command-line without explicitly invoking NodeJS:

- `tether-receive`
- `tether-send`
- `tether-topics`
- `tether-record`
- `tether-playback`

## Override default settings

Both `tether-send` and `tether-receive` use some defaults, for convenience. Most importantly, they assume you want to connect to the public "test" server at tether-io.dev:

```
protocol: "tcp",
host: "tether-io.dev",
port: 1883,
```

You can override any of these using command-line flags, e.g.

```
tether-send --host localhost
```

In the case of `tether-receive`, it's often useful to change the topic subscription, e.g.:

```
tether-receive --host 127.0.0.1 --topic "some/group/topic"
```

---

## tether-receive

### Defaults and overrides

By default, connects to the "test broker" @ `tcp://tether-io.dev:1883`, and subscribes to ALL topics (MQTT topic wildcard `#`).

Example, overriding server and topic:

```
tether-receive --host localhost --topic some/other/topic
```

Listens for messages on the given topic, runs till `Ctrl+C`.

### Decoding

This utility does not do much more than an ordinary MQTT client, except for attempting to decode the message payload (content) using MessagePack. You can disable the decoding using `--jsonDecode=false` if you're expecting strings.

### Format as JSON

Easily pipe valid JSON array to a file:

```
node tether-receive.js --json.enabled=true > test.json
```

All logging messages (except for `fatal` level) will be suppressed. Even `^C` (Crl + C) will be handled internally and the JSON array will be closed off before actually exiting the process.

---

## tether-send

Similar defaults and options as per `tether-receive` but you might also want to set the message (JSON, but escape quote marks) and topic, e.g.:

```
tether-send --message {\"foo\":1} --topic "my/custom/topic"
```

Publishes a single message, then exits.

---

## tether-topics

This utility listens to all messages on all topics, and tries to build up a list of topics as messages are received. From topics, it also parses agent types, agent IDs and output names.

Example output:

```
{
  topics: [ 'my/custom/topic', 'tether-send/unknown/dummy' ],
  agentTypes: [ 'my', 'tether-send' ],
  agentIds: [ 'custom', 'unknown' ],
  outputNames: [ 'topic', 'dummy' ]
}
```

Runs continuously till you press `Ctrl+C`.

This utility cannot see into the "past", i.e. it will only list topics for messages it has received since it connected.

---

## Tether Record

Works almost identically to `tether-receive`, except that it does **not** attempt to decode message payloads.

You can specify a path, file name and optional (auto-appended) timestamp for the file created to save all messages.

The defaults:

```
file: {
  basePath: "./",
  baseName: "recording",
  nameIncludesTimestamp: true,
}
```

## Tether Playback
