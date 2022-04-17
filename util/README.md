# alass-util

![Crates.io](https://img.shields.io/crates/v/alass-util)
[![documentation](https://docs.rs/alass-util/badge.svg)](https://docs.rs/alass-util)
![Build](https://github.com/Wsiegenthaler/alass-ffi/workflows/Build/badge.svg)
![minimum rustc 1.40](https://img.shields.io/badge/rustc-1.40+-red.svg)
[![License](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](https://opensource.org/licenses/GPL-3.0)

*A Rust convenience API for subtitle synchronization with `alass-core`*

`alass-core` is a fantastic library which performs fast and accurate subtitle synchronization. `alass-util` is a wrapper library that provides various facilities to make integrating subtitle synchronization into your Rust program easier. Such facilities include:

* Loading and saving subtitle files
* Processing audio for voice activity using the `webrtc-vad` crate
* Converting voice activity to reference timespans
* Automatic subtitle character set detection
* Saving and loading of reference timespans to disk
* Experimental support for automatic framerate correction
* Experimental support for "cleaning" voice activity data

What this crate does not provide:
* Facilities for extracting and resampling audio streams from media files

## Docs

See [docs.rs](https://docs.rs/alass-util) for API details.

#### Voice Activity Detector

This crate provides two options for voice activity detection:

1. [WebRTC VAD](https://crates.io/crates/webrtc-vad): The default detector used by this crate. Fast but lower quality results. See `vad-webrtc` Cargo feature.
2. [Silero](https://github.com/snakers4/silero-vad): LSTM model with much better results but longer processing time. See the `vad-silero-tract` or `vad-silero-onnx
runtime` depending on your choice of ONNX runtime. 

## FFI

Not using Rust? See the [`alass-ffi`](https://github.com/wsiegenthaler/alass-ffi/tree/master/ffi) companion crate for C bindings.

## License

Everything in this repo is GPL-3.0 unless otherwise specified
