# Roadmap

## Milestones

### pre-1 (core functionality) [#16](https://github.com/roc-streaming/rocd/issues/16)

- [ ] prototype: devices, streams, API (1) -- [#32](https://github.com/roc-streaming/rocd/issues/32), ...
  - [ ] devices: pipewire based on C API bindings
    - [ ] streaming devices: for working with pipewire modules, roc-pulse or roc-vad
  - [ ] streams: CRUD, send, receive
- [ ] tests: integration (single process): openapi-based generated client class that interacts with rocd server + PW backend mock

### pre-2 (core functionality)

- [ ] ZMQ: protocol
- [ ] cross-management

### pre-3

- [ ] user configuration (plain text) [#9](https://github.com/roc-streaming/rocd/issues/9)
- [ ] administration panel (UI for stable REST API) [#17](https://github.com/roc-streaming/rocd/issues/17)
- [ ] documentation [#21](https://github.com/roc-streaming/rocd/issues/21)

### later

- [ ] tests: integration (black box; multiprocess): dockerized client + server

---

### after pre-1

- [ ] convert `roc-droid` -> `roc-cast`
- [ ] use rocd in `roc-cast` on desktop

### after pre-3

- [ ] **public announce**


## Platform support

### Linux

### macOS

### Windows

### Android

### iOS
