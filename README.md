# Quick start

This assumes that you have already installed [Rust](https://www.rust-lang.org/tools/install)

Note: If you get errors during one of the `cargo install` steps below, it will likely be because of a missing OS dependency. If you are running Debian you should make sure to run `apt-get install pkg-config` to install the `pkg-config` package.

```
rustup target add wasm32-unknown-unknown
cargo install wasm-gc
cargo install wasm-bindgen-cli
make
cd www
python3 server.py
```

When you have completed the above steps, point your browser to http://localhost:8000.
