# Tether CLI Utilities

This CLI utility tool, written in Rust, provides a single binary with subcommands:

- [receive](#receive): Subscribe to all/some messages and attempt to decode them
- [send](#send): Publish a single message with optional custom payload
- [topics](#topics): Subscribe to all/some messages and parse the topic parts such as Agent Role, Agent ID and Plug Name

You can find common options by appending `--help`.

> 💡Tip: append `--tether.host tether-io.dev` to use a preconfigured test server if you don't want to set up an MQTT Broker on you own machine.

## Installation

The simplest installation method is using the [homebrew](https://brew.sh/) pacaage manager for Mac. Simply run the following two commands:

```
brew tap randomstudio/tether
brew install tether
```

(The formula and release files are hosted at https://github.com/RandomStudio/homebrew-tether - but you don't need to know that.)

## Subcommands

### Receive

Run with defaults: `tether receive`

More options can be found using `tether send --help`

### Send

Run with defaults: `tether receive`

More options can be found using `tether send --help`

#### Note on `--message`:

This will be automatically converted into a MessagePack payload if it is valid JSON. Remember to escape characters such as `"`, `[`, `]`, `{` and `}`.

Example:

```
--message \{\"hello\":\"world\"\,\"arr\":\[1,2,3\]\}
```

### Topics

Run with defaults: `tether topics`

More options can be found using `tether topics --help`

## TODO

- [x] Tether Record
- [ ] Tether Record could include options for delayed start, timed end, zero at first entry (default)
