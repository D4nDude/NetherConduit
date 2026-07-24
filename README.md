# NetherConduit

A Minecraft proxy written in Rust.

> [!WARNING]
> PRE-ALPHA — NOT READY FOR PRODUCTION USE

## Overview

NetherConduit is an open-source Minecraft proxy written in Rust.

The project aims to provide a modular, extensible proxy architecture, with a focus on low latency and low memory usage.

NetherConduit is currently in the very early stages of development and is not suitable for production use.

### Project Status

NetherConduit is currently under active development.

## Roadmap

This roadmap outlines planned features and may change as the project develops.

### Core Proxy

- [ ] Protocol Primitives
- [ ] Basic Relay:
  - [ ] Accept TCP connections from clients
  - [ ] Connect to a single hardcoded backend
  - [ ] Bidirectional relay
- [ ] Packet Inspection:
  - [ ] Parse packet types
  - [ ] State-aware packet parsing
  - [ ] Inline core dispatcher
- [ ] Multi-backend:
  - [ ] Backend registry
  - [ ] Config-based routing
  - [ ] Client authentication
  - [ ] Backend handshake on behalf of player
- [ ] Server Switching:
  - [ ] Swappable server components in state
  - [ ] Dimension switch client sequence
  - [ ] Programmatic packet injection

### Proxy Infrastructure

- [ ] Reloadable configuration
- [ ] CLI
- [ ] Player commands
- [ ] Docker image
- [ ] External logging
- [ ] Shared state:
  - [ ] Player registry
  - [ ] Proxy-wide event bus
  - [ ] Player events
  - [ ] Broadcast messaging

### Extensibility

- [ ] Native LuckPerms integration
- [ ] Config-based plugins
- [ ] Scripting
- [ ] WASM/native plugins

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md).

## Contributing

NetherConduit is currently under active development and is expected to evolve significantly before its first stable release.

Pull requests are welcome, but major architectural changes may not be accepted while the core design is still evolving. Contribution guidelines will be expanded as the project approaches MVP.

To build and run the project from source:

```bash
cargo run
```

## Licence

NetherConduit is licensed under the [MIT Licence](LICENSE).
