<!-- vim: set textwidth=110: -->

# `roc-cast`

`roc-cast` is a component used for two primary purposes:

- Provide UI for user interaction with rocd.
- On Android (and — possibly — iOS) devices it implements a wrapper application with a so-called main activity
  context to allow interaction between non-Java libraries (C/C++/...) with mobile sound devices.

## Implementation

`roc-cast` provides two interfaces:
- native UI (Flutter) and
- web UI.

The latter may be used on headless systems.

User configures `roc` (sound devices, network, pipeline) via native `roc-cast` client (primary way) or
its web client. On mobile platforms it is always `roc-cast` client. On embeded platforms the default way is
HTTP client.

![](./assets/dia/user--rocd-interaction.svg)
/// caption
///

| Platform    | `roc-cast`: native | `roc-cast`: web (HTTP) |
| ---:        | :---:              | :---:                  |
| PC          |  ✅                | ✅                     |
| Android/iOS |  ✅                | ❌                     |
| embedded    |  ❌                | ✅                     |
/// caption
Supported clients for each platform
///


<!--
```plantuml
component "host 1" {
  [roc-cast 1] -- [rocd 1]
}

component "host 2" {
  [roc-cast 2] -- [rocd 2]
}

component "host 3" {
  [web ui] -- [rocd 3]
}

"rocd 1" -- "rocd 2": ZMQ
"rocd 2" -- "rocd 3": ZMQ
"rocd 3" -- "rocd 1": ZMQ
```
-->
