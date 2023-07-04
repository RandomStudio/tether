# Tether CLI Utilities

This CLI utility tool, written in Rust, provides a single binary with subcommands:

- [receive](#receive): Subscribe to all/some messages and attempt to decode them
- [send](#send): Publish a single message with optional custom payload
- [topics](#topics): Subscribe to all/some messages and parse the topic parts such as Agent Role, Agent ID and Plug Name
- [record](#record): Record messages to disk. Useful for simulation, in combination with `playback` below
- [playback](#playback): Playback messages with their original topics and timing, to simulate one or more Agents

You can find common options by appending `--help` _before_ the subcommand.

> 💡Tip: append `--tether.host tether-io.dev` to use a preconfigured test server if you don't want to set up an MQTT Broker on you own machine.

## Installation

The simplest installation method is using the [homebrew](https://brew.sh/) pacaage manager for Mac. Simply run the following two commands:

```
brew tap randomstudio/tether
brew install tether
```

(The formula and release files are hosted at https://github.com/RandomStudio/homebrew-tether - but you don't need to know that.)

---

## Receive

- Run with defaults: `tether receive`
- More options can be found using `tether send --help`

## Send

- Run with defaults: `tether receive`
- More options can be found using `tether send --help`

### Note on `--message`:

This will be automatically converted into a MessagePack payload if it is valid JSON. Remember to escape characters such as `"`, `[`, `]`, `{` and `}`.

Example:

```
--message \{\"hello\":\"world\"\,\"arr\":\[1,2,3\]\}
```

## Topics

- Run with defaults: `tether topics`
- More options can be found using `tether topics --help`

## Record

- Run with defaults: `tether record`
- More options can be found using `tether record --help`

By default, a file named `recording-00000000.json` (where the numbers are a timestamp) is generated in the current directory.

## Playback

- Run with defaults: `tether playback`
- More options can be found using `tether playback --help`

If you don't specify a file with `--file.path`, an included demo file (`demo.json`) will be used instead. You probably want to specify a path to a real file, in most cases.

> 💡Tip: loop the playback infinitely by passing `--loops.infinite`
