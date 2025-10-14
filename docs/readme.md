# Run on server

```bash
$ cargo run --release -p nginx_auth_http > output.log 2>&1 &
$ cargo run --release > output.log 2>&1 &
```

## Find all TCP processes

Find all running processes:

```bash
$ sudo lsof -nP -iTCP:2525 -sTCP:LISTEN
```
