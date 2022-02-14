# Tether CLI

## Install on your system

It's very convenient to have the command available system-wide. From _this_ directory, you can run the following (possibly with `sudo`):

```
npm install -g .
```

Now you can simply launch `tether-send` or `tether-receive` from the command line without invoking `node` or anything:

```
$ tether-receive
[2022-02-14T12:46:25.058] [INFO] tether-receive - Connecting to MQTT broker @ tcp://tether-io.dev:1883 ...
[2022-02-14T12:46:25.230] [INFO] tether-receive - ...connected OK
[2022-02-14T12:46:25.231] [INFO] tether-receive - Subscribing to topic "#"
```

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

## tether-receive

### Format as JSON

Easily pipe valid JSON array to a file:

```
node tether-receive.js --json.enabled=true > test.json
```

All logging messages (except for `fatal` level) will be suppressed. Even `^C` (Crl + C) will be handled internally and the JSON array will be closed off before actually exiting the process.
