# logic

A set of Rust crates for digital logic simulation and circuit analysis.

This README.md primarily discusses how to set up the project and develop it. The generated Rust docs
contain most of the technical documentation. See below for how to generate and browse the
documentation.

## Dependencies

### Easy Mode

This series of commands will install the Rust nightly toolchain and all required dependencies. If
you've already installed the rust toolchain or some of these tools, you may not need to run all of
the commands. See the details section below for more information on the dependencies.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install nightly
rustup default nightly
cargo install just
just deps
```

### Details

You must have the Rust toolchain installed. See [rustup.rs](https://rustup.rs) for details. This
crate currently requires the nightly toolchain.

If you want to use the command automations in `justfile`, then you'll need to install it globally
using `cargo` like:

```sh
cargo install just
```

At this point, if you simply want to be able to run everything, running the following command should
fetch all of the other dependencies:

```sh
just deps
```

While the above convenience command exists, the below documents the external dependencies required
in more detail.

If you want to collect code coverage statistics, you'll need to run the following commands:

```sh
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
```

To run the HTTP servers defined in the `justfile`, you'll need `http-server`, which can be installed
by running:

```sh
cargo install http-server
```

## Automation

Use `just --list` to show the various automations available in this repository and descriptions of
what they do.

## Workspaces

This repository is a Cargo workspace, containing several interdependent crates. Each crate is listed
below. While a brief description is given below, more detailed documentation for each workspace is
available through the generated rust docs. Use `just doc` to generate them, and `just doc-serve` to
start a server so the documentation can be browsed.

Further workspaces are planned as the project grows.

### device-derive

This crate is just a procedural macro for implementing the `Device` trait, which is defined in the
`foundation` crate. Ideally this would just go in the `foundation` crate, but procedural macros must
be in their own crate presently.

### foundation

This is the lowest level crate. It provides just enough structure to simulate transistors connected
together in arbitrary ways. Everything else is built on these primitives.

### gate

Logic gates are introduced here, built from transistors.

### basic

Some basic digital logic structures are built using logic gates.
