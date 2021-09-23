# Tether CLI

## tether-receive

### Format as JSON

Easily pipe valid JSON array to a file:

```
node tether-receive.js --json.enabled=true > test.json
```

All logging messages (except for `fatal` level) will be suppressed. Even `^C` (Crl + C) will be handled internally and the JSON array will be closed off before actually exiting the process.
