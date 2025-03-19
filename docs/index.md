# Overview

!!! warning

    This is a draft of `rocd` documentation. The project is work-in-progress and has not been released yet.

## Preface

`rocd` is a real-time audio streaming daemon that integrates audio devices and other **audio sources** and **destinations** across the network into a system where they can be **orchestrated** together in a wide variety of different ways.

`rocd` is a part of the [Roc Streaming](https://roc-streaming.org/) ecosystem. It is built on top of [Roc Toolkit](https://github.com/roc-streaming/roc-toolkit/), a library for real-time audio transport, offering features such as Forward Erasure Correction codes (FEC), adaptive latency tuning, clock drift compensation, lossy and lossless codecs, and more.

`rocd` incorporates Roc Toolkit for the transport part, [ZeroMQ](https://zeromq.org/) for the P2P signaling protocol, and implements high-level orchestration on top of that. Everything is wrapped into a simple **REST API** that you can use in your application.

![](./assets/dia/overview.svg)

## Features

- **Isolates complexities of real-time streaming** and audio I/O in a high-performant daemon with HTTP API.

    You can use any platform (e.g. Node.js or Python) without any special requirements. The real-time path, sensitive to latency and performance, is fully enclosed within the daemon.

- **Encapsulates the hassle of P2P interactions** and implements unified orchestration.

    After choosing a topology and configuring `rocd` instances, you can orchestrate the entire system via any instance (with sufficient privileges). You don't need to worry about low-level details like discovery, NAT traversal, distributed transactions, etc.

- **Inherits [real-time features](https://roc-streaming.org/toolkit/docs/about_project/features.html)** provided by Roc Toolkit.

    Roc Toolkit enables streaming of HD-quality audio with fixed or bounded latency and high robustness to packet loss. It is suitable for both low-latency, low-jitter networks like Ethernet and high-jitter unreliable networks like Wi-Fi and Internet.

- **Supports network topology** of your choice.

    You can choose between a fully peer-to-peer setup, a central server for discovery and NAT traversal, central hub/mixer/transcoder, etc. `rocd` doesn't impose a specific topology and allows you to build your own.

- Provides an easy-to-use **REST API** for:

    - **Managing audio streams**

        You can interconnect audio sources (like microphones or virtual speakers) and destinations (like speakers) across all `rocd` instances.

    - **Managing audio devices**

        You can read and write persistent configuration for both **local** and **remote** audio devices across `rocd` instances.

    - **Managing virtual devices (VADs)**

        You can create and configure Virtual Audio Devices (VADs) on local and remote instances. For example, you can create **virtual speakers** on one computer, and tell it to stream all audio played to it to speakers on another computer.

    - **Metrics and events**

        You can also collect metrics and subscribe to events across `rocd` instances.

## Platforms

!!! info

    Platforms marked with :construction: are not supported yet. See [roadmap](./roadmap.md) for status.

`rocd` aims to support the following platforms:

- **Desktop** — Computers with a display, sound card, and local apps doing audio I/O. Can be :white_check_mark: `Linux`, :white_check_mark: `macOS`, or :construction: `Windows` ^[roadmap](./roadmap.md#windows)^.

- **Mobile devices** — :white_check_mark: `Android` and :construction: `iOS` ^[roadmap](./roadmap.md#ios)^.

    For mobile platforms, we ship `rocd` as an *embeddable service* wrapped into platform-native library (Kotlin or Swift). When you integrate this library into your app, it becomes a full-featured `rocd` instance.

- **Embedded Linux** — :white_check_mark: Computers similar to PC, probably with a sound card, but often with constrained resources and no display.

- **Server Linux** — :white_check_mark: Computers similar to PC, but typically without sound card and display. Intended to host software in a proxy/hub mode.

## Use cases

*TBD*

## Applications

- [**Roc Cast**](https://github.com/roc-streaming/roc-cast) is an open-source end-user application for Home Audio and AOIP, built on top of `rocd`.

    For details on how the two projects are integrated, see [Relation to Roc Cast](./implementation/roc_cast.md).
