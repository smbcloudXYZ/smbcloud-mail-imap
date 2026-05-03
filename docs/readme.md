# Running on a server

These are quick notes for running the IMAP service manually while you are still in the prototype stage.

## Run in the background

```bash
cargo run --release > output.log 2>&1 &
```

## Check the listener

For a local IMAP listener on port `1143`:

```bash
sudo lsof -nP -iTCP:1143 -sTCP:LISTEN
```

If you change the port, update the command to match.

## Note

This is only a rough operator note for development and manual testing. It is not a deployment guide yet.
