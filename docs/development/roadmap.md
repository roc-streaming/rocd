# Roadmap

## Prototype Stage I (Basis)

- :construction: **Design docs** ([gh-15][gh-15])
    - architecture, relations to roc-toolkit and roc-cast
- :white_check_mark: **Daemon skeleton** ([gh-16][gh-16])
    - REST API
    - OpenAPI (Swagger)
    - CLI, logging
- :white_large_square: **Integration tests skeleton** ([gh-49][gh-49])
    - driver mock + storage + port and stream dispatchers + rest api server
    - swagger-generated client
    - send requests via client and check how driver mock behaves
- :white_check_mark: **Persistent storage** ([gh-32][gh-32])
    - persistent storage for runtime state
- :white_large_square: **DeviceDriver and PipewireDriver** ([gh-50][gh-50])
    - generic driver interface
    - implementation for PipeWire using C bindings to libpipewire
- :white_large_square: **Port management** ([gh-46][gh-46])
    - CRUD for io ports (audio devices)
    - uses DeviceDriver to control devices
- :white_large_square: **Stream management** ([gh-47][gh-47])
    - CRUD for network streams
    - uses DeviceDriver for I/O
    - support sending from port to address
    - support receiving from address to port
    - for now don't support port-to-port
- :white_large_square: **Event subscription** ([gh-48][gh-48])
    - REST API to subscribe to events
    - integrate with PortDispatcher and StreamDispatcher

[gh-15]: https://github.com/roc-streaming/rocd/issues/15
[gh-16]: https://github.com/roc-streaming/rocd/issues/16
[gh-32]: https://github.com/roc-streaming/rocd/issues/32
[gh-46]: https://github.com/roc-streaming/rocd/issues/46
[gh-47]: https://github.com/roc-streaming/rocd/issues/47
[gh-49]: https://github.com/roc-streaming/rocd/issues/49
[gh-50]: https://github.com/roc-streaming/rocd/issues/50

## Prototype Stage II (P2P)

- :white_large_square: **P2P protocol**
    - choose technology (ZMQ, libp2p)
    - protocol spec
    - protocol implementation
- :white_large_square: **Peer management**
    - sessions and authentication
    - auto-discovery
    - REST API for /peers
- :white_large_square: **Cross-peer streams**
    - support streams connecting ports on different peers

## Prototype Stage III (Extras)

- :white_large_square: **User config file** ([gh-9][gh-9])
    - YAML file for static configuration
- :white_large_square: **Web admin** ([gh-17][gh-17])
    - simple web interface
- :white_large_square: **Full documentation** ([gh-21][gh-21])
    - complete documentation

[gh-9]: https://github.com/roc-streaming/rocd/issues/9
[gh-17]: https://github.com/roc-streaming/rocd/issues/17
[gh-21]: https://github.com/roc-streaming/rocd/issues/21
[gh-48]: https://github.com/roc-streaming/rocd/issues/48
