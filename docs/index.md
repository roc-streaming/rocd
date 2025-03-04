<!-- vim: set textwidth=110: -->

# Overview

We start from a small preview of a system as a whole to understand what is the role of `rocd` component. The
purpose of the [Roc Toolkit](https://roc-streaming.org/toolkit/docs/about_project/overview.html) (further
referred to as `roc`) is to implement real-time audio streaming over network.

`rocd` handles devices and streams, and connects ones with others. Its core functions are:

- create, read, update and delete (CRUD) metainfo about system devices (speakers, microphones);
- CRUD metainfo about streaming devices (virtual speakers and microphones passed through network);
- find other rocd instances over network.

See also [Entities](#entities) section.

That is, `rocd` is intended to be used for cooperating multiple sound I/O devices into a system where they can
be orchestrated together as a whole in a wide variety of different ways.

In Roc Streaming, we intend to use `rocd` in junction with the [`roc-cast`](roc-cast.md).

For implementation status, refer to the [Roadmap](roadmap.md) page.

## Entities

Here are the top-level components:

**host**
:   Computer where the `roc` instance is currently running on. Computer platform may be one of:

    - PC — a personal computer, usually a desktop device with monitor or a similar output device; GNU/Linux and
      other *nix, macOS, Windows ([future plans](roadmap.md#windows));
    - Android;
    - iOS ([future plans](roadmap.md#ios));
    - embedded GNU/Linux — can be similar to PC, but often doesn't have a monitor;
    - server — a computer without sound devices that hosts `roc` software in a proxy/hub mode.

**rocd**
:   Service component used for stream management and auto discovery over network. Internally it has a library
    component because of mobile platforms' specific limitations (programming language, permission models,
    ...). See [Mobile-specific](#mobile-specific) for details.

[**roc-cast**](roc-cast.md)
:   Component that provides a user-friendly UI and business logic on the top of `rocd` specific to its
    use-cases.

[**roc-toolkit**](https://roc-streaming.org/toolkit/docs/index.html)
:   The core library for real-time streaming. Can be used for some or all of the following features: sound
    device I/O, processing pipeline, network I/O. In the MVP stage, `rocd` will use it via command-line tools
    for simplicity, later we'll switch to the C API.

## Usage cases

Here `roc` may be used as a binary only as it doesn't currently implement C API for all 3 components:

![](./assets/dia/simple.svg)
/// caption
Simple case: `roc` implements all steps between input and output sound devices
///

<!-- TODO: add more use case examples here -->

![](./assets/dia/general.svg)
/// caption
...
///

## Implementation

`rocd` sets up a corresponding `roc-toolkit` instance. A desired `roc-toolkit` [API is not complete
yet](roadmap.md#roc-toolkit-apis), so the current implementation may vary.

<!-- TODO: add more details -->

### Mobile-specific

On Android, we cannot run a non-Java service that has access to sound devices that is why we cannot run such
Rust (C, C++, ...) code without wrapping it into a Java application.

Diagram below illustrates it:

<!-- TODO
![](./assets/dia/android-component-diag.svg)
/// caption
///
-->

## Glossary

| term | definition     |
| :--- | :---           |
| I/O | input/output   |
