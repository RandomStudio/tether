# Tether CLI Utilities

This CLI utility tool, written in Rust, provides a single binary with subcommands:

- [receive](#receive): Subscribe to all/some messages and attempt to decode them
- [send](#send): Publish a single message with optional custom payload
- [topics](#topics): Subscribe to all/some messages and parse the topic parts such as Agent Role, Agent ID and Plug Name

You can find common options by appending `--help`.

## Receive

Run with defaults: `tether receive`

More options can be found using `tether send --help`

## Send

Run with defaults: `tether receive`

More options can be found using `tether send --help`

### Note on `--message`:

This will be automatically converted into a MessagePack payload if it is valid JSON. Remember to escape characters such as `"`, `[`, `]`, `{` and `}`.

Example:

```
--message \{\"hello\":\"world\"\,\"arr\":\[1,2,3\]\}
```

## Topics

Run with defaults: `tether topics`

More options can be found using `tether topics --help`
