# dancelist

`dancelist` is software to run several websites listing folk dance events:

- https://folkdance.page/
- https://balfolk.org/
- https://balfolk.uk/

It reads data from [a separate repository](https://github.com/qwandor/dancelist-data).

This is not an officially supported Google product.

## Building and running locally

To run `dancelist` locally for development:

1. Install Rust from https://rustup.rs/.
2. Clone this repository.
3. Copy `dancelist.example.toml` to `dancelist.toml`.
4. Edit `dancelist.toml` to configure as desired. For local development comment out the `public_dir`
   line and set `events = "https://folkdance.page/index.yaml"`.
5. Build and run a local server with `RUST_LOG=info cargo run`.
6. Open http://localhost:3002 in your browser.

Use `cargo test` to run tests, and try `cargo run help` to see the various utility subcommands
available.

## License

Licensed under the Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
http://www.apache.org/licenses/LICENSE-2.0)

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be licensed under the Apache-2.0 license,
without any additional terms or conditions.

If you want to contribute to the project, see details of
[how we accept contributions](CONTRIBUTING.md).
