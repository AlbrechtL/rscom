# rscom

`rscom` is a command-line serial terminal emulator for embedded development workflows (for example U-Boot over UART). It uses a YAML file for full runtime configuration and supports keyboard-triggered command macros on `F1..F12`.

## Features in current implementation

- Serial terminal over UART (`/dev/ttyUSB*`)
- YAML-based configuration
- Runtime overrides for device and baud rate (`--device`, `--baud`)
- Macro execution with `F1..F12` (sends command + Enter)
- Reconnect on unplug/replug with backoff
- Optional RX/TX session logging

## Build

```bash
cargo build
```

## Run

```bash
cargo run -- --config examples/rscom.yaml
```

Optional overrides:

```bash
cargo run -- --config examples/rscom.yaml --device /dev/ttyUSB1 --baud 921600
```

## Runtime keys

- `F1..F12`: send configured macro command with auto Enter
- `Ctrl+C`: forward ETX (`0x03`) to target serial device
- `Ctrl+A`, then `x`: exit rscom immediately

## YAML example

Use [examples/rscom.yaml](examples/rscom.yaml) as a starting point.

## Linux permission note

If opening the serial device fails with permission denied, add your user to the correct group (often `dialout`) and re-login.
