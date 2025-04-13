# Roadmap

## Milestones

### pre-1 (core functionality) [#16](https://github.com/roc-streaming/rocd/issues/16)

- :white_large_square: prototype: devices, streams, API (1) -- [#32](https://github.com/roc-streaming/rocd/issues/32), ...
  - :white_large_square: devices: pipewire based on C API bindings
    - :white_large_square: streaming devices: for working with pipewire modules, roc-pulse or roc-vad
  - :white_large_square: streams: CRUD, send, receive
- :white_large_square: tests: integration (single process): openapi-based generated client class that interacts with rocd server + PW backend mock

### pre-2 (core functionality)

- :white_large_square: ZMQ: protocol
- :white_large_square: cross-management

### pre-3

- :white_large_square: user configuration (plain text) [#9](https://github.com/roc-streaming/rocd/issues/9)
- :white_large_square: administration panel (UI for stable REST API) [#17](https://github.com/roc-streaming/rocd/issues/17)
- :white_large_square: documentation [#21](https://github.com/roc-streaming/rocd/issues/21)

### later

- :white_large_square: tests: integration (black box; multiprocess): dockerized client + server

---

### after pre-1

- :white_large_square: convert `roc-droid` -> `roc-cast`
- :white_large_square: use rocd in `roc-cast` on desktop

### after pre-3

- :white_large_square: **public announce**


## Platform support

### Linux

### macOS

### Windows

### Android

### iOS
