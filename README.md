> <h1>PROJECT IS WORK IN PROGRESS!</h1>

# rocd: audio streaming daemon

[![Build](https://github.com/roc-streaming/rocd/actions/workflows/build.yaml/badge.svg)](https://github.com/roc-streaming/rocd/actions/workflows/build.yaml) [![Matrix chat](https://matrix.to/img/matrix-badge.svg)](https://app.element.io/#/room/#roc-streaming:matrix.org)

## About project

`rocd` is a real-time audio streaming daemon with [REST API](https://roc-streaming.org/daemon/).

With it, you can:

* inspect and control audio devices
* initiate audio streaming between local devices and remote peers
* create virtual devices that automatically stream all sound to/from remote peers

The key idea is to isolate application code from complexities of network streaming and audio I/O.

Applications can be written in any language (e.g. node.js or python) and don't have any special requirements. Real-time path, sensitive to latency and performance, is fully enclosed inside `rocd`.

## Features

Internally, `rocd` is based on [Roc Toolkit](https://github.com/roc-streaming/roc-toolkit/), which gives you:

* streaming high-quality audio with guaranteed latency
* robust work on unreliable networks like Wi-Fi, due to use of Forward Erasure Correction codes
* multiple profiles for different CPU and latency requirements
* portability and low requirements to hardware

With `rocd`, these benefits are combined with a high-level, language-independent, and easy to use HTTP API. Basically you just say "stream from here to there" and it works.

## Use cases

*TBD*

## Technology

`rocd` is built on top of several big lower-level technologies:

* For streaming:

   * [roc-toolkit](https://github.com/roc-streaming/roc-toolkit/) - real-time streaming library that combines high quality, guaranteed latency, and loss repair
   * [roc-go](https://github.com/roc-streaming/roc-go/) - golang bindings to Roc Toolkit

* For virtual devices:

   * [roc-sink](https://docs.pipewire.org/page_module_roc_sink.html) and
    [roc-source](https://docs.pipewire.org/page_module_roc_source.html) PipeWire modules - virtual devices for Linux with PipeWire
   * [roc-pulse](https://github.com/roc-streaming/roc-pulse) - virtual devices for Linux with PulseAudio
   * [roc-vad](https://github.com/roc-streaming/roc-vad) - virtual devices for macOS CoreAudio

## Compatibility

Since `rocd` is based on Roc Toolkit, it is automatically interoperable with all other software based on it.

Remote peer can be:

* another instance of `rocd`
* apps using [C library](https://roc-streaming.org/toolkit/docs/api.html) or its [bindings to other languages](https://roc-streaming.org/toolkit/docs/api/bindings.html)
* [command-line tools](https://roc-streaming.org/toolkit/docs/tools/command_line_tools.html)
* [sound server modules](https://roc-streaming.org/toolkit/docs/tools/sound_server_modules.html) (PulseAudio, PipeWire, macOS CoreAudio)
* [Roc Droid](https://github.com/roc-streaming/roc-droid/) (Android app)

Third-party RTP peers are also supported, given that they implement all necessary extensions.

## Platforms

Currently supported platforms:

- [x] Linux / PipeWire
- [x] Linux / PulseAudio
- [ ] Linux / ALSA
- [ ] macOS / CoreAudio
- [ ] Windows / WASAPI

## Releases

*TBD*

## Installation

*TBD*

## Usage

There are two ways to use `rocd`:

* via REST API
* via web interface

In addition, `rocd` can be configured via command-line flags and configuration file.

For further details, see [USAGE.md](USAGE.md).

## Hacking

Contributions in any form are always welcome! You can find issues needing help using [help wanted](https://github.com/roc-streaming/rocd/labels/help%20wanted) and [good first issue](https://github.com/roc-streaming/rocd/labels/good%20first%20issue) labels.

If you would like to dig into the project internals, have a look at [HACKING.md](HACKING.md).

## Authors

See [here](https://github.com/roc-streaming/rocd/graphs/contributors).

## License

Contents of the repository is licensed under [MPL-2.0](LICENSE).

For details on Roc Toolkit licensing, see [here](https://roc-streaming.org/toolkit/docs/about_project/licensing.html).
