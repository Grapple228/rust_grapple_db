# Cargo Rust project base

This is template structure for cargo projects

## 🚴 Usage

### 🐑 Use `cargo generate` to Clone this Template

[Learn more about `cargo generate` here.](https://github.com/ashleygwilliams/cargo-generate)

```sh
cargo generate --git https://github.com/Grapple228/rust-base.git --name my-project
cd my-project
```

## Dev setup

Firstly install `cargo-watch`

```sh
cargo install cargo-watch
```

### For execution app on save, use command

```sh
cargo watch -q -c -w src/ -w .cargo/ -x run
```

### For execution test app on save, use command

```sh
cargo watch -q -c -w examples/ -w .cargo/ -x "run --example quick-dev"
```

### For execution tests on save, use command

```sh
cargo watch -q -c -x "test -q -- --nocapture"
```

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or [LINK](http://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE](LICENSE) or [LINK](http://opensource.org/licenses/MIT))

at your option.
