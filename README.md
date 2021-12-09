# Interactive Simulation of Distributed Systems

Demo: https://trudi.weizenbaum-institut.de/isds_prototype/

## Getting started

1. Make sure you have some basic tools installed:

   - [Rust](https://www.rust-lang.org/learn/get-started)
   - [trunk](https://trunkrs.dev/)
     - Install (one of several options): `cargo install trunk`

1. Choose a website to build from the `sites` directory; `cd` into its directory.
1. Run: `trunk serve`
1. Open [localhost:8080](http://localhost:8080) in a browser.
1. Hack around.

Note: `trunk serve` builds without optimizations; it you want performance you should use `trunk build --release` instead.

## Running tests

If your website has unit tests, you can instruct `trunk` to run them on each build. See the supplied `Trunk.toml` files.

For testing `isds` and its components, you need to (for now) manually run `wasm-pack test` from the `isds` directory. For example:

`wasm-pack test --headless --firefox`

You can automate the running of tests on file changes using [`cargo watch`](https://crates.io/crates/cargo-watch):

`cargo watch -- wasm-pack test --headless --firefox`

## Deploy

1. Run `trunk build --release`.
1. The contents of the `dist` folder are the website; copy them to where you want the site to be hosted at.
