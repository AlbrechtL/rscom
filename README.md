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

## Macro send modes

By default a macro sends its entire command in one write. For targets that drop characters when flooded (e.g. U-Boot prompts), two slower modes are available per macro:

| Field | Behaviour |
|---|---|
| `char_delay_ms: N` | Send one character, wait N milliseconds, then send the next. |
| `echo_timeout_ms: N` | Send one character and wait until the device echoes it back before sending the next. If no echo arrives within N milliseconds, the next character is sent anyway (safety fallback for non-echoed bytes such as `\r`/`\n`). |

The two fields are mutually exclusive — setting both is a configuration error.

```yaml
macros:
  - key: F1
    description: slow fixed delay
    command: "bootm"
    char_delay_ms: 20          # 20 ms between each character

  - key: F2
    description: echo-gated send
    command: "printenv"
    echo_timeout_ms: 500       # wait for echo; fall through after 500 ms

  - key: F3
    description: instant send (default)
    command: "saveenv"
```

## Linux permission note

If opening the serial device fails with permission denied, add your user to the correct group (often `dialout`) and re-login.
