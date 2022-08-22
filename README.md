# Interactive Simulation of Distributed Systems

A set of building blocks for explaining distributed systems concepts in an interactive way.
Also a few websites that use these building blocks,
most prominently our [Layers of Bitcoin](https://layers.trudi.group/).

Want to embed a cool interactive distributed system thingy on your website?
Here is [an idea](https://layers.trudi.group/network/standalone).
You can build your own!

Everything here is currently geared towards simulating and visualizing Bitcoin-like systems,
**but** the underlying simulation framework and interaction components were built with more general goals in mind.
The core framework (in `isds`) can be extended to also simulate and visualize
PBFT-like consensus protocols, DHTs, onion routing and maybe even something like TCP congestion control.
Maybe someday it even *will* be extended in some of these ways...

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

1. Run `trunk build --release --public-url URL` where `URL` is the URL at which you plan to serve the site (can also be a relative URL like `"/isds/"`; defaults to `"/"`).
1. The contents of the `dist` folder are the website; copy them to where you want the site to be hosted at.
