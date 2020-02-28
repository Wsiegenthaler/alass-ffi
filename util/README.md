# alass-util

![Crates.io](https://img.shields.io/crates/v/alass-util)
[![documentation](https://docs.rs/sobol/badge.svg)](https://docs.rs/alass-util)
![Rust](https://github.com/Wsiegenthaler/sobol-rs/workflows/build/badge.svg)
![minimum rustc 1.40](https://img.shields.io/badge/rustc-1.40+-red.svg)
[![License](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](https://opensource.org/licenses/GPL-3.0)

*A Rust convenience API for subtitle synchronization with `alass-core`*

`alass-core` is a fantastic library which performs fast and accurate subtitle synchronization. `alass-util` is a wrapper library that provides various facilities to make integrating subtitle synchronization into your Rust program easier. Such facilities include:

* Loading, parsing, and saving subtitle files to/from disk
* Processing audio for voice activity using the `webrtc-vad` crate
* Converting voice activity to reference timespans
* Automatic subtitle charset detection
* Saving and loading of reference timespans to disk (useful for caching)
* Experimental support for automatic framerate correction
* Experimental support for "cleaning" voice activity data

What this crate does not provide:
* Facilities for extracting and resampling audio streams from media files

## Docs

See [docs.rs](https://docs.rs/alass-util) for API details.

## FFI

Not using Rust? See the companion [`alass-ffi`](https://github.com/wsiegenthaler/alass-util/tree/master/ffi) crate for a functionally equivalent C API.

## License

Everything in this repo is GPL-3.0 unless otherwise specified
