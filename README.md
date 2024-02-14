# Damasc Workspace

## How to use

### Running Tests

```shell
cargo test
```

### Run REPL

```shell
cargo run --bin damasc-cli
```

### Run as WASM in Browser

> Requires `cargo install cargo-server` to run local webserver

```shell
wasm-pack build damasc-wasm --target web --out-dir public/wasm
cargo server --open --path damasc-wasm/public
```

### WASM with Cargo Watch

> Requires `cargo install cargo-watch`

```shell
cargo watch -- wasm-pack build damasc-wasm --target web --out-dir public/wasm
```
