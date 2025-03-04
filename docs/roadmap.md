<!-- vim: set textwidth=110: -->

## `roc-toolkit` APIs

`roc-toolkit` may be used in [different ways](https://roc-streaming.org/toolkit/docs/api/reference.html):

![](./assets/dia/roc-toolkit-apis-1.svg)
/// caption
as a library for pipeline only (`roc_{sender,receiver}_{encoder,decoder}*`)
///

![](./assets/dia/roc-toolkit-apis-2.svg)
/// caption
as a library for pipeline and network components (`roc_{sender,receiver)*`)
///

![](./assets/dia/roc-toolkit-apis-3.svg)
/// caption
as a library for all three components (`roc_transceiver*`; not implemented)
///

All three components — `sndio`, `pipeline`, and `netio` — are configured:

| Component | via CLI args | via C API |
| ---:      | :---:        | :---:     |
| sndio     | ✅           | ❌        |
| pipeline  | ✅           | ✅        |
| netio     | ✅           | ✅        |
/// caption
///

## Platform support

### Windows

TBD

### iOS

TBD
