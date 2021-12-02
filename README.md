# Interactive Simulation of Distributed Systems; Prototype for Bitcoin Consensus

Demo: https://trudi.weizenbaum-institut.de/isds_prototype/

## Getting started

1. Make sure you have some basic tools installed:

   - [Rust](https://www.rust-lang.org/learn/get-started)
   - [trunk](https://trunkrs.dev/)
     - Install (one of several options): `$ cargo install trunk`

1. Open a terminal and run: `trunk serve`
1. Open [localhost:8080](http://localhost:8080) in a browser.
1. Hack around.
1. (optional) Press `d` on your keyboard to see some `d`ebug infos.

Note: `trunk serve` builds without optimizations; it you want performance you should use `trunk build --release` instead.

## Deploy

1. Run `trunk build --release`.
1. The contents of the `dist` folder are the website; copy them to where you want the site to be hosted at.
