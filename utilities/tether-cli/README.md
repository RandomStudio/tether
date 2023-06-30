# Tether CLI Utilities

## tether-send

### `--message`:

This will be automatically converted into a MessagePack payload if it is valid JSON. Remember to escape characters such as `"`, `[`, `]`, `{` and `}`.

Example:

```
--message \{\"hello\":\"world\"\,\"arr\":\[1,2,3\]\}
```
