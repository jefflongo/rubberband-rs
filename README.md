# rubberband-rs

Rust bindings for [Rubber Band Library](https://breakfastquay.com/rubberband/). This crate supports Rubber Band Library v4.0.0.

## Dependencies

Rubber Band Library does not need to be installed on the system. The library will be built by the included `rubberband-sys` crate, which uses [`bindgen`](https://github.com/rust-lang/rust-bindgen). Therefore, Clang 9.0 or greater is required.
