# Tether CLI Utilities

This CLI utility tool, written in Rust, provides a single binary with subcommands:

- [receive](#receive): Subscribe to all/some messages and attempt to decode them
- [send](#send): Publish a single message with optional custom payload
- [topics](#topics): Subscribe to all/some messages and parse the topic parts such as Agent Role, Agent ID and Plug Name
- [record](#record): Record messages to disk. Useful for simulation, in combination with `playback` below
- [playback](#playback): Playback messages with their original topics and timing, to simulate one or more Agents

## Passing arguments

There are always **two parts** to the CLI command

- The main command `tether`
  - Followed by optional parameters _for general configuration_ such as `--tether.host` or `--loglevel`
- The subcommand `receive`, `send`, `topics`, `record` or `playback`
  - Followed by optional paramaters _relating to the specific subcommand_

### Example

Here's an example of using the `receive` subcommand but specifying some non-default details for the MQTT Broker, and a non-default topic:

```
tether --tether.host 10.0.0.1 --tether.username myUserName --tether.password myPaSsWorD! receive --topic +/+/someSpecificPlug
```

## Installation

The simplest installation method is using the [homebrew](https://brew.sh/) package manager for Mac. Simply run the following two commands:

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

- Run with defaults: `tether send`
- More options can be found using `tether send --help`

### Note on `--message`:

This will be automatically converted into a MessagePack payload if it is valid JSON. Remember to escape characters such as `"`, `[`, `]`, `{` and `}`.

Example:

```
tether send --message \{\"hello\":\"world\"\,\"arr\":\[1,2,3\]\}
```

## Topics

Super useful for seeing which Agents are online, and how message topics are being parsed according to Agent Role, Agent ID and Plug Name. This utility can't see into the past (except in the case of retained messages), so keep this in mind for Agents that don't publish frequently.

- Run with defaults: `tether topics`
- More options can be found using `tether topics --help`

## Record

- Run with defaults: `tether record`
- More options can be found using `tether record --help`

By default, a file named `recording-00000000.json` (where the numbers are a timestamp) is generated in the current directory.

## Playback

- Run with defaults: `tether playback`
- More options can be found using `tether playback --help`

If you don't specify a file with `--file.path`, an included demo file (`demo.json`) will be used instead. **You probably want to specify a path to a real file, in most cases.**

> 💡Tip: loop the playback infinitely by passing `--loops.infinite`
