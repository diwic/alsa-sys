#!/bin/sh

RUST_LOG=bindgen cargo run --features "use-bindgen" --bin regenerate_bindings
