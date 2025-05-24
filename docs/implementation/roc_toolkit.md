# Use of Roc Toolkit

## About Roc Toolkit

[Roc Toolkit](https://github.com/roc-streaming/roc-toolkit/) is a library for real-time audio streaming. It implements a *transport*: you feed it audio stream, or tell it where to read it, and its job is to deliver the stream while maintaining the requested latency and quality, and recover packet losses.

On top of Roc Toolkit, `rocd` implements a P2P signaling protocol that interconnects instances and allows them to exchange configuration and events, and a high-level REST API for application developers for orchestrating streams and devices.

## Roc Toolkit API

Roc Toolkit has several [API groups](https://roc-streaming.org/toolkit/docs/api/reference.html):

**encoder/decoder API**
:    Network-less version of sender and receiver that allow user to encode audio stream into network packets (`roc_sender_encoder`) and decode them back into audio stream (`roc_receiver_decoder`). Does not implement delivery itself.

    ![](./../assets/dia/roc-toolkit-apis-1.svg)
    /// caption
    `roc_sender_encoder` example workflow
    ///

**sender/receiver API**
:    Basic single-stream version of sender and receiver. User writes audio stream to sender (`roc_sender`) and reads it from receiver (`roc_receiver`). The interface looks much like a TCP socket, but under the hood it's a real-time stream with bounded latency.

    ![](./../assets/dia/roc-toolkit-apis-2.svg)
    /// caption
    `roc_sender` example workflow
    ///

**transceiver API**
:    Advanced multi-stream sender/receiver (`roc_transceiver`). Supports multiple concurrent sending and receiving streams. The user can connect streams with audio devices, other streams (for relaying / transcoding), or custom callbacks.

    !!! warning

        `roc_transceiver` is not implemented yet.

    ![](./../assets/dia/roc-toolkit-apis-3.svg)
    /// caption
    `roc_transceiver` example workflow
    ///

## Using API in rocd

When `roc_transceiver` API is ready (see [roc-toolkit#gh-260](https://github.com/roc-streaming/roc-toolkit/issues/260)), `rocd` will use it to fully delegate real-time path to `libroc`.

![](./../assets/dia/roc-toolkit-interaction.svg)
/// caption
example connection between a few `rocd` instances using `roc_transceiver` API
///

This design is our goal. However, currently, Roc Toolkit only provides encoder/decoder and sender/receiver API groups in its C API (`libroc`), and sender/receiver + audio I/O via command-line tools (`roc-send` and `roc-recv`).

|                             | CLI tools          | C API              |
|----------------------------:|:------------------:|:------------------:|
|             encoder/decoder | *not planned*      | :white_check_mark: |
|             sender/receiver | *not planned*      | :white_check_mark: |
| sender/receiver + audio I/O | :white_check_mark: | *not planned*      |
|                 transceiver | *not planned*      | :construction:     |

To proceed with `rocd` development more quickly and independently from the Roc Toolkit schedule, we have the following plan:

1. First MVP of `rocd` will not use C API at all. It will instead invoke CLI tools (`roc-send` and `roc-recv`), creating one process for every stream. This approach has many downsides, but it allows to quickly get something working.

2. First versions of `rocd` will use `roc_sender`/`roc_receiver` instead of `roc_transceiver`, and re-implement audio I/O in Rust (more precisely, their simplified versions narrowed to `rocd` use cases).

3. When `roc_transceiver` is implemented in Roc Toolkit, we'll switch `rocd` to that new API, effectively delegating all real-time I/O and processing to it.

See also [Platform support](../usage/platforms.md) for current status.
