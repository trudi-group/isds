# Interactive Simulation of Distributed Systems; Prototype for Bitcoin Consensus

*isds* - pronounced like *eye-see/dee-see*, similarly to the name of that Australian rock band...

## Getting started

1. Make sure you have some basic tools installed:

   - [Rust](https://www.rust-lang.org/learn/get-started)
   - [cargo-make](https://sagiegurari.github.io/cargo-make/)
     - Install: `$ cargo install cargo-make`

1. Open a terminal and run: `cargo make serve`
1. Open a second terminal and run: `cargo make watch`
1. Open [localhost:8000](http://localhost:8000) in a browser.
1. Hack around.
1. Refresh your browser to see changes.

Note: `cargo watch` builds without optimizations; it you want performance you should use `cargo make build_release` instead.

## Prepare your project for deploy

1. Run `cargo make verify` in your terminal to format and lint the code.
1. Run `cargo make build_release`.
1. Upload `index.html` and `pkg` into your server's public folder.
