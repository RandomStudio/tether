## Self-signed certiicate

Using

```
openssl req -x509 -nodes -sha256 -days 365 -newkey rsa:2048 -keyout self-signed.key -out self-signed.crt
```

Leaving FQDN/Common Name blank
[TCP Segment Len: 0]
